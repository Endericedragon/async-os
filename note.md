# async-os学习研究笔记

## 问题汇总

> 新版本AsyncOS如何运行用户态程序？

直接在 `user_apps/hello_world` 项目中编写代码即可。编写完成后，直接在根目录下执行以下命令：

```sh
cd user_apps && make build_uapps && cd ..
make A=apps/user_boot ARCH=riscv64 LOG=off SMP=1 FEATURES=sched_fifo,img BLK=y run
```

如果提示 `error: Could not find incbin file 'vdso/target/riscv64gc-unknown-linux-musl/release/libcops.so'` ，则需要到 `vdso` 目录下执行 `make build` 命令。

如果提示 `Error loading shared library libgcc_s.so.1` ，则有两种办法：

- 指定rustc让它静态链接。方法是在 `user_apps/.cargo/config.toml` 中写：

  ```toml
  [target.riscv64gc-unknown-linux-musl]
  linker = "riscv64-linux-musl-gcc" # 原本就有
  rustflags = ["-C", "target-feature=+crt-static"] # 指定静态链接
  ```
- 在 `modules/trampoline/src/fs_api.rs` 的 `fs_init` 函数中，手动指定缺失的文件的链接情况。此处的情况是，`libgcc_s.so.1` 位于 `tool-libs` 中，需要将其手动复制到 `testcases/riscv64_linux_musl` 中，才能正确链接到它。

如果提示找不到 qemu-system-riscv64，则需要确认已安装 qemu-riscv64 软件包，然后在 `scripts/make/qemu.mk` 中修改 `QEMU` 变量为 qemu-system-riscv64 所在的路径。

编译完成后，将ELF可执行文件的文件名放到 `apps/user_boot/src/main.rs` 的 `BUSYBOX_TESTCASES` 数组中，即可运行。

> 运行 chat-example 遇到的问题

组会上提及，AsyncOS无法运行 rust-libp2p 官方提供的 chat 示例代码（位于本仓库的 `user_apps/chat_example` 中）。其中，核心报错就一条：`Failed building the Runtime: Os { code: 9, kind: Uncategorized, message: "Bad file descriptor" }[  1.795876 0:3 syscall::syscall:57]` 。将日志等级开到info之后，获得了如下的错误报告：

