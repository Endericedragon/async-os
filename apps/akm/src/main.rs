#![no_std]
#![no_main]

use async_std::prelude::{Read, Write};

#[macro_use]
extern crate async_std;
extern crate async_net;

use async_collections::{vec, Vec};

mod peer_id_emulate;

use alloc::string::ToString;
use async_net::multistream_select::{dialer_select, Version};
use async_net::TcpSocket;

#[async_std::async_main]
async fn main() -> isize {
    println!("Greetings!");

    // let mut tcp_socket = TcpSocket::new();
    // // tcp_socket.connect("127.0.0.1:7878").await;
    // dialer_select(
    //     tcp_socket,
    //     vec!["p1".to_string(), "proto2".to_string()],
    //     Version::V1,
    // )
    // .await
    // .await
    // .expect("Failed to select protocol!");

    0
}

async fn web_server() {
    let ip_port_pair = "0.0.0.0:7878";

    let listener = async_std::net::TcpListener::bind(ip_port_pair)
        .await
        .expect("Failed to bind!");
    println!("Listening on {}...", ip_port_pair);

    loop {
        let (mut tcp_stream, socket_addr) = listener.accept().await.expect("Failed to accept!");
        let mut buf = [0u8; 1024];
        println!("Received connection from {}...", socket_addr);
        tcp_stream.read(&mut buf).await.expect("Failed to read!");
        println!("Received: {}", alloc::string::String::from_utf8_lossy(&buf));
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        tcp_stream
            .write_all(response.as_bytes())
            .await
            .expect("Failed to write!");
    }
}
