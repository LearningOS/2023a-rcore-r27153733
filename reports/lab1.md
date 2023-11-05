## 简答

### 1.正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容 (运行 [Rust 三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test-2023A/tree/master/src/bin) ， 注意在编译时至少需要指定 `LOG=ERROR` 才能观察到内核的报错信息) ， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

qemu8.1.2
sbi应该是最新的

```bash
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003c4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```

1. 访问非法地址
2. 非法指令异常（sret）
3. 非法指令异常（获取sstatus值）

### 2.深入理解 [trap.S](https://github.com/LearningOS/rCore-Tutorial-Code-2023A/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题

- #### L40：刚进入 `__restore` 时，`a0` 代表了什么值。请指出 `__restore` 的两种使用情景。

  a0值:     `riscv`规范规定`a0`为函数返回值，这里指`trap_handler`的返回值，即保存在内核栈的`Trap`上下文

  __restore 用法:    通过`goto_restore`初始化`task`的`Trap`上下文 或 `Trap`处理完成后恢复`task`的`Trap`上下文

- #### L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

  ```asm
  ld t0, 32*8(sp) # 恢复sstatus
  ld t1, 33*8(sp) # 恢复sepc
  ld t2, 2*8(sp) # 恢复sscratch
  csrw sstatus, t0 # 记录特权级等信息
  csrw sepc, t1 # Trap 处理完成后默认会执行的下一条指令的地址
  csrw sscratch, t2 # 用户栈
  ```

- #### L50-L56：为何跳过了 `x2` 和 `x4`？

  x2(sp)后面保存了

  x4(tp)应用程序用不到

- #### L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

  该指令用于交换，执行之后，`sp`为用户栈`` `sscratch`为`内核栈。

- #### `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

  `sret`

  作用是：

  - 将当前的特权级按照 `sstatus` 的 `SPP` 字段设置为 U 或者 S （进入用户态的原因）
  - 跳转到 `sepc` 寄存器指向的那条指令，然后继续执行

- #### L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

  该指令用于交换，执行之后，`sp`为内核栈 `sscratch`为用户栈。

- #### 从 U 态进入 S 态是哪一条指令发生的？

  `ecall`

## 实验

实现了：

查询当前正在执行的任务状态、任务使用的系统调用次数、系统调用时刻距离任务第一次被调度时刻的时长（单位ms）。

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > 

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

