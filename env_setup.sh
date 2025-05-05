#!/usr/bin/sh

# 安装RISC-V交叉编译工具链
sudo apt install libc6-riscv64-cross
# 安装RISC-V的GNU工具链
sudo apt install binutils-riscv64-linux-gnu gcc-riscv64-linux-gnu
# 安装裸机环境工具链
sudo apt install binutils-riscv64-unknown-elf gcc-riscv64-unknown-elf
# 下载QEMU源代码，编译并安装
wget https://download.qemu.org/qemu-7.0.0.tar.xz
tar xvJf qemu-7.0.0.tar.xz
cd qemu-7.0.0
./configure --target-list=riscv64-softmmu,riscv64-linux-user
make -j $(nproc)
# 配置环境变量
echo export PATH=$PATH:~/qemu-7.0.0/build >> ~/.bashrc
source ~/.bashrc