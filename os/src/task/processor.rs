//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{MapPermission, VirtAddr};
use crate::sync::UPSafeCell;
use crate::timer::get_time_us;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            if task_inner.start_time == 0 {
                task_inner.start_time = get_time_us();
            }
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

/// Incr the number of system calls
pub fn incr_syscall_times(syscall_id: usize) {
    let task = current_task().unwrap();
    task.inner_exclusive_access().syscall_times[syscall_id] += 1;
}

/// Get the number of system calls
pub fn get_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    let task = current_task().unwrap();
    let res = task.inner_exclusive_access().syscall_times;
    res
}
/// Get the program start running time
pub fn get_program_start_time() -> usize {
    let task = current_task().unwrap();

    let res = task.inner_exclusive_access().start_time;
    res
}

/// 申请物理内存，将其映射到 start_va 开始, end_va 结束的虚存，内存页属性为 port
pub fn mmap(start_va: VirtAddr, end_va: VirtAddr, port: usize) -> isize {
    if !start_va.aligned() || port & !0x7 != 0 || port & 0x7 == 0 {
        return -1;
    }

    let task = current_task().unwrap();

    let mut permission: MapPermission = MapPermission::U | MapPermission::U;
    if port & 0b1 == 0b1 {
        permission |= MapPermission::R;
    }
    if port & 0b10 == 0b10 {
        permission |= MapPermission::W;
    }
    if port & 0b100 == 0b100 {
        permission |= MapPermission::X;
    }

    let res = task
        .inner_exclusive_access()
        .memory_set
        .mmap(start_va, end_va, permission);
    res
}

/// 取消 start_va 开始, end_va 结束的虚存的映射
pub fn munmap(start_va: VirtAddr, end_va: VirtAddr) -> isize {
    if !start_va.aligned() {
        return -1;
    }
    let task = current_task().unwrap();
    let res = task
        .inner_exclusive_access()
        .memory_set
        .munmap(start_va, end_va);
    res
}
