use core::net::SocketAddr;
use std::hash::{Hash as _, Hasher};
use libp2p::swarm::{behaviour, dummy, NetworkBehaviour};
use libp2p::PeerId;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
const MAGIC_PORT: [usize; 2] = [42666, 42667];

#[derive(Debug)]
pub enum DiscoveryEvent {
    Discovered(libp2p::PeerId),
    IncomingMessage(libp2p::PeerId, String, String),
}

#[derive(Debug)]
pub struct DiscoveryBehaviour {
    sock: UdpSocket,
    port: usize,
    buf: [u8; 1024],
    known_peers: HashMap<SocketAddr, PeerId>,
    next_broadcast: Instant,
    local_peer_id: PeerId,
}

impl DiscoveryBehaviour {
    pub fn new(local_peer_id: PeerId) -> Self {
        let mut idx = 0usize;
        let mut port = MAGIC_PORT[idx];
        let udp_socket = {
            let mut udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", MAGIC_PORT[idx]));
            while udp_socket.is_err() && idx < MAGIC_PORT.len() {
                idx += 1;
                udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", MAGIC_PORT[idx]));
                port = MAGIC_PORT[idx];
            }
            if udp_socket.is_err() {
                panic!();
            }
            let udp_socket = udp_socket.unwrap();
            udp_socket.set_broadcast(true).unwrap();
            udp_socket.set_nonblocking(true).unwrap();
            udp_socket
        };
        Self {
            sock: udp_socket,
            port,
            buf: [0; 1024],
            known_peers: HashMap::new(),
            next_broadcast: Instant::now(),
            local_peer_id,
        }
    }

    pub fn broadcast(&self, msg: &[u8]) {
        for port in MAGIC_PORT {
            if port == self.port {
                continue;
            }
            self.sock
                .send_to(msg, format!("255.255.255.255:{}", port))
                .unwrap();
        }
    }
}

// 学着点，兄弟：https://github.com/libp2p/rust-libp2p/blob/master/protocols/mdns/src/behaviour.rs
impl NetworkBehaviour for DiscoveryBehaviour {
    type ConnectionHandler = dummy::ConnectionHandler;

    type ToSwarm = DiscoveryEvent;
    // 连上了就不管了
    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        peer: PeerId,
        local_addr: &libp2p::Multiaddr,
        remote_addr: &libp2p::Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }
    // 同理，连上了就不管了
    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        peer: PeerId,
        addr: &libp2p::Multiaddr,
        role_override: libp2p::core::Endpoint,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn on_swarm_event(&mut self, event: behaviour::FromSwarm) {
        /* 暂且不管 */
    }

    fn on_connection_handler_event(
        &mut self,
        _peer_id: PeerId,
        _connection_id: libp2p::swarm::ConnectionId,
        _event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        /* 暂且不管 */
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<behaviour::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>>
    {
        // 广播心跳，让其他节点发现我们
        if Instant::now() >= self.next_broadcast {
            cx.waker().wake_by_ref();
            self.next_broadcast = Instant::now() + HEARTBEAT_INTERVAL;
            let msg_local_peer_id = self.local_peer_id.to_bytes();
            self.broadcast(&msg_local_peer_id);
        }

        // 通过接收广播来发现节点
        self.buf.fill(0);
        match self.sock.recv_from(&mut self.buf) {
            Ok((length, peer_sockaddr)) => {
                if let Ok(peer_id) = PeerId::from_bytes(&self.buf[..length]) {
                    if !self.known_peers.contains_key(&peer_sockaddr)
                        && peer_id != self.local_peer_id
                    {
                        self.known_peers.insert(peer_sockaddr, peer_id);
                        return std::task::Poll::Ready(behaviour::ToSwarm::GenerateEvent(
                            DiscoveryEvent::Discovered(peer_id),
                        ));
                    }
                } else if let Ok(msg) = String::from_utf8(self.buf[..length].to_vec()) {
                    let remote_peer_id = self.known_peers.get(&peer_sockaddr).unwrap();
                    let msg_id = {
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        let msg_bytes = msg.as_bytes();
                        msg_bytes.hash(&mut hasher);
                        hasher.finish().to_string()
                    };
                    return std::task::Poll::Ready(behaviour::ToSwarm::GenerateEvent(
                        DiscoveryEvent::IncomingMessage(*remote_peer_id, msg_id, msg),
                    ));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 继续等待
                cx.waker().wake_by_ref();
            }
            Err(e) => {
                println!("Error receiving: {e}");
                cx.waker().wake_by_ref();
            }
        }
        cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }
}
