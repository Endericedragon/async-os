use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    time::Duration,
};

use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;

mod simple_discover;

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    // gossipsub: gossipsub::Behaviour,
    // mdns: mdns::tokio::Behaviour,
    sd: simple_discover::DiscoveryBehaviour,
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
            // // To content-address message, we can take the hash of message and use it as an ID.
            // let message_id_fn = |message: &gossipsub::Message| {
            //     let mut s = DefaultHasher::new();
            //     message.data.hash(&mut s);
            //     gossipsub::MessageId::from(s.finish().to_string())
            // };

            // // Set a custom gossipsub configuration.
            // let gossipsub_config = gossipsub::ConfigBuilder::default()
            //     .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            //     .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            //     .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
            //     .flood_publish(true)
            //     .build()
            //     .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

            // // build a gossipsub network behaviour. Netlink call occurs here!!
            // let gossipsub = gossipsub::Behaviour::new(
            //     gossipsub::MessageAuthenticity::Signed(key.clone()),
            //     gossipsub_config,
            // )?;
            // 是mDNS触发了Netlink调用，得想办法把它换掉
            // let mdns_config = mdns::Config::default();
            // let local_peer_id_key = key.public().to_peer_id();
            // println!("Constructing mdns...");
            // let mdns = mdns::tokio::Behaviour::new(mdns_config, local_peer_id_key)?;

            let sd = simple_discover::DiscoveryBehaviour::new(key.public().to_peer_id());

            // Ok(MyBehaviour { gossipsub, mdns })
            // Ok(MyBehaviour { gossipsub, sd })
            Ok(MyBehaviour { sd })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // // Create a Gossipsub topic
    // let topic = gossipsub::IdentTopic::new("test-net");
    // // subscribes to our topic
    // swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Listen on all interfaces and whatever port the OS assigns
    // swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?; // 会触发Netlink
    swarm.listen_on("/ip4/192.168.124.4/udp/0/quic-v1".parse()?)?; // 不会触发Netlink，而且能用
                                                                   // swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    // Kick it off
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                // if let Ok(e) = swarm
                //     .behaviour_mut().gossipsub
                //     .publish(topic.clone(), line.as_bytes()) {
                //     println!("Publish error: {e:?}");
                // }
                swarm.behaviour_mut().sd.broadcast(&line.as_bytes());
            }
            event = swarm.select_next_some() => match event {
                // SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                //     for (peer_id, _multiaddr) in list {
                //         println!("mDNS discovered a new peer: {peer_id}");
                //         swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                //     }
                // },
                // SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                //     for (peer_id, _multiaddr) in list {
                //         println!("mDNS discover peer has expired: {peer_id}");
                //         swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                //     }
                // },
                SwarmEvent::Behaviour(MyBehaviourEvent::Sd(simple_discover::DiscoveryEvent::Discovered(new_peer_id))) => {
                    println!("simple_discover discovered a new peer: {}", new_peer_id);
                    // swarm.behaviour_mut().gossipsub.add_explicit_peer(&new_peer_id);
                    // for each in swarm.behaviour_mut().gossipsub.all_peers() {
                    //     println!("{:?}", each);
                    // }
                }
                SwarmEvent::Behaviour(MyBehaviourEvent::Sd(simple_discover::DiscoveryEvent::IncomingMessage(remote_peer, new_msg_id, new_msg))) => {
                    println!(
                        "Got message: '{}' with {} from peer: {}",
                        new_msg,
                        new_msg_id,
                        remote_peer,
                    );
                }
                // SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                //     propagation_source: peer_id,
                //     message_id: id,
                //     message,
                // })) => println!(
                //         "Got message: '{}' with id: {id} from peer: {peer_id}",
                //         String::from_utf8_lossy(&message.data),
                //     ),
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
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
