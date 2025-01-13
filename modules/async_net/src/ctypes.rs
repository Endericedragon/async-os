/// libc中sockaddr_nl的定义
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct sockaddr_nl {
    pub nl_family: u16,
    pub nl_pad: u16,
    pub nl_pid: u32,
    pub nl_groups: u32,
}
