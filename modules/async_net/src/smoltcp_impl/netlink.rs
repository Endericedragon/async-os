use async_collections::{vec /*是那个宏定义*/, Vec, VecDeque};
use public_types::LibcSockAddrNl;

/** 类似于Linux中的netlink_sock，为Netlink协议提供支持
```c
struct netlink_sock {
    /* struct sock has to be the first member of netlink_sock */
    struct sock sk;
    u32 pid;
    u32 dst_pid;
    u32 dst_group;
    u32 flags;
    u32 subscriptions;
    // ngroups 表示 最大支持的多播组数量
    u32 ngroups;
    // groups保存组位掩码
    unsigned long* groups;
    unsigned long state;
    wait_queue_head_t wait;
    struct netlink_callback* cb;
    struct mutex* cb_mutex;
    struct mutex cb_def_mutex;
    // netlink_rcv 回调函数，当收到用户态发来的消息时被调用
    void (*netlink_rcv)(struct sk_buff* skb);
    // 这份Linux代码是2.6.39.4版本，还没有netlink_bind和netlink_unbind函数
    struct module* module;
};
```
 */
#[allow(dead_code)]
#[derive(Debug)]
pub struct NetlinkSocket {
    pid: u32,
    dst_pid: u32,
    flags: u32,
    groups: u32,
    ngroups: u32,
    state: NetlinkState,
    recv_queue: VecDeque<SkBuff>,
    send_queue: VecDeque<SkBuff>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum NetlinkState {
    Unconnected,
    Connecting,
    Connected,
}

impl NetlinkSocket {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&self, addr_nl: LibcSockAddrNl) {
        let mutable_self = self as *const NetlinkSocket as *mut NetlinkSocket;
        // 实在不行只能用unsafe绕开mut的限制了
        unsafe {
            (*mutable_self).pid = addr_nl.nl_pid;
            (*mutable_self).state = NetlinkState::Connected;
            (*mutable_self).groups = addr_nl.nl_groups;
        }
    }
}

impl Default for NetlinkSocket {
    fn default() -> Self {
        Self {
            pid: 0,     // 在bind时设置，内核看到的就是创建Socket的进程的pid
            dst_pid: 0, // 在sendto中设置，内核看到的应该是0（即内核系统进程的pid）
            flags: 0,
            groups: 0, // 默认不加入任何组播组
            ngroups: 0,
            state: NetlinkState::Unconnected,
            recv_queue: VecDeque::new(),
            send_queue: VecDeque::new(),
        }
    }
}

#[derive(Debug)]
pub struct SkBuff {
    head: Vec<u8>, // 数据包缓冲区
    data: usize,   // 当前协议层数据的起始位置
    tail: usize,   // 当前协议层数据的结束位置
    len: usize,    // 数据包的总长度
    protocol: u16, // 协议类型
}

impl SkBuff {
    fn new(buffer_size: usize) -> Self {
        SkBuff {
            head: vec![0; buffer_size],
            data: 0,
            tail: 0,
            len: 0,
            protocol: 0,
        }
    }

    fn push_data(&mut self, data: &[u8]) {
        let end = self.data + data.len();
        if end > self.head.len() {
            panic!("Buffer overflow");
        }
        self.head[self.data..end].copy_from_slice(data);
        self.tail = end;
        self.len += data.len();
    }

    fn pull_data(&mut self, len: usize) {
        self.data += len;
        self.len -= len;
    }
}
