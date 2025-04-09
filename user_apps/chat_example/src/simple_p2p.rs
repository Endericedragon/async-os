//! 简单的节点发现机制，基于TCP配合Known Peers机制实现。
//! 规定：每个P2P网络中总有一个节点充当枢纽Hub的作用。该Hub总是工作在42666端口上。
//!
//! 当新节点启动时，应当这样做：
//! - 尝试绑定到42666端口。若成功，则认为自己是Hub节点，使用 TcpListern 开始监听42666端口。
//! - 若绑定失败，则认为自己仅仅是普通节点，并建立一条通往Hub的 TcpStream 连接。
//!
//! 当节点需要更新 known_peers 时，应当这样做：
//! - Hub 不会主动更新 known_peers 。因为每个节点启动后都要连接到 Hub ，Hub 可以借此机会得知局域网内所有节点的信息。
//! - 普通节点会主动更新 known_peers 。它会每隔一段时间像 Hub 请求更新known_peers ，目前的实现是只增不减。
//!
//! 当节点需要广播信息时，应当这样做：
//! - Hub 自己就可以广播信息。同时，它接受来自普通节点的广播请求，并帮助它们广播消息到除它们以外的其他节点。
//! - 普通节点会向 Hub 发送广播请求，Hub 会将消息广播到除它们以外的其他节点。

use libp2p::{
    swarm::{behaviour, dummy, NetworkBehaviour},
    PeerId,
};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash as _, Hasher};
use std::io::{ErrorKind, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::task::Poll;
use std::time::{Duration, Instant};

use crate::{
    known_peer_item::KnownPeerItem, p2p_message::P2PMessage, peer_id_wrapper::PeerIdWrapper,
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
const HUB_ADDR: &str = "0.0.0.0";
const HUB_PORT: u16 = 42666;
pub const BUFFER_SIZE: usize = 2048;

/// 节点角色。前文已经提及，节点角色分为枢纽Hub节点和普通Normal节点
#[derive(Debug)]
enum PeerRole {
    Hub {
        hub_listener: TcpListener,
        known_peers: HashMap<SocketAddr, (PeerId, TcpStream)>,
    },
    Normal {
        stream_to_hub: TcpStream,
        known_peers: HashSet<PeerId>,
        next_heartbeat: Instant,
    },
}

impl core::fmt::Display for PeerRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerRole::Hub { .. } => f.write_str("Hub"),
            PeerRole::Normal { .. } => f.write_str("Normal"),
        }
    }
}

/// 消息ID，计算一个消息（字符串）的哈希，以用于去重
#[derive(Debug)]
pub struct MessageId(String);

impl From<&str> for MessageId {
    fn from(val: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let msg_bytes = val.as_bytes();
        msg_bytes.hash(&mut hasher);
        Self(hasher.finish().to_string())
    }
}

impl core::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// P2P事件，上层应用会收到这些事件，然后采取一些措施
#[derive(Debug)]
pub enum P2PEvent {
    UpdatedKnownPeers(Vec<libp2p::PeerId>),
    IncomingMessage(libp2p::PeerId, MessageId, String),
    ErrorAndExit,
}

#[derive(Debug)]
pub struct SimpleP2PBehaviour {
    role: PeerRole,
    buf: [u8; BUFFER_SIZE],
    local_peer_id: PeerId,
    /// 待发送的事件队列。
    /// 如果单次发送一个事件，则直接用 return Poll::Ready(...) 的形式完成。
    /// 只有在需要一次性发送多个事件时，才会进入该队列，并在poll函数的开头逐个发送。
    event_queue: Vec<P2PEvent>,
}

impl SimpleP2PBehaviour {
    pub fn new(local_peer_id: PeerId) -> Self {
        // 根据角色不同分配TcpListener或者TcpStream
        let role = match TcpListener::bind(format!("{}:{}", HUB_ADDR, HUB_PORT)) {
            Ok(hub_listener) => {
                hub_listener.set_nonblocking(true).unwrap();
                println!(
                    "The hub is listening at {}",
                    hub_listener.local_addr().unwrap(),
                );
                PeerRole::Hub {
                    hub_listener,
                    known_peers: HashMap::new(),
                }
            }
            Err(_) => {
                let stream_to_hub =
                    TcpStream::connect(format!("{}:{}", HUB_ADDR, HUB_PORT)).unwrap();
                stream_to_hub.set_nonblocking(true).unwrap();
                println!("The node has connected to the hub");
                PeerRole::Normal {
                    stream_to_hub,
                    known_peers: HashSet::new(),
                    next_heartbeat: Instant::now(),
                }
            }
        };

        Self {
            role,
            buf: [0u8; BUFFER_SIZE],
            local_peer_id,
            event_queue: Vec::new(),
        }
    }

