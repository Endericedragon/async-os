use std::{
    error::Error,
    time::{Duration, Instant},
};

use futures::stream::StreamExt;
use libp2p::{
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use peer_id_wrapper::PeerIdWrapper;
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;

mod simple_liaison;
use simple_liaison::SimpleLiaisonBehaviour;

mod known_peer_item;
mod liaison_message;
mod peer_id_wrapper;

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct CustomBehaviour {
    spb: SimpleLiaisonBehaviour,
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
            let spb = simple_liaison::SimpleLiaisonBehaviour::new(local_peer_id);
            Ok(CustomBehaviour { spb })
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
                eprintln!("[Sending...] {:?}", Instant::now());
                let line = line.trim();
                swarm.behaviour_mut().spb.broadcast_string(
                    local_peer_id_wrapper.as_inner(),
                    String::from(line)
                );
                eprintln!("[Sent] {:?}!", Instant::now());
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(CustomBehaviourEvent::Spb(simple_liaison::LiaisonEvent::UpdatedKnownPeers(peer_ids))) => {
                    for each in peer_ids {
                        println!("SimpleP2P updated a new peer: {}", each);
                    }
                }
                SwarmEvent::Behaviour(CustomBehaviourEvent::Spb(simple_liaison::LiaisonEvent::IncomingMessage(remote_peer_id, new_msg_id, new_msg))) => {
                    // println!(
                    //     "Got message: '{}' with {} from peer: {}",
                    //     new_msg,
                    //     new_msg_id,
                    //     remote_peer,
                    // );
                    match new_msg.as_str() {
                        "/stress_test" => {
                            eprintln!("[Stress test starting...] {:?}", Instant::now());
                            for _ in 0..4096 {
                                swarm.behaviour_mut().spb.broadcast_string(
                                    local_peer_id_wrapper.as_inner(),
                                    format!("[S] Stress[{:?}]", Instant::now())
                                );
                            }
                            eprintln!("[Stress test done!] {:?}", Instant::now());
                        }
                        "/resp" => {
                            eprintln!("[Responding...] {:?}", Instant::now());
                            swarm.behaviour_mut().spb.broadcast_string(
                                local_peer_id_wrapper.as_inner(),
                                format!("[R] {:?}", Instant::now())
                            );
                            eprintln!("[Responded!] {:?}", Instant::now());
                        }
                        x if x.starts_with("[R]") => {
                            eprintln!("[Got response] {:?}", Instant::now());
                        }
                        x if x.starts_with("[S]") => {
                            eprintln!("[Got stress message] {:?}", Instant::now());
                        }
                        _ => {
                            // 只有普通消息才会打印出来
                            println!(
                                "Got message: '{}' with id: {} from peer: {} at {:?}",
                                new_msg, new_msg_id, remote_peer_id, Instant::now(),
                            );
                        }
                    }
                }
                SwarmEvent::Behaviour(CustomBehaviourEvent::Spb(simple_liaison::LiaisonEvent::ErrorAndExit)) => {
                    eprintln!("Fatal error occured and exit.");
                    break Ok(())
                }
                SwarmEvent::Behaviour(CustomBehaviourEvent::Spb(simple_liaison::LiaisonEvent::PeerExpired(expired_peer_id))) => {
                    println!("Peer {} expired!", expired_peer_id);
                }
                _ => {}
            }
        }
    }
}
