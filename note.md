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

