#![no_std]
#![no_main]

#[macro_use]
extern crate async_std;
extern crate async_net;

#[async_std::async_main]
async fn main() -> isize {
    let udp_socket = async_std::net::UdpSocket::bind("0.0.0.0:42660")
        .await
        .unwrap();
    match udp_socket.peer_addr() {
        Ok(peer_addr) => println!("Bound to {:?}", peer_addr),
        Err(e) => println!("Error: {:?}", e),
    }

    loop {
        if let Err(e) = udp_socket.send_to(b"hello", "255.255.255.255:42666").await {
            println!("{:?}", e);
            return -1;
        }
        async_std::task::sleep(async_std::time::Duration::from_secs(2)).await;
    }
}
