use std::{error::Error, time::Duration};

use futures::stream::StreamExt;
use libp2p::{
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use peer_id_wrapper::PeerIdWrapper;
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;

mod simple_p2p;
use simple_p2p::SimpleP2PBehaviour;

mod known_peer_item;
mod p2p_message;
mod peer_id_wrapper;

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    spb: SimpleP2PBehaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    // swarm是 libp2p 中表示一个Peer的核心结构，负责管理节点的 网络连接、协议处理、事件循环，以及与其他节点的交互。
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let local_peer_id = key.public().to_peer_id();
            println!("Local peer id = {:?}", local_peer_id);
            let spb = simple_p2p::SimpleP2PBehaviour::new(local_peer_id);
            Ok(MyBehaviour { spb })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    let local_peer_id_wrapper = PeerIdWrapper::from(swarm.local_peer_id());
    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    println!("Enter messages via STDIN and they will be sent to connected peers using SimpleP2P.");

    // Kick it off
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                let line = line.trim();
                swarm.behaviour_mut().spb.broadcast_string(
                    local_peer_id_wrapper.as_inner(),
                    String::from(line)
                );
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Spb(simple_p2p::P2PEvent::UpdatedKnownPeers(peer_ids))) => {
                    for each in peer_ids {
                        println!("SimpleP2P updated a new peer: {}", each);
                    }
                }
                SwarmEvent::Behaviour(MyBehaviourEvent::Spb(simple_p2p::P2PEvent::IncomingMessage(remote_peer, new_msg_id, new_msg))) => {
                    println!(
                        "Got message: '{}' with {} from peer: {}",
                        new_msg,
                        new_msg_id,
                        remote_peer,
                    );
                }
                SwarmEvent::Behaviour(MyBehaviourEvent::Spb(simple_p2p::P2PEvent::ErrorAndExit)) => {
                    eprintln!("Fatal error occured and exit.");
                    break Ok(())
                }
                SwarmEvent::Behaviour(MyBehaviourEvent::Spb(simple_p2p::P2PEvent::PeerExpired(expired_peer_id))) => {
                    println!("Peer {} expired!", expired_peer_id);
                }
                _ => {}
            }
        }
    }
}

// mod mailbox;

// fn main() {
//     let mut my_mailbox = mailbox::Mailbox::new("AsyncOS".to_string());
//     loop {
//         println!("Sending...");
//         my_mailbox.prepare_message(b"Hello, world!");
//         my_mailbox.broadcast_buf();
//         std::thread::sleep(std::time::Duration::from_secs(2));
//     }
// }
