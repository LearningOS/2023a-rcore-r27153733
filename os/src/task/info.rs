//! Implementation of [`TaskInfo`]

pub const MAX_SYSCALL_NUM: usize = 500;
pub const INVALID_TIME: usize = usize::MAX;

#[derive(Copy, Clone)]
#[repr(C)]
/// task context structure containing some registers
pub struct TaskInfo {
    // status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Program start time
    pub start_time: usize
}

impl TaskInfo {
    /// Create a new empty task context
    pub fn zero_init() -> Self {
        Self {
            syscall_times: [0; MAX_SYSCALL_NUM],
            start_time: INVALID_TIME
        }
    }
    // /// Create a new task context with a trap return addr and a kernel stack pointer
    // pub fn goto_restore(kstack_ptr: usize) -> Self {
    //     extern "C" {
    //         fn __restore();
    //     }
    //     Self {
    //         ra: __restore as usize,
    //         sp: kstack_ptr,
    //         s: [0; 12],
    //     }
    // }
}