```
Entering user_boot...
[  0.384363 0:1 executor::link:247] create_link: /lib/ld-musl-riscv64-sf.so.1 -> /libc.so
[  0.388335 0:1 executor::link:247] create_link: /lib/ld-musl-riscv64.so.1 -> /libc.so
[  0.391642 0:1 executor::link:247] create_link: /lib/tls_get_new-dtv_dso.so -> /tls_get_new-dtv_dso.so
[  0.395912 0:1 executor::link:247] create_link: /usr/sbin/ls -> /busybox
[  0.398916 0:1 executor::link:247] create_link: /usr/bin/ls -> /busybox
[  0.400240 0:1 executor::link:247] create_link: /bin/ls -> /busybox
[  0.401701 0:1 executor::link:247] create_link: /usr/sbin/mkdir -> /busybox
[  0.403442 0:1 executor::link:247] create_link: /usr/bin/mkdir -> /busybox
[  0.404547 0:1 executor::link:247] create_link: /bin/mkdir -> /busybox
[  0.406108 0:1 executor::link:247] create_link: /usr/sbin/touch -> /busybox
[  0.407851 0:1 executor::link:247] create_link: /usr/bin/touch -> /busybox
[  0.409066 0:1 executor::link:247] create_link: /bin/touch -> /busybox
[  0.410241 0:1 executor::link:247] create_link: /usr/sbin/mv -> /busybox
[  0.411437 0:1 executor::link:247] create_link: /usr/bin/mv -> /busybox
[  0.412680 0:1 executor::link:247] create_link: /bin/mv -> /busybox
[  0.413628 0:1 executor::link:247] create_link: /usr/sbin/busybox -> /busybox
[  0.414945 0:1 executor::link:247] create_link: /usr/bin/busybox -> /busybox
[  0.416327 0:1 executor::link:247] create_link: /bin/busybox -> /busybox
[  0.417627 0:1 executor::link:247] create_link: /usr/sbin/sh -> /busybox
[  0.419093 0:1 executor::link:247] create_link: /usr/bin/sh -> /busybox
[  0.420389 0:1 executor::link:247] create_link: /bin/sh -> /busybox
[  0.421489 0:1 executor::link:247] create_link: /usr/sbin/which -> /busybox
[  0.422789 0:1 executor::link:247] create_link: /usr/bin/which -> /busybox
[  0.424111 0:1 executor::link:247] create_link: /bin/which -> /busybox
[  0.425179 0:1 executor::link:247] create_link: /usr/sbin/cp -> /busybox
[  0.426316 0:1 executor::link:247] create_link: /usr/bin/cp -> /busybox
[  0.427474 0:1 executor::link:247] create_link: /bin/cp -> /busybox
[  0.429064 0:1 executor::link:247] create_link: /bin/lmbench_all -> /lmbench_all
[  0.431843 0:1 executor::link:247] create_link: /bin/iozone -> /iozone
[  0.434867 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/ld-musl-riscv64.so.1 -> /lib/libc.so
[  0.440627 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libatomic.so -> /riscv64-linux-musl-native/lib/libatomic.so.1.2.0
[  0.445546 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libatomic.so.1 -> /riscv64-linux-musl-native/lib/libatomic.so.1.2.0
[  0.450746 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libgfortran.so -> /riscv64-linux-musl-native/lib/libgfortran.so.5.0.0
[  0.455889 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libgfortran.so.5 -> /riscv64-linux-musl-native/lib/libgfortran.so.5.0.0
[  0.461349 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libgomp.so -> /riscv64-linux-musl-native/lib/libgomp.so.1.0.0
[  0.466023 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libgomp.so.1 -> /riscv64-linux-musl-native/lib/libgomp.so.1.0.0
[  0.471330 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libssp.so -> /riscv64-linux-musl-native/lib/libssp.so.0.0.0
[  0.477065 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libssp.so.0 -> /riscv64-linux-musl-native/lib/libssp.so.0.0.0
[  0.481803 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libstdc++.so -> /riscv64-linux-musl-native/lib/libstdc++.so.6.0.29
[  0.486956 0:1 executor::link:247] create_link: /riscv64-linux-musl-native/lib/libstdc++.so.6 -> /riscv64-linux-musl-native/lib/libstdc++.so.6.0.29
[  0.492373 0:1 executor::link:247] create_link: /lib/libgcc_s.so.1 -> /libgcc_s.so.1
[  1.479575 0:1 executor::loader:56] load app args: ["chat_example"] name: chat_example
[  1.480956 0:1 executor::loader:58] The elf base addr may be different in different arch!
[  1.482061 0:1 elf_parser:142] Base addr for the elf: 0x0
[  1.483309 0:1 elf_parser:66] Base addr for the elf: 0x0
[  1.488206 0:1 elf_parser::arch::riscv:54] Base addr for the elf: 0x0
[  1.489603 0:1 elf_parser::arch::riscv:168] Relocating done
[  1.511333 0:1 executor::loader:96] [new region] user heap: [VA:0x3fa00000, VA:0x3fe00000)
[  1.512592 0:1 elf_parser::auxv:48] ELF header addr: 0x10000
[  1.519885 0:1 executor::loader:119] [new region] user stack: [VA:0x3fe00000, VA:0x40000000)
[  1.524597 0:3 syscall::syscall:42] [syscall] id = GETEUID, args = [1073741344, 0, 0, 48, 65600, 0], entry
[  1.526730 0:3 syscall::syscall:57] [syscall] id = 175,return 0
[  1.528209 0:3 syscall::syscall:42] [syscall] id = GETUID, args = [0, 0, 0, 48, 65600, 0], entry
[  1.529604 0:3 syscall::syscall:57] [syscall] id = 174,return 0
[  1.530817 0:3 syscall::syscall:42] [syscall] id = GETEGID, args = [0, 0, 0, 48, 65600, 1], entry
[  1.532939 0:3 syscall::syscall:57] [syscall] id = 177,return 0
[  1.534185 0:3 syscall::syscall:42] [syscall] id = GETGID, args = [0, 0, 0, 48, 65600, 1], entry
[  1.535797 0:3 syscall::syscall:57] [syscall] id = 176,return 0
[  1.537510 0:3 syscall::syscall:20] [syscall] id = BRK, args = [0, 1073740872, 0, 512, 31, 0], entry
[  1.539877 0:3 syscall::syscall:57] [syscall] id = 214,return 1067450368
[  1.541833 0:3 syscall::syscall:20] [syscall] id = BRK, args = [1067453760, 1073740872, 0, 512, 6507256, 1067453760], entry
[  1.544345 0:3 syscall::syscall:57] [syscall] id = 214,return 1067453760
[  1.545983 0:3 syscall::syscall:42] [syscall] id = SET_TID_ADDRESS, args = [1067450576, 6047088, 0, 6506816, 1067450560, 6506816], entry
[  1.549520 0:3 syscall::syscall:57] [syscall] id = 96,return 3
[  1.551490 0:3 syscall::syscall:42] [syscall] id = SET_ROBUST_LIST, args = [1067450592, 24, 18446744073709551584, 1, 6506592, 1067450592], entry
[  1.555779 0:3 syscall::syscall:57] [syscall] id = 99,return 0
[  1.558875 0:3 syscall::syscall:42] [syscall] id = UNAME, args = [1073740616, 643575647, 949831348063, 182369249931103, 28892816874418015, 3703830112808742656], entry
[  1.561791 0:3 syscall::syscall:57] [syscall] id = 160,return 0
[  1.563480 0:3 syscall::syscall:42] [syscall] id = PRLIMIT64, args = [0, 3, 0, 1073740984, 5318696, 3], entry
[  1.566095 0:3 syscall::syscall:57] [syscall] id = 261,return 0
[  1.567951 0:3 syscall::syscall:30] [syscall] id = PREADLINKAT, args = [18446744073709551516, 5309104, 1073736704, 4096, 0, 18446744073709547520], entry
[  1.572049 0:3 syscall::syscall_fs::imp::io:673] read link at: /proc/self/exe
[  1.573798 0:3 syscall::syscall:57] [syscall] id = 78,return 13
[  1.575060 0:3 syscall::syscall:42] [syscall] id = GETRANDOM, args = [6503912, 8, 1, 99, 1, 0], entry
[  1.577047 0:3 syscall::syscall:57] [syscall] id = 278,return 8
[  1.578866 0:3 syscall::syscall:20] [syscall] id = BRK, args = [1067588928, 6471864, 0, 688, 18446744073709547520, 1067588928], entry
[  1.581323 0:3 syscall::syscall:57] [syscall] id = 214,return 1067453760
[  1.584142 0:3 syscall::syscall:20] [syscall] id = MMAP, args = [0, 1048576, 3, 34, 18446744073709551615, 0], entry
[  1.586112 0:3 syscall::syscall_mem::imp:52] flags: MMAPFlags(MAP_PRIVATE | MAP_ANONYMOUS)
[  1.588449 0:3 async_mem:277] [mmap] vaddr: [VA:0x0, VA:0x100000), MappingFlags(READ | WRITE | USER), shared: false, fixed: false, backend: false
[  1.591055 0:3 async_mem:297] find free area
[  1.593096 0:3 async_mem:302] found area [VA:0x635000, VA:0x735000)
[  1.595436 0:3 syscall::syscall:57] [syscall] id = 222,return 6508544
[  1.599693 0:3 syscall::syscall:20] [syscall] id = MPROTECT, args = [6045696, 417792, 1, 6046920, 18446744073709547520, 6046920], entry
[  1.602360 0:3 async_mem:347] [mprotect] addr: [VA:0x5c4000, VA:0x62a000), flags: MappingFlags(READ | USER)
[  1.606286 0:3 syscall::syscall:57] [syscall] id = 226,return 0
[  1.608503 0:3 syscall::syscall:30] [syscall] id = PPOLL, args = [1073740456, 3, 1073740344, 0, 0, 0], entry
[  1.611194 0:3 syscall::syscall:57] [syscall] id = 73,return 0
[  1.613107 0:3 syscall::syscall:42] [syscall] id = SIGACTION, args = [13, 1073739720, 1073739864, 8, 18446744073709551271, 268435456], entry
[  1.616409 0:3 syscall::syscall_task::imp::signal:20] signum: 13, action: 3FFFF7C8, old_action: 3FFFF858
[  1.618502 0:3 syscall::syscall:57] [syscall] id = 134,return 0
[  1.620433 0:3 syscall::syscall:30] [syscall] id = OPENAT, args = [18446744073709551516, 5292744, 524288, 0, 4259840, 0], entry
[  1.624007 0:3 syscall::syscall_fs::imp::io:412] path: "/proc/self/maps"
[  1.626577 0:3 syscall::syscall:57] [syscall] id = 56,return -2
[  1.628355 0:3 syscall::syscall:42] [syscall] id = SIGACTION, args = [11, 0, 1073740200, 8, 1, 0], entry
[  1.629982 0:3 syscall::syscall_task::imp::signal:20] signum: 11, action: 0, old_action: 3FFFF9A8
[  1.631605 0:3 syscall::syscall:57] [syscall] id = 134,return 0
[  1.633742 0:3 syscall::syscall:42] [syscall] id = SIGALTSTACK, args = [0, 1073740280, 0, 1073740576, 0, 0], entry
[  1.636598 0:3 syscall::syscall:57] [syscall] id = 132,return 0
[  1.638532 0:3 syscall::syscall:20] [syscall] id = MMAP, args = [0, 12288, 3, 131106, 18446744073709551615, 0], entry
[  1.641008 0:3 syscall::syscall_mem::imp:52] flags: MMAPFlags(MAP_PRIVATE | MAP_ANONYMOUS | MAP_STACK)
[  1.643171 0:3 async_mem:277] [mmap] vaddr: [VA:0x0, VA:0x3000), MappingFlags(READ | WRITE | USER), shared: false, fixed: false, backend: false
[  1.645634 0:3 async_mem:297] find free area
[  1.646714 0:3 async_mem:302] found area [VA:0x1000, VA:0x4000)
[  1.649076 0:3 syscall::syscall:57] [syscall] id = 222,return 4096
[  1.651107 0:3 syscall::syscall:20] [syscall] id = MPROTECT, args = [4096, 4096, 0, 131106, 18446744073709551615, 18446744073709547520], entry
[  1.654952 0:3 async_mem:347] [mprotect] addr: [VA:0x1000, VA:0x2000), flags: MappingFlags(USER)
[  1.659481 0:3 syscall::syscall:57] [syscall] id = 226,return 0
[  1.660856 0:3 syscall::syscall:42] [syscall] id = SIGALTSTACK, args = [1073740280, 0, 0, 131106, 18446744073709551615, 18446744073709547520], entry
[  1.665149 0:3 syscall::syscall:57] [syscall] id = 132,return 0
[  1.666405 0:3 syscall::syscall:42] [syscall] id = SIGACTION, args = [11, 1073740056, 0, 8, 18446744073709551223, 134217732], entry
[  1.668873 0:3 syscall::syscall_task::imp::signal:20] signum: 11, action: 3FFFF918, old_action: 0
[  1.671079 0:3 syscall::syscall:57] [syscall] id = 134,return 0
[  1.672656 0:3 syscall::syscall:42] [syscall] id = SIGACTION, args = [7, 0, 1073740200, 8, 1, 0], entry
[  1.674406 0:3 syscall::syscall_task::imp::signal:20] signum: 7, action: 0, old_action: 3FFFF9A8
[  1.675416 0:3 syscall::syscall:57] [syscall] id = 134,return 0
[  1.676611 0:3 syscall::syscall:42] [syscall] id = SIGACTION, args = [7, 1073740056, 0, 8, 18446744073709551223, 134217732], entry
[  1.678547 0:3 syscall::syscall_task::imp::signal:20] signum: 7, action: 3FFFF918, old_action: 0
[  1.680058 0:3 syscall::syscall:57] [syscall] id = 134,return 0
[  1.682241 0:3 syscall::syscall:42] [syscall] id = GETRANDOM, args = [1073713528, 16, 4, 0, 6508560, 0], entry
[  1.683993 0:3 syscall::syscall:57] [syscall] id = 278,return 16
[  1.685217 0:3 syscall::syscall:30] [syscall] id = OPENAT, args = [18446744073709551516, 1073711560, 524288, 0, 4259840, 0], entry
[  1.688362 0:3 syscall::syscall_fs::imp::io:412] path: "/proc/self/cgroup"
[  1.689770 0:3 syscall::syscall:57] [syscall] id = 56,return -2
[  1.691002 0:3 syscall::syscall:42] [syscall] id = SCHED_GETAFFINITY, args = [0, 128, 1073712960, 2147483647, 1073712960, 128], entry
[  1.694466 0:3 syscall::syscall:57] [syscall] id = 123,return 1
[  1.695770 0:3 syscall::syscall:30] [syscall] id = OPENAT, args = [18446744073709551516, 5302376, 524288, 0, 4259840, 0], entry
[  1.698104 0:3 syscall::syscall_fs::imp::io:412] path: "/sys/devices/system/cpu/online"
[  1.700106 0:3 executor::link:247] create_link: /sys/devices/system/cpu/online -> /sys/devices/system/cpu/online
[  1.703990 0:3 syscall::syscall:57] [syscall] id = 56,return 3
[  1.705044 0:3 syscall::syscall:30] [syscall] id = READ, args = [3, 1073710728, 1024, 10, 1073711752, 1073710728], entry
[  1.707064 0:3 syscall::syscall_fs::imp::io:30] [read()] fd: 3, buf: 0x3fff8688, len: 1024
[  1.709361 0:3 syscall::syscall:57] [syscall] id = 63,return 3
[  1.710635 0:3 syscall::syscall:30] [syscall] id = CLOSE, args = [3, 0, 10, 8, 1073710731, 1073710731], entry
[  1.713090 0:3 syscall::syscall_fs::imp::io:473] Into syscall_close. fd: 3
[  1.714657 0:3 syscall::syscall:57] [syscall] id = 57,return 0
[  1.716187 0:3 syscall::syscall:30] [syscall] id = EPOLL_CREATE, args = [524288, 1, 1, 0, 1024, 0], entry
[  1.718354 0:3 syscall::syscall:57] [syscall] id = 20,return 3
[  1.719768 0:3 syscall::syscall:30] [syscall] id = EVENTFD, args = [0, 526336, 1, 0, 1024, 0], entry
[  1.721852 0:3 syscall::syscall:57] [syscall] id = 19,return 4
[  1.723317 0:3 syscall::syscall:30] [syscall] id = EPOLL_CTL, args = [3, 1, 4, 1073711624, 1024, 0], entry
[  1.726080 0:3 syscall::syscall:57] [syscall] id = 21,return -9
[  1.727405 0:3 syscall::syscall:30] [syscall] id = CLOSE, args = [4, 1, 4, 1073711624, 1024, 0], entry
[  1.729054 0:3 syscall::syscall_fs::imp::io:473] Into syscall_close. fd: 4
[  1.730536 0:3 syscall::syscall:57] [syscall] id = 57,return 0
[  1.732494 0:3 syscall::syscall:30] [syscall] id = CLOSE, args = [3, 1, 4, 1073711624, 1024, 0], entry
[  1.735260 0:3 syscall::syscall_fs::imp::io:473] Into syscall_close. fd: 3
[  1.737077 0:3 syscall::syscall:57] [syscall] id = 57,return 0
[  1.742906 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 4569320, 8, 3485508, 0, 0], entry
[  1.745015 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x45b8e8, len: 8
thread '[  1.747567 0:3 syscall::syscall:57] [syscall] id = 64,return 8
[  1.748755 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 4590076, 4, 0, 0, 0], entry
[  1.750593 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x4609fc, len: 4
main[  1.753214 0:3 syscall::syscall:57] [syscall] id = 64,return 4
[  1.754385 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 5242551, 14, 3485508, 0, 0], entry
[  1.756258 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x4ffeb7, len: 14
' panicked at [  1.758059 0:3 syscall::syscall:57] [syscall] id = 64,return 14
[  1.759330 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 4600265, 34, 0, 6073084, 0], entry
[  1.761110 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x4631c9, len: 34
user_apps/chat_example/src/main.rs[  1.763412 0:3 syscall::syscall:57] [syscall] id = 64,return 34
[  1.764799 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 5263167, 1, 3485508, 6073084, 0], entry
[  1.766721 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x504f3f, len: 1
:[  1.768417 0:3 syscall::syscall:57] [syscall] id = 64,return 1
[  1.769703 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 1073712430, 2, 0, 0, 0], entry
[  1.771400 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x3fff8d2e, len: 2
74[  1.774657 0:3 syscall::syscall:57] [syscall] id = 64,return 2
[  1.775841 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 5263167, 1, 3485508, 0, 0], entry
[  1.777761 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x504f3f, len: 1
:[  1.779039 0:3 syscall::syscall:57] [syscall] id = 64,return 1
[  1.779917 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 1073712431, 1, 0, 0, 0], entry
[  1.781242 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x3fff8d2f, len: 1
7[  1.784077 0:3 syscall::syscall:57] [syscall] id = 64,return 1
[  1.785920 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 5241922, 2, 3485508, 0, 0], entry
[  1.787590 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x4ffc42, len: 2
:
[  1.789209 0:3 syscall::syscall:57] [syscall] id = 64,return 2
[  1.790438 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 6510400, 96, 0, 0, 0], entry
[  1.792900 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x635740, len: 96
Failed building the Runtime: Os { code: 9, kind: Uncategorized, message: "Bad file descriptor" }[  1.795876 0:3 syscall::syscall:57] [syscall] id = 64,return 96
[  1.797090 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 5263123, 1, 3485508, 0, 0], entry
[  1.799192 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x504f13, len: 1

[  1.800504 0:3 syscall::syscall:57] [syscall] id = 64,return 1
[  1.801820 0:3 syscall::syscall:30] [syscall] id = WRITE, args = [2, 5242565, 78, 3485508, 0, 0], entry
[  1.802822 0:3 syscall::syscall_fs::imp::io:105] [write()] fd: 2, buf: 0x4ffec5, len: 78
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
[  1.804609 0:3 syscall::syscall:57] [syscall] id = 64,return 78
[  1.832267 0:3 syscall::syscall:42] [syscall] id = FUTEX, args = [6507924, 129, 2147483647, 0, 8, 2], entry
[  1.834347 0:3 executor::futex:143] [futex_wake] vaddr: VA:0x634d94, flags: 0, nr_waken: 2147483647
[  1.836108 0:3 axfutex::queues:17] Initializing futex queues
[  1.837180 0:3 executor::futex:152] hash_bucket is empty
[  1.838017 0:3 syscall::syscall:57] [syscall] id = 98,return 0
[  1.840764 0:3 syscall::syscall:42] [syscall] id = SIGALTSTACK, args = [1073740184, 0, 1, 0, 2, 1067452848], entry
[  1.844469 0:3 syscall::syscall:57] [syscall] id = 132,return 0
[  1.845698 0:3 syscall::syscall:20] [syscall] id = MUNMAP, args = [4096, 12288, 1, 0, 2, 1067452848], entry
[  1.848033 0:3 async_mem:317] [munmap] [VA:0x1000, VA:0x4000)
[  1.849698 0:3 async_mem:188] splitting for [VA:0x1000, VA:0x4000)
[  1.851244 0:3 async_mem:193]   drop [VA:0x1000, VA:0x2000)
[  1.853333 0:3 async_mem:193]   drop [VA:0x2000, VA:0x4000)
[  1.854894 0:3 syscall::syscall:57] [syscall] id = 215,return 0
[  1.857036 0:3 async_mem:422] Page fault address VA:0x0 not found in memory set
[  1.859686 0:3 executor::signal:177] cpu: 0, task: 3, handler signal: 11
[  1.861476 0:3 executor::signal:259] use stack: 0x3ffff930
[  1.863505 0:3 executor::signal:266] restorer :0x40000000, handler: 0x352014
[  1.866003 0:3 syscall::syscall:42] [syscall] id = SIGACTION, args = [11, 1073738600, 0, 8, 18446744073709551231, 0], entry
[  1.868630 0:3 syscall::syscall_task::imp::signal:20] signum: 11, action: 3FFFF368, old_action: 0
[  1.870440 0:3 syscall::syscall:57] [syscall] id = 134,return 0
[  1.872873 0:3 syscall::syscall:42] [syscall] id = SIGRETURN, args = [0, 1073738600, 0, 8, 0, 0], entry
[  1.875352 0:3 syscall::syscall:57] [syscall] id = 139,return 0
[  1.877121 0:3 async_mem:422] Page fault address VA:0x0 not found in memory set
[  1.878839 0:3 executor::signal:177] cpu: 0, task: 3, handler signal: 11
[  1.880805 0:3 executor::signal:123] Terminate process: 2
[  1.882960 0:3 executor::api:68] exit task id 3 with code _11_
[  1.884695 0:3 executor::futex:143] [futex_wake] vaddr: VA:0x3fa000d0, flags: 0, nr_waken: 1
[  1.887487 0:3 executor::futex:152] hash_bucket is empty
task count 1
[  1.893313 0:1 axhal::platform::riscv64_qemu_virt::misc:3] Shutting down...
```