    /// 广播消息，将消息广播给所有known_peers
    pub fn broadcast_string(&mut self, sender: PeerId, message: String) {
        match &mut self.role {
            PeerRole::Hub {
                hub_listener: _,
                known_peers,
            } => {
                // 直接自行广播即可
                if known_peers.len() == 0 {
                    println!("Insufficient peers!");
                    return;
                }
                for (_, (peer_id, stream)) in known_peers.iter_mut() {
                    if peer_id == &sender {
                        continue;
                    }
                    stream
                        .write_all(
                            &P2PMessage::Text {
                                sender: PeerIdWrapper::from(self.local_peer_id).into(),
                                text_message: message.clone(),
                            }
                            .serialize(),
                        )
                        .unwrap();
                }
            }
            PeerRole::Normal {
                stream_to_hub,
                known_peers: _,
                next_heartbeat: _,
            } => {
                // 请求 Hub 帮忙广播
                // lifetime注意：这儿的message一定活得比p2p_message长
                // todo: Hub自己也要收到自己发送的消息
                let p2p_message = P2PMessage::RequestBroadcast {
                    sender: PeerIdWrapper::from(self.local_peer_id).into(),
                    message,
                };
                stream_to_hub.write_all(&p2p_message.serialize()).unwrap();
            }
        }
    }
}

// 学着点，兄弟：https://github.com/libp2p/rust-libp2p/blob/master/protocols/mdns/src/behaviour.rs
impl NetworkBehaviour for SimpleP2PBehaviour {
    type ConnectionHandler = dummy::ConnectionHandler;

