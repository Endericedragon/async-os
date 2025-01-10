app="user_boot"
log_level="off"

while [ $# -gt 0 ]; do
    case "$1" in
        --app)
            app="$2"
            shift 2 # 吃掉两个命令行参数
            ;;
        --log)
            log_level="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

if [ "$log_level" != ""  ]; then
    echo "Running $app with log level: $log_level"
else
    echo "Running $app with default arguments"
fi

make A=apps/"$app" ARCH=riscv64 LOG="$log_level" SMP=1 FEATURES=sched_fifo,img BLK=y NET=y run