use std::io::{ErrorKind, Read as _};
use std::net::TcpStream;

use serde::{Deserialize, Serialize};

use crate::known_peer_item::KnownPeerItem;
use crate::peer_id_wrapper::PeerIdWrapper;

#[derive(Serialize, Deserialize, Debug)]
pub enum LiaisonMessage {
    Text {
        sender: PeerIdWrapper,
        text_message: String,
    },
    HubReturnsKnownPeers {
        sender: PeerIdWrapper,
        known_peers: Vec<KnownPeerItem>,
    },
    HubDeclaresExpiredPeer(PeerIdWrapper),
    NormalRequestsKnownPeers {
        sender: PeerIdWrapper,
    },
    NormalRequestsBroadcast {
        sender: PeerIdWrapper,
        message: String,
    },
}

impl LiaisonMessage {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(&data)
    }

    pub fn read_from_tcp_stream(
        stream: &mut TcpStream,
        buf: &mut [u8],
    ) -> Result<Self, std::io::Error> {
        match stream.read(buf) {
            Ok(0) => return Err(ErrorKind::UnexpectedEof.into()),
            Ok(n) => return Ok(Self::deserialize(&buf[0..n]).unwrap()),
            Err(e) => return Err(e),
        }
    }
}