> 条件编译一直标红线怎么办？

把条件干掉。目前受害者如下：

- `modules/taskctx/Cargo.toml`

> 问题：无法运行`make A=apps/helloworld ARCH=x86_64 run`

只能在riscv64架构上运行，请使用`ARCH=riscv64`

> 问题：尝试运行，然后报错：
>
> ```
> rust-objcopy --binary-architecture=riscv64 apps/helloworld/helloworld_riscv64-qemu-virt.elf --strip-all -O binary apps/helloworld/helloworld_riscv64-qemu-virt.bin
> make: rust-objcopy: No such file or directory
> make: *** [scripts/make/build.mk:46: apps/helloworld/helloworld_riscv64-qemu-virt.bin] Error 127
> ```

先`cargo install cargo-binutils`，再`rustup component add llvm-tools-preview`。如果出现其他报错，按照rCore_dev_guide处理。

> 问题：找不到块设备。

先运行`./build_img.sh -a riscv64`，再在`Makefile`中设置`BLK ?= y`，最后检查`DISK_IMG`的默认值是否为`disk.img`。或者直接在`make`指令中指定`BLK=y`也成。

> 问题：默认在Unikernel模式下，不知如何进入宏内核模式？

就此请教赵前辈，得到的信息以及验证情况如下：

