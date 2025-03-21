use super::socket::Socket;
use alloc::string::String;
use async_collections::Vec;
use parity_scale_codec::{Decode, Encode};

use super::common_types::Message;

pub async fn dial(sock: &Socket, protos: &Vec<String>) -> isize {
    info!("HEHEHEHE");
    let mut res = -1isize;
    for (idx, proto) in protos.iter().enumerate() {
        sock.sendto(
            Message::Protocol(String::from(proto)).encode().as_slice(),
            None,
        )
        .await
        .unwrap();
        let mut buf = [0u8; 2048];
        sock.recv_from(&mut buf).await.unwrap();
        match Message::decode(&mut &buf[..]) {
            Ok(Message::NA) => {
                continue;
            }
            Ok(Message::Protocol(_)) => {
                res = idx as isize;
                break;
            }
            Err(_) => {
                break;
            }
        }
    }

    res
}
