/// The nano seconds number per second
pub const NSEC_PER_SEC: usize = 1_000_000_000;
pub const NANOS_PER_SEC: u64 = 1000000000;
pub const NANOS_PER_MICROS: u64 = 1000;
pub const MICROS_PER_SEC: u64 = 1000000;
pub const AXCONFIG_TIMER_FREQUENCY: usize = 0;

/// sys_times 中指定的结构体类型
#[derive(Debug)]
#[repr(C)]
pub struct Tms {
    /// 进程用户态执行时间，单位为us
    pub tms_utime: usize,
    /// 进程内核态执行时间，单位为us
    pub tms_stime: usize,
    /// 子进程用户态执行时间和，单位为us
    pub tms_cutime: usize,
    /// 子进程内核态执行时间和，单位为us
    pub tms_cstime: usize,
}

impl Default for Tms {
    fn default() -> Self {
        Self {
            tms_utime: Default::default(),
            tms_stime: Default::default(),
            tms_cutime: Default::default(),
            tms_cstime: Default::default(),
        }
    }
}

/// sys_gettimeofday 中指定的类型
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TimeVal {
    /// seconds
    pub sec: usize,
    /// microseconds
    pub usec: usize,
}

impl Default for  TimeVal {
    fn default() -> Self {
        Self { sec: Default::default(), usec: Default::default() }
    }
}

impl TimeVal {
    /// turn the TimeVal to nano seconds
    pub fn turn_to_nanos(&self) -> usize {
        self.sec * NANOS_PER_SEC as usize + self.usec * NANOS_PER_MICROS as usize
    }

    /// create a TimeVal from nano seconds
    pub fn from_micro(micro: usize) -> Self {
        TimeVal {
            sec: micro / (MICROS_PER_SEC as usize),
            usec: micro % (MICROS_PER_SEC as usize),
        }
    }

    /// turn the TimeVal to cpu ticks, which is related to cpu frequency
    pub fn turn_to_ticks(&self) -> u64 {
        (self.sec * AXCONFIG_TIMER_FREQUENCY) as u64
            + (self.usec as u64) * NANOS_PER_MICROS
    }
}