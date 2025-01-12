#![no_std]
#![no_main]

use async_std::prelude::{Read, Write};

#[macro_use]
extern crate async_std;

#[async_std::async_main]
async fn main() -> isize {
    println!("Greetings!");

    let listener = async_std::net::TcpListener::bind("0.0.0.0:7878")
        .await
        .expect("Failed to bind!");
    async_std::println!("Listening on 0.0.0.0:7878");

    loop {
        let (mut tcp_stream, socket_addr) = listener.accept().await.expect("Failed to accept!");
        let mut buf = [0u8; 1024];
        async_std::println!("Received connection from {}...", socket_addr);
        tcp_stream.read(&mut buf).await.expect("Failed to read!");
        async_std::println!("Received: {}", alloc::string::String::from_utf8_lossy(&buf));
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        tcp_stream
            .write_all(response.as_bytes())
            .await
            .expect("Failed to write!");
    }

    0
}
