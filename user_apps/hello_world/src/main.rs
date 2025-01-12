use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").expect("Failed to bind!");

    loop {
        let Ok((mut tcp_stream, socket_addr)) = listener.accept() else {
            println!("Failed to accept connection!");
            break;
        };

        let mut buffer = [0; 1024];
        println!("Connection from: {}", socket_addr);
        if let Err(_) = tcp_stream.read(&mut buffer) {
            println!("Failed to read from stream!");
            break;
        }
        println!("Received: {}", String::from_utf8_lossy(&buffer));

        let resp = "Hello";
        if let Err(_) = tcp_stream.write_all(resp.as_bytes()) {
            println!("Failed to send response!");
            break;
        }
    }
}
