use core::arch::asm;
use parity_scale_codec::{Decode, Encode};
use std::ffi::c_void;

const MULTISTREAM_SELECT_DIALER: usize = 42666;

#[derive(Encode, Decode)]
struct ProtocolList(Vec<String>);

pub struct Negotiator {
    protocols: ProtocolList,
    selected_proto: Option<String>,
    remote_fd: Option<usize>,
}

impl Negotiator {
    /// 简单的初始化
    pub fn new() -> Self {
        Self {
            protocols: ProtocolList(Vec::new()),
            selected_proto: None,
            remote_fd: None,
        }
    }
    /// 给自身添加一个协议，表示自身支持该协议
    pub fn add_protocol(&mut self, proto: &str) {
        self.protocols.0.push(proto.to_string());
    }
    /// 尝试与远程主机建立连接，返回bool值指示协商是否成功
    pub fn dial(&mut self, addr: [u8; 4], port: u16) -> bool {
        // todo 实现这个系统调用
        let mut fd: isize;
        let mut proto_idx: isize;
        let encoded_protocol_list = self.protocols.encode();
        unsafe {
            asm!(
                "ecall",
                inlateout("a7") MULTISTREAM_SELECT_DIALER => fd, // 暂定42666
                inlateout("a0") addr.as_ptr() => proto_idx,
                in("a1") port,
                in("a2") encoded_protocol_list.as_ptr(),
                in("a3") encoded_protocol_list.len(),
            )
        }
        if fd >= 0 {
            self.selected_proto = Some(self.protocols.0[proto_idx as usize].clone());
            self.remote_fd = Some(fd as usize);
            true
        } else {
            false
        }
    }
    /// 使用协商好的数据，向远程主机发送数据，返回已发送的字节数
    pub fn send(&self, buf: &[u8]) -> isize {
        if let Some(remote_fd) = self.remote_fd {
            unsafe {
                libc::send(
                    remote_fd as i32,
                    buf.as_ptr() as *const c_void,
                    buf.len(),
                    0,
                ) as isize
            }
        } else {
            -1
        }
    }

    /// 使用协商好的数据，从远程主机接收数据，返回已接收的字节数
    pub fn recv(&self, buf: &mut [u8]) -> isize {
        if let Some(remote_fd) = self.remote_fd {
            unsafe {
                libc::recv(
                    remote_fd as i32,
                    buf.as_mut_ptr() as *mut c_void,
                    buf.len(),
                    0,
                )
            }
        } else {
            -1
        }
    }
}
