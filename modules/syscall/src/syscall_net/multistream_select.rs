use super::socket::Socket;
use alloc::string::String;
use async_collections::Vec;
use ms_shared_types::{
    kernel_exclusive::{Message, MULTISTREAM_SELECT_HEADER},
    ps_codec::{Decode, Encode},
};

pub async fn dial(sock: &Socket, protos: &Vec<String>) -> isize {
    let mut res = -1isize;
    let mut buf = [0u8; 2048];
    // send the header to tell the listener that the negotiation process begins
    error!("Sending handshake message...");
    sock.sendto(
        Message::Protocol(String::from(MULTISTREAM_SELECT_HEADER))
            .encode()
            .as_slice(),
        None,
    )
    .await
    .unwrap();
    let length = match sock.recv_from(&mut buf).await {
        Ok((length, _)) => {
            assert!(length > 0);
            let msg = Message::decode(&mut &buf[..length]).unwrap();
            error!("Received handshake message: {:?}", msg);
            length
        }
        Err(e) => {
            error!("Failed to receive handshake message: {}", e);
            return -1;
        }
    };
    match Message::decode(&mut &buf[..length]) {
        Ok(Message::NA) => {
            error!("The listener does not support multistream-select protocol???");
            return -1;
        }
        Ok(Message::Protocol(proto)) => {
            // do nothing
            if proto != MULTISTREAM_SELECT_HEADER {
                error!("The listener does not return valid handshake message???");
                return -1;
            }
        }
        Err(e) => {
            error!("Failed to decode handshake message: {}", e);
            return -1;
        }
    }
    for (idx, proto) in protos.iter().enumerate() {
        error!("Suggesting {}", proto);
        sock.sendto(
            Message::Protocol(String::from(proto)).encode().as_slice(),
            None,
        )
        .await
        .unwrap();

        let (length, _) = sock.recv_from(&mut buf).await.unwrap();
        assert!(length > 0);
        match Message::decode(&mut &buf[..]) {
            Ok(Message::NA) => {
                error!("We do not support this protocol...");
                continue;
            }
            Ok(Message::Protocol(_)) => {
                res = idx as isize;
                error!("We support this protocol!");
                break;
            }
            Err(_) => {
                error!("Failed to decode message!");
                return -1;
            }
        }
    }

    res
}
