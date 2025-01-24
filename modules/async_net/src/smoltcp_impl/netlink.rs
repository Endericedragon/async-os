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
    state: NetlinkState,
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
}

impl Default for NetlinkSocket {
    fn default() -> Self {
        Self {
            pid: 0,     // 在bind时设置，内核看到的就是创建Socket的进程的pid
            dst_pid: 0, // 在sendto中设置，内核看到的应该是0（即内核系统进程的pid）
            flags: 0,
            state: NetlinkState::Unconnected,
        }
    }
}
