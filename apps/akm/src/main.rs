#![no_std]
#![no_main]

#[macro_use]
extern crate async_std;
extern crate async_net;

// use async_net::multistream_select;
use async_std::net::SocketAddr;

#[async_std::async_main]
async fn main() -> isize {
    let udp_socket = async_std::net::UdpSocket::bind("0.0.0.0:0").await.unwrap();
    loop {
        udp_socket
            .send_to(b"hello", SocketAddr::from(([255, 255, 255, 255], 42666)))
            .await
            .unwrap();
        async_std::task::sleep(async_std::time::Duration::from_secs(1)).await;
    }
    0
}
