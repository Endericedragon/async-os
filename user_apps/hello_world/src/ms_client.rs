use core::arch::asm;
use parity_scale_codec::{Decode, Encode};
use std::ffi::c_void;

const MULTISTREAM_SELECT_DIALER: usize = 42666;

pub struct Negotiator {
    protocols: Vec<String>,
    pub selected_proto_idx: Option<usize>,
    remote_fd: Option<usize>,
}

#[derive(Encode, Decode, Debug)]
struct NegotiationSession {
    addr: [u8; 4],
    port: u16,
    protocols: Vec<String>,
    result: (i64, i64),
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
    pub fn add_protocol(&mut self, proto: &str) {
        self.protocols.push(proto.to_string());
    }
    /// 尝试与远程主机建立连接并协商后续协议
    /// 返回bool值指示协商是否成功
    pub fn dial(&mut self, addr: [u8; 4], port: u16) -> bool {
        let mut _ret: isize;
        let sess = NegotiationSession {
            addr,
            port,
            protocols: self.protocols.clone(),
            result: (i64::MIN, i64::MIN),
        };
        let encoded_ptr = sess.encode();
        let encoded_sess_ptr = encoded_ptr.as_ptr();
        unsafe {
            asm!(
                "ecall",
                in("a7") MULTISTREAM_SELECT_DIALER, // 暂定42666
                in("a0") encoded_ptr.len(),
                in("a1") encoded_sess_ptr,
            )
        }
        let negotiation_sess_after = NegotiationSession::decode(&mut &encoded_ptr[..]).unwrap();
        println!("{:#?}", negotiation_sess_after);
        match negotiation_sess_after.result {
            (fd, proto_idx) if fd >= 0 && proto_idx >= 0 => {
                self.remote_fd = Some(fd as usize);
                self.selected_proto_idx = Some(proto_idx as usize);
                true
            }
            _ => false,
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
