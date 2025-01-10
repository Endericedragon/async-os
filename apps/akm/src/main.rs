#![no_std]
#![no_main]

#[macro_use]
extern crate async_std;

// use libp2p_identity::{Keypair, PeerId};

#[async_std::async_main]
async fn main() -> isize {
    println!("Greetings!");
    // println!("Testing...");

    // let keypair = Keypair::generate_ed25519();
    // let peer_id = keypair.public().to_peer_id();
    // println!("Original PeerId: {:?}", peer_id);
    // let peer_id_bytes = peer_id.to_bytes();

    // let restored_peer_id = PeerId::from_bytes(&peer_id_bytes).unwrap();
    // println!("Restored PeerId: {:?}", restored_peer_id);

    // println!("All tests passed!");
    0
}