- [x] 目前，只有`apps/user_boot`这个App能进入用户态。进入后，将通过文件系统读取用户程序的`ELF`文件，然后加载运行。
- [x] 运行上述App的`make`指令为`make A=apps/user_boot ARCH=riscv64 LOG=off SMP=1 FEATURES=sched_fifo,img  BLK=y run`。
- [ ] 想要令某个app使用用户态，则必须在其依赖项中加入`features = [ "monolithic" ]`
- [ ] 若要自行编写用户态App，需要遵循以下步骤：
  - [x] 参考rCore的用户态程序编写用户态代码，然后编译获得ELF文件。此时，工具链用 `riscv...unknown...elf` 的，并且可用Linux下的标准调用语法（个人理解：借助`libc` crate，将我们的操作翻译为系统调用）
  - [x] 将编译获得的ELF文件放在 `testcases/riscv64_linux_musl` 目录下
  - [x] 重新编译一次 `disk.img`

按ZFL前辈的说法，rCore那里编写的用户态程序是可以直接在async-os的用户态中运行的。于是，我们采取这样的做法：

1. 在rCore的 `user/` 目录下运行 `make build` 后，在 `user/target/riscv64gc-unknown-none-elf/release` 中找到了编译成ELF文件的 `hello_world` 。将其拷贝到 `testcases/riscv64_linux_musl` 目录下后，重新编译 `disk.img` （通过运行“找不到块设备”中的命令），执行上述 `make` 指令，没变化欸？？？
2. 观察 `apps/user_boot/src/main.rs` 的内容，发现它将测例的名字 `hello` 硬编码到代码中了，于是改名成 `hello_world` 再次编译，发现还是没变化欸？？？
3. 将日志等级开到 `LOG=info` 再次编译，发现报告了错误：
   ```no_run
   panicked at /home/endericedragon/repos/async-os/modules/trampoline/src/task_api.rs:139:21:
   Unhandled trap Exception(LoadPageFault) @ 0x100a8:
   TrapFrame { ... }
   ```
   似乎是在用户态出现了未能处理的异常。