    type ToSwarm = P2PEvent;
    // 连上了就不管了
    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        _peer: PeerId,
        _local_addr: &libp2p::Multiaddr,
        _remote_addr: &libp2p::Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }
    // 同理，连上了就不管了
    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        _peer: PeerId,
        _addr: &libp2p::Multiaddr,
        _role_override: libp2p::core::Endpoint,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }
    // 暂且不管
    fn on_swarm_event(&mut self, _event: behaviour::FromSwarm) {}
    // 暂且不管
    fn on_connection_handler_event(
        &mut self,
        _peer_id: PeerId,
        _connection_id: libp2p::swarm::ConnectionId,
        _event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
    }
    // 核心！！
    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<behaviour::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>> {
        if let Some(event) = self.event_queue.pop() {
            return Poll::Ready(behaviour::ToSwarm::GenerateEvent(event));
        }

        match &mut self.role {
            PeerRole::Hub {
                hub_listener,
                known_peers,
            } => {
                // 先把后面可能要用到的数据算好
                let known_peers_vec: Vec<KnownPeerItem> = known_peers
                    .iter()
                    .map(|(_addr, (peer_id, _stream))| KnownPeerItem(PeerIdWrapper::from(peer_id)))
                    .collect();
                // 检查是否有新的连接，并建立从TCP地址到PeerId和TcpStream的映射关系
                while let Ok((mut stream, addr)) = hub_listener.accept() {
                    stream.set_nonblocking(true).unwrap();
                    println!("New connection from {}", addr);
                    // 此时新节点一定会发送 RequestKnownPeers 消息
                    let p2p_message = loop {
                        match P2PMessage::read_from_tcp_stream(&mut stream, &mut self.buf) {
                            Ok(p2p_message) => break p2p_message,
                            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                continue;
                            }
                            Err(e) => {
                                // 因为此时 Hub 还没有把新节点加入 known_peers ，所以直接退出就好了
                                eprintln!("[Hub] Error reading from stream: {}", e);
                                return Poll::Ready(behaviour::ToSwarm::GenerateEvent(
                                    P2PEvent::ErrorAndExit,
                                ));
                            }
                        };
                    };
                    match p2p_message {
                        P2PMessage::RequestKnownPeers { sender } => {
                            let sender_peer_id = sender.into_inner();
                            println!("New node {} has connected to the hub", sender_peer_id);
                            // 发送 KnownPeers 消息
                            let p2p_message = P2PMessage::KnownPeers {
                                sender: PeerIdWrapper::from(self.local_peer_id).into(),
                                known_peers: known_peers_vec.clone(),
                            };
                            stream.write_all(&p2p_message.serialize()).unwrap();
                            known_peers.insert(addr, (sender_peer_id, stream));
                        }
                        _ => unreachable!(),
                    }
                }
                let known_peers_vec: Vec<KnownPeerItem> = known_peers
                    .iter()
                    .map(|(_addr, (peer_id, _stream))| KnownPeerItem(PeerIdWrapper::from(peer_id)))
                    .collect();
                // 处理老连接中的消息
                // 老节点可能会发送 RequestKnownPeers, RequestBroadcast 两种消息
                let mut msg_to_broadcast = Vec::<(PeerId, String)>::new();
                let mut peer_to_remove = Vec::<SocketAddr>::new();
                for (peer_addr, (_peer_id, stream)) in known_peers.iter_mut() {
                    match P2PMessage::read_from_tcp_stream(stream, &mut self.buf) {
                        Ok(p2p_message) => match p2p_message {
                            P2PMessage::RequestKnownPeers { sender: _ } => {
                                // 如法炮制
                                // 发送 KnownPeers 消息
                                let p2p_message = P2PMessage::KnownPeers {
                                    sender: PeerIdWrapper::from(self.local_peer_id).into(),
                                    known_peers: known_peers_vec.clone(),
                                };
                                stream.write_all(&p2p_message.serialize()).unwrap();
                            }
                            P2PMessage::RequestBroadcast { sender, message } => {
                                msg_to_broadcast.push((sender.into_inner(), message));
                            }
                            _ => unreachable!(),
                        },
                        Err(e) if e.kind() == ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            println!("[Hub] Error reading from stream: {}", e);
                            peer_to_remove.push(*peer_addr);
                            // return Poll::Ready(behaviour::ToSwarm::GenerateEvent(
                            //     P2PEvent::ErrorAndExit,
                            // ));
                        }
                    }
                }
                // 处理 peer_to_remove
                for peer_addr in peer_to_remove {
                    known_peers.remove(&peer_addr);
                }
                // 处理 msg_to_broadcast
                if msg_to_broadcast.len() > 0 {
                    for (sender, message) in msg_to_broadcast {
                        self.broadcast_string(sender, message.clone());
                        self.event_queue.push(P2PEvent::IncomingMessage(
                            sender,
                            MessageId::from(message.as_str()),
                            message,
                        ));
                    }
                }
            }
            PeerRole::Normal {
                stream_to_hub,
                known_peers,
                next_heartbeat,
            } => {
                // 首先 RequestKnownPeers
                if Instant::now() >= *next_heartbeat {
                    *next_heartbeat = Instant::now() + HEARTBEAT_INTERVAL;
                    let request_known_peers_message = P2PMessage::RequestKnownPeers {
                        sender: PeerIdWrapper::from(self.local_peer_id).into(),
                    };
                    stream_to_hub
                        .write_all(&request_known_peers_message.serialize())
                        .unwrap();
                }
                match P2PMessage::read_from_tcp_stream(stream_to_hub, &mut self.buf) {
                    Ok(msg) => match msg {
                        P2PMessage::KnownPeers {
                            sender: _,
                            known_peers: new_known_peers,
                        } => {
                            let mut new_peer_ids = vec![];
                            for each in new_known_peers {
                                let new_peer_id = each.0.into_inner();
                                if known_peers.contains(&new_peer_id)
                                    || new_peer_id == self.local_peer_id
                                {
                                    continue;
                                }
                                known_peers.insert(new_peer_id);
                                new_peer_ids.push(new_peer_id);
                            }
                            return Poll::Ready(behaviour::ToSwarm::GenerateEvent(
                                P2PEvent::UpdatedKnownPeers(new_peer_ids),
                            ));
                        }
                        P2PMessage::Text {
                            sender,
                            text_message,
                        } => {
                            return Poll::Ready(behaviour::ToSwarm::GenerateEvent(
                                P2PEvent::IncomingMessage(
                                    sender.into_inner(),
                                    MessageId::from(text_message.as_str()),
                                    text_message,
                                ),
                            ))
                        }
                        _ => {}
                    },
                    Err(e) => {
                        if e.kind() != ErrorKind::WouldBlock {
                            eprintln!("[Normal] Error reading from stream: {}", e);
                            return Poll::Ready(behaviour::ToSwarm::GenerateEvent(
                                P2PEvent::ErrorAndExit,
                            ));
                        }
                    }
                }
            }
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
