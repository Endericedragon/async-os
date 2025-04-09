use std::net::{ TcpListener, SocketAddr};

const MAGIC_PORT: u16 = 42666;

fn main() {
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], MAGIC_PORT))).expect("Failed to bind listener");

    while let Ok((_stream, peer_addr)) = listener.accept() {
        // 处理连接
        println!("New connection from {}", peer_addr);
    }

    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();
    //     println!("New connection from {}", stream.peer_addr().unwrap());
    // }
}
