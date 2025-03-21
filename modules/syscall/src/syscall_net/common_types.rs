use alloc::string::String;
use async_collections::Vec;
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct NegotiationSession {
    pub addr: [u8; 4],
    pub port: u16,
    pub protocols: Vec<String>,
    pub result: (i64, u64),
}

#[derive(Encode, Decode, Debug)]
pub enum Message {
    NA,
    Protocol(String),
}
