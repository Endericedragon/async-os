# User Apps

该目录下存储从rCore搬过来的用户态程序开发环境。

Async OS中并不存在原生的用户态程序开发环境。若想编写用户态程序，需要前往rCore的仓库中，编译获得用户程序的elf文件，再拷贝到Async OS的 `testcases/riscv64-linux-musl` 目录下，并重新编译磁盘镜像，才能工作，步骤繁琐，不利于开发。

笔者将rCore中的用户态程序开发环境移到此处，省去在两个开发环境中来回切换的麻烦。供读者参考。

## 目录结构

该目录存储的crate为lib型，即rust库。大体内容和rCore中的类似，这里说一下需要注意的部分：

- Async OS的`Cargo.toml`，应当将该crate排除在外，即列入`[workspace.exclude]`中。否则，每次编译用户程序，Cargo都会尝试用Async OS的编译配置覆盖该库的，非常烦。
- Makefile中增加了几条自定义目标，食用方法将在下一节介绍。
- 如果编译完用户程序，却发现Async OS的行为没有变化，请尝试先 `cargo clean` 一下，再重新编译Async OS
- 编译出的ELF文件文件名必须要和 `apps/user_boot/src/main.rs` 中 main 函数中的数组的第一个元素名字相同，否则无法被加载：
  ```rust
  let task = trampoline::init_user(vec![String::from("<ELF文件的文件名>")], &get_envs())
        .await
        .unwrap();
  ```

## 食用方法

### 编译用户程序

首先，在 `user_apps/bin` 目录下编写用户程序，假设我们编写了 `test.rs` 程序。然后，在 `user_apps` 目录下，依次运行如下make命令：

- `make my_process`：这将编译用户程序为ELF文件，不删除debug用的符号表等信息（可能在使用GDB调试时能提供一些方便），将这些ELF文件拷贝到 `testcases/riscv64-linux-musl` 目录下 ，紧接着制作磁盘镜像 `disk.img`。

返回上一级目录（即`async-os`目录），依次运行如下命令：

- `cargo clean`：清理编译环境，删除上次编译的结果。
- `make A=apps/user_boot ARCH=riscv64 LOG=info SMP=1 FEATURES=sched_fifo,img  BLK=y run`：编译Async OS，并以宏内核模式开始运行。