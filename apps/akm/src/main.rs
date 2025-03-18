#![no_std]
#![no_main]

#[macro_use]
extern crate async_std;
extern crate async_net;

use alloc::string::String;
use async_collections::Vec;
use async_std::net::{SocketAddr, TcpStream};

#[async_std::async_main]
async fn main() -> isize {
    0
}

struct Negotiator {
    protos: Vec<String>,
    stream: Option<TcpStream>,
}

impl Negotiator {
    pub fn new() -> Self {
        Self {
            protos: Vec::new(),
            stream: None,
        }
    }

    pub fn add_proto(&mut self, proto_name: String) {
        self.protos.push(proto_name);
    }

    pub fn dial(&mut self, sock_addr: SocketAddr) {}
}
