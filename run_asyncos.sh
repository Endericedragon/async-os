if [ $# -eq 0  ]; then
    echo "Running user_boot..."
    make A=apps/user_boot ARCH=riscv64 LOG=off SMP=1 FEATURES=sched_fifo,img BLK=y run
elif [ $# -eq 1 ]; then
    echo "Running $1 with default arguments"
    make A=apps/$1 ARCH=riscv64 LOG=off SMP=1 FEATURES=sched_fifo,img BLK=y run
fi
