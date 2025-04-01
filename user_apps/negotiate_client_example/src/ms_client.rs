use core::arch::asm;
use ms_shared_types::ps_codec::Encode;
use ms_shared_types::NegotiationContext;
use std::ffi::c_void;

const MULTISTREAM_SELECT_DIALER: usize = 42666;

pub struct Negotiator {
    protocols: Vec<String>,
    pub selected_proto_idx: Option<usize>,
    remote_fd: Option<usize>,
}

impl Negotiator {
    /// 简单的初始化
    pub fn new() -> Self {
        Self {
            protocols: Vec::new(),
            selected_proto_idx: None,
            remote_fd: None,
        }
    }
    /// 给自身添加一个协议，表示自身支持该协议
    pub fn add_protocol(mut self, proto: &str) -> Self {
        self.protocols.push(proto.to_string());
        self
    }
    /// 尝试与远程主机建立连接并协商后续协议
    /// 返回bool值指示协商是否成功
    pub fn dial(&mut self, addr: [u8; 4], port: u16) -> bool {
        let mut _ret: isize;
        let neg_ctxt = NegotiationContext {
            addr,
            port,
            protocols: self.protocols.clone(),
        };
        let mut encoded_ptr = neg_ctxt.encode();
        let mut fd: i64;
        let proto_idx = -1i64;
        let encoded_sess_ptr = encoded_ptr.as_mut_ptr();
        unsafe {
            asm!(
                "ecall",
                in("a7") MULTISTREAM_SELECT_DIALER, // 暂定42666
                inlateout("a0") encoded_ptr.len() => fd,
                in("a1") encoded_sess_ptr,
                in("a2") &proto_idx as *const i64 as *mut i64,
            )
        }
        // let new_vec = unsafe {
        //     Vec::from_raw_parts(encoded_sess_ptr, length, length)
        // };
        if fd >= 0 && proto_idx >= 0 {
            println!("Negotiation successful!");
            self.remote_fd = Some(fd as usize);
            self.selected_proto_idx = Some(proto_idx as usize);
            true
        } else {
            false
        }
    }
    /// 使用协商好的协议，向远程主机发送数据，返回已发送的字节数
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

    /// 使用协商好的协议，从远程主机接收数据，返回已接收的字节数
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
