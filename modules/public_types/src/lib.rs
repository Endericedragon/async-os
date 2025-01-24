//! 这个模块中存储了一些Linux中使用的、和网络有关的结构体，例如 `socketaddr_nl` 等。
//! 在该模块中，它们被进行了重命名以符合Rust推荐的命名规范，例如 `socketaddr_nl` => `LibcSockAddrNl` 。
//! 除此之外，该模块还支持从 `*const u8` 中恢复这些结构体。

/// Structure describing a generic socket address in libc.
#[allow(unused)]
#[repr(C)]
pub struct LibcSocketAddr {
    common: u16,
    sa_data: [u8; 14],
}

/// libc中sockaddr_nl的定义
#[allow(unused)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LibcSockAddrNl {
    pub nl_family: u16,
    pub nl_pad: u16,
    pub nl_pid: u32,
    pub nl_groups: u32,
}

impl From<*const u8> for LibcSockAddrNl {
    /*
    pub struct sockaddr_nl {
        pub nl_family: u16,
        nl_pad: u16,
        pub nl_pid: u32,
        pub nl_groups: u32,
    }
    */
    fn from(value: *const u8) -> Self {
        //? 存疑：Linux中是直接返回了 (struct sockaddr_nl*)value 的
        let val = value as *const u16;
        Self {
            nl_family: u16::from_le(unsafe { *val }),
            nl_pad: u16::from_le(unsafe { *(val.add(1)) }),
            nl_pid: u32::from_le(unsafe { *(val.add(2) as *const u32) }),
            nl_groups: u32::from_le(unsafe { *(val.add(4) as *const u32) }),
        }
    }
}

/// libc中sockaddr_in的定义
#[allow(unused)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LibcSockAddrIn {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: InAddr, // a struct with a member of u32
    pub sin_zero: [u8; 8],
}

impl From<*const u8> for LibcSockAddrIn {
    fn from(value: *const u8) -> Self {
        let ptr = value as *const u16;
        Self {
            sin_family: u16::from_be(unsafe { *ptr.add(0) }),
            sin_port: u16::from_be(unsafe { *ptr.add(1) }),
            sin_addr: InAddr {
                s_addr: unsafe { *(ptr.add(2) as *const u32) },
            },
            sin_zero: [0; 8],
        }
    }
}

/// libc中in_addr的定义，用于sockaddr_in的sin_addr成员
#[allow(unused)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InAddr {
    pub s_addr: u32,
}

impl InAddr {
    pub fn to_ipv4_format(&self) -> [u8; 4] {
        self.s_addr.to_le_bytes()
    }
}
