# async-os学习研究笔记

## 问题汇总

> 问题：无法运行`make A=apps/helloworld ARCH=x86_64 run`

回答：只能在riscv64架构上运行，请使用`ARCH=riscv64`

> 问题：尝试运行，然后报错：
>
> ```
> rust-objcopy --binary-architecture=riscv64 apps/helloworld/helloworld_riscv64-qemu-virt.elf --strip-all -O binary apps/helloworld/helloworld_riscv64-qemu-virt.bin
> make: rust-objcopy: No such file or directory
> make: *** [scripts/make/build.mk:46: apps/helloworld/helloworld_riscv64-qemu-virt.bin] Error 127
> ```

回答：先`cargo install cargo-binutils`，再`rustup component add llvm-tools-preview`。如果出现其他报错，按照rCore_dev_guide处理。

> 问题：找不到块设备。

回答：先运行`./build_img.sh -a riscv64`，再在`Makefile`中设置`BLK ?= y`，最后检查`DISK_IMG`的默认值是否为`disk.img`。或者直接在`make`指令中指定`BLK=y`也成。

## 代码阅读

### 异步的Mutex实现探究

既然目前的问题聚焦于`Mutex`之上，那我们不如就抓着`Mutex`这一条线，采取纵向阅读的方法，探究这个异步的`Mutex`到底是如何实现的呗？

我们知道这个`Mutex`来自`modules/sync/src/mutex.rs`，我们的阅读就从这里开始吧。

## 增添功能

### 新建模块async_std::collections

注意到当前操作系统没有提供容器数据结构，仅有`alloc::vec::Vec`，因此有必要增添`async_std::collections`来提供常用的容器数据结构。

本次功能增添一共新增两个容器：`HashMap`和`BinaryHeap`。前者直接通过引入`hashbrown`库实现，后者

### 尝试移植futures-bounded#0.2.3

这个库在rust-libp2p的依赖图拓扑排序中排在非常靠前的位置，因此先从它开始移植。
