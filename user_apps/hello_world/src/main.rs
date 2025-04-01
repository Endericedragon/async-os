use std::net::{SocketAddr, UdpSocket};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // let sock = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 42665))).unwrap();
    let sock = UdpSocket::bind(SocketAddr::from(([192, 168, 124, 4], 42665))).unwrap();
    sock.set_broadcast(true).unwrap();
    // sock.set_nonblocking(true).unwrap();
    loop {
        let msg = "Hello, world!";
        match sock.send_to(msg.as_bytes(), "255.255.255.255:42666") {
            Ok(n) => {
                println!("Sent {} bytes", n);
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
        sleep(Duration::from_secs(2));
    }
}
