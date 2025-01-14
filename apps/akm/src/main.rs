#![no_std]
#![no_main]

use async_std::prelude::{Read, Write};

#[macro_use]
extern crate async_std;

mod peer_id_emulate;

#[async_std::async_main]
async fn main() -> isize {
    println!("Greetings!");

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

    0
}
