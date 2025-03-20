#![no_std]
#![no_main]

#[macro_use]
extern crate async_std;
extern crate async_net;

use alloc::string::String;
use async_collections::Vec;
use async_net::multistream_select;
use async_std::net::SocketAddr;
use core::arch::asm;
const SYS_MULTISTREAM_DIAL: u16 = 42667;

#[async_std::async_main]
async fn main() -> isize {
    0
}


