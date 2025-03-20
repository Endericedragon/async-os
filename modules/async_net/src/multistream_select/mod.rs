extern crate alloc;
use crate::TcpSocket;
use alloc::string::String;
use async_collections::Vec;
use core::net::{SocketAddr, SocketAddrV4, Ipv4Addr};

// https://github.com/multiformats/multistream-select

struct Negotiator {
    protos: Vec<String>,
}

impl Negotiator {
    pub fn new() -> Self {
        Self { protos: Vec::new() }
    }

    pub fn add_proto(&mut self, proto_name: String) {
        self.protos.push(proto_name);
    }

    pub async fn dial(&mut self, sock_addr: SocketAddr) -> Option<usize> {
        // let _ = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        let mut num_proto = self.protos.len();
        let sock = TcpSocket::new();
        sock.connect(sock_addr).await.unwrap();
        todo!()
    }
}
