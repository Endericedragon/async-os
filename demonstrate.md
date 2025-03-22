# AsyncOS 作为Dialer和外部Listener协商的演示

本文讲述了AsyncOS作为Dialer和外部Listener协商的过程。

## 相关源代码

- `modules/syscall/src/syscall_net/multistream_select.rs`：负责将dialer支持的协议等数据包装并送给内核，以及协商完成后的数据收发等操作
- `modules/syscall/src/syscall_net/imp.rs`：新增系统调用syscall_multistream_select_dialer和上个文件的ecall调用对接，在其中完成协商操作后，返回本次通信的socket fd和选择的协议
- `user_apps/hello_world`：充当dialer，调用上述系统调用，和listener协商使用/echo/1.0协议（自创的实验性协议，listener将重复所有dialer发送的信息）
- 以及运行在Windows用户态的listener，相关代码未放在AsyncOS中。

## 演示过程

1. 启动外部Listener。
2. 将 `apps/user_boot/src/main.rs` 中的 `BUSYBOX_TESTCASES` 数组更改为 `["hello_world"]`，以指定运行的测试用例。
3. 执行 `sh run_asyncos.sh`，启动AsyncOS。AsyncOS将进入用户态，并开始执行hello_world测试用例。
4. 观察外部Listener的输出，能看到它接收到AsyncOS建议的协议并做出回复。当最终协商至 `/echo/1.0` 时，将显示协商成功的信息。
5. 在AsyncOS的控制台中输入任意信息，将能收到Listener回复的完全相同的信息，因为这就是 `echo/1.0` 所规定的行为。