排查工作至此卡住，询问ZFL前辈后得知两个内核向用户态传输参数的方式不同。只需将 `modules/taskctx/src/arch/riscv/mod.rs` 中的以下代码注释掉即可：

```rust
impl TrapFrame {
    /// 用于创建用户态任务的初始化
    pub fn init_user_context(app_entry: usize, user_sp: usize) -> Self {
        // -- snip --
        unsafe {
            // a0为参数个数
            // a1存储的是用户栈底，即argv
            trap_frame.regs.a0 = *(user_sp as *const usize);
            trap_frame.regs.a1 = *(user_sp as *const usize).add(1);
        }
        // -- snip --
    }
}
```

虽然问题暂时解决了，但是深层次的问题没有解决：

1. ZFL前辈是如何定位到这个问题的？
2. 有办法让rCore的用户程序编译完可以直接被async-os加载运行吗？

要解决第一个问题，赵前辈提出了一系列技术方法：

- 调试：GDB
- 反汇编：rust-objdump
- 模拟器日志：qemu log，搜到了一篇 [QEMU虚拟机日志调试](https://www.baeldung.com/linux/qemu-vm-logging-debugging) 博文，在async-os中可以通过 `QEMU_LOG=y` 启用
- 跟踪系统调用：strace

### 尝试自主定位问题

首先尝试采用阅读QEMU日志的方法进行故障排查。指定 `QEMU_LOG=y` 后再运行一次 `user_boot` ，然后查看 `qemu.log` 文件。由于我们从报错信息得知错误类型为 `LoadPageFault`，且出现错误的代码位于 `0x100a8`，因此可以搜索 `0x100a8` 相关的日志信息。

经过搜索，找到相关信息如下：

```
----------------
IN:
Priv: 0; Virt: 0
...
0x00000000000100a8:  00154503          lbu             a0,1(a0)  # <- Here
0x00000000000100ac:  0605              addi            a2,a2,1
0x00000000000100ae:  f97d              bnez            a0,-10          # 0x100a4

riscv_cpu_tlb_fill ad 6f775f6f6c6c6568 rw 0 mmu_idx 0
riscv_cpu_tlb_fill address=6f775f6f6c6c6568 ret 1 physical 0000000000000000 prot 0
riscv_cpu_do_interrupt: hart:0, async:0, cause:000000000000000d, epc:0x00000000000100a8, tval:0x6f775f6f6c6c6568, desc=load_page_fault
----------------
```

可以看到，`0x100a8` 处是一个 `LBU` 指令，其功能解释为，从内存地址 `a0 + 1` 中读取一个字节，然后放到 `a0` 寄存器中去。显然，`a0` 指向的内存地址有问题。

Fitten Code提示说，此时让反汇编介入可获得更多信息，于是尝试对 ELF 文件进行反汇编：

```sh
rust-objdump --disassemble-all testcases/riscv64_linux_musl/hello_world > disassemble.txt
```

然后再搜索 `100a8` ，获得的结果和QEMU LOG中的类似，一条LBU指令。

那么，我们可以还原出错前一小段时间内CPU里发生的事情：

```asm
1009e: 52 95        	add	a0, a0, s4  # 给a0加上s4的值
100a0: 0c 61        	ld	a1, 0x0(a0) # 将内存地址a0中的值放到a1中，注意到此时读内存是正常的
100a2: 7d 56        	li	a2, -0x1    # 给a2赋值为-1
100a4: 33 85 c5 00  	add	a0, a1, a2  # 给a0加上a1和a2的和
# 综合上述过程，其实相当于：a0 = *(a0 + 0) - 1
100a8: 03 45 15 00  	lbu	a0, 0x1(a0) # 炸了！
```

我们已经搞明白了出问题的指令是什么，但这个指令是谁发出的？在源代码中又体现在哪里呢？我们需要借助GDB一探究竟。

### GDB调试

书接上回，我们利用GDB尝试进行调试（找了篇[教程](https://www.cnblogs.com/lvdongjie/p/8994092.html)熟悉了一下常用命令）。调试需要二进制文件中具有调试信息，我们做如下更改：
- `scripts/make/build.mk` 中的 `_cargo_build` 命令，将 `--strip-all` 删除
- 调试时，使用如下指令： `make A=apps/user_boot ARCH=riscv64 MODE=debug LOG=info SMP=1 FEATURES=sched_fifo,img BLK=y mydebug`

其中的 `mydebug` 命令如下定义：

```makefile
mydebug: build
	$(call run_qemu_debug) &
	sleep 1
	$(GDB) $(OUT_ELF) \
	  -ex 'target remote localhost:1234'
```

## 代码阅读

### 异步的Mutex实现探究

既然目前的问题聚焦于`Mutex`之上，那我们不如就抓着`Mutex`这一条线，采取纵向阅读的方法，探究这个异步的`Mutex`到底是如何实现的呗？

我们知道这个`Mutex`来自`modules/sync/src/mutex.rs`，我们的阅读就从这里开始吧。

## 增添功能

### 新建模块async_std::collections

注意到当前操作系统没有提供容器数据结构，仅有`alloc::vec::Vec`，因此有必要增添`async_std::collections`来提供常用的容器数据结构。

本次功能增添一共新增两个容器：`HashMap`和`BinaryHeap`。前者直接通过引入`hashbrown`库实现，后者由笔者手动编写实现。
