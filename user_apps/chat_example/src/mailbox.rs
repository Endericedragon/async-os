use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const MAGIC_PORT: u16 = 42666;

/// 邮箱。工作在42666端口上的UDP协议。
pub struct Mailbox {
    identity: String,
    recv_buf: Vec<u8>,
    send_buf: Vec<u8>,
    broadcast_socket: UdpSocket,
    known_peers: HashMap<SocketAddr, String>,
}

impl Mailbox {
    pub fn new(identity: String) -> Self {
        let broadcast_socket =
            UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], MAGIC_PORT))).unwrap();
        broadcast_socket.set_broadcast(true).unwrap();
        broadcast_socket.set_nonblocking(true).unwrap();
        Self {
            identity,
            recv_buf: vec![0u8; 8192],
            send_buf: vec![0u8; 8192],
            broadcast_socket,
            known_peers: HashMap::new(),
        }
    }

    /// 将存储在 `self.buf` 中的消息广播出去。使用前，必须先调用 `prepare_message` 方法将消息写入 `self.buf`。
    fn broadcast_buf(&self) {
        let Ok(size_sent) = self.broadcast_socket.send_to(
            &self.send_buf,
            SocketAddr::from(([255, 255, 255, 255], MAGIC_PORT)),
        ) else {
            panic!("Failed to broadcast UDP message.");
        };
        // 能看到输出
        // println!("Broadcasted message of size {}.", size_sent);
    }

    /// 将消息写入 `self.buf`，准备发送。
    fn prepare_message(&mut self, message: &[u8]) {
        self.send_buf.clear();
        self.send_buf.extend_from_slice(message);
    }

    /// 从广播端口接收消息，并将其存入 `self.buf`。
    pub fn recv_from_broadcast(&mut self) -> Option<(usize, SocketAddr)> {
        match self.broadcast_socket.recv_from(&mut self.recv_buf) {
            Ok((size, addr)) => Some((size, addr)),
            Err(_err) => None,
        }
    }

    /// 启动邮箱，开始监听广播端口，处理收到的消息。
    pub fn run(mut self) {
        let (tx, rx) = mpsc::channel::<String>();

        // 用户输入线程：控制用户输入，然后把内容放入共享字符串中
        let user_input_thread = thread::spawn(move || {
            println!("Waiting for input...");
            let mut input_string = String::new();
            loop {
                std::io::stdin().read_line(&mut input_string).unwrap();
                match tx.send(input_string.trim().to_string()) {
                    Ok(_) => (),
                    Err(err) => println!("Failed to queue the message, error: {}.", err),
                }
                input_string.clear();
            }
        });

        // 邮箱处理线程：处理收到的消息，并回应
        let mailbox_process_thread = thread::spawn(move || {
            println!("Background task is running...");
            let local_ip = get_local_ip();
            let mut last_time_check = Instant::now();
            loop {
                // 每隔两秒，广播自身身份
                if last_time_check.elapsed().as_secs() > 2 {
                    println!("Broadcasting who I am...");
                    let msg_whoami = MailInner::WhoAmI(String::from(self.identity.clone()));
                    let mail = Mail {
                        sender: self.identity.clone(),
                        inner: msg_whoami,
                    };
                    self.prepare_message(&mail.as_bytes());
                    self.broadcast_buf();
                    last_time_check = Instant::now();
                }

                // 尝试发送消息，若共享字符串中有内容，则发
                // println!("Trying to send message...");
                // match rx.try_recv() {
                //     Ok(input_string) => {
                //         let msg = input_string.trim().to_string();
                //         // println!("Sending message: {}", msg);
                //         let msg_text = MailInner::TextMessage(msg);
                //         let mail = Mail {
                //             sender: self.identity.clone(),
                //             inner: msg_text,
                //         };
                //         self.prepare_message(&mail.as_bytes());
                //         self.broadcast_buf();
                //     }
                //     Err(_err) => (),
                // }

                // 从广播端口接收消息
                // println!("Trying to receive message...");
                // if let Some((_recv_size, peer_addr)) = self.recv_from_broadcast() {
                //     if local_ip == peer_addr.ip() {
                //         continue;
                //     }

                //     // 处理收到的消息并回应
                //     // println!(
                //     //     "Processing message of size {} from {}...",
                //     //     _recv_size, peer_addr
                //     // );
                //     let mail: Mail = bincode::deserialize(&self.recv_buf).unwrap();
                //     match mail.inner {
                //         MailInner::WhoAmI(who) => {
                //             // 处理非自身产生的消息
                //             if !self.known_peers.contains_key(&peer_addr) {
                //                 println!("New peer: {} ({})", who, peer_addr);
                //                 self.known_peers.insert(peer_addr, who);
                //             }
                //         }
                //         MailInner::TextMessage(text) => {
                //             let name_of_peer = self.known_peers.get(&peer_addr).unwrap();
                //             println!("{} said: {}", name_of_peer, text);
                //         }
                //     }
                // };
            }
        });

        user_input_thread.join().unwrap();
        mailbox_process_thread.join().unwrap();
    }
}

#[derive(Serialize, Deserialize)]
struct Mail {
    sender: String,
    inner: MailInner,
}

impl Mail {
    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum MailInner {
    WhoAmI(String),
    TextMessage(String),
}

impl MailInner {
    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

/// 获取本地IP地址
pub fn get_local_ip() -> IpAddr {
    let dummy = UdpSocket::bind("0.0.0.0:0").unwrap();
    dummy.connect("223.5.5.5:0").unwrap();
    let local_ip = dummy.local_addr().unwrap().ip();
    local_ip
}
