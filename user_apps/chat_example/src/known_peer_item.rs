use crate::peer_id_wrapper::PeerIdWrapper;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct KnownPeerItem(pub PeerIdWrapper);
