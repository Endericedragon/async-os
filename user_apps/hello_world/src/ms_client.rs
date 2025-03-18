use core::arch::asm;
use std::net::SocketAddr;

const SYS_MULTISTREAM_SELECT_DIAL: usize = 42666;

struct Negotiator {
    protos: Vec<String>,
}

impl Negotiator {
    pub fn new() -> Self {
        Self { protos: vec![] }
    }

    pub fn add_protocol(&mut self, protocol: String) {
        self.protos.push(protocol);
    }

    pub fn dial(&mut self, addr: SocketAddr) {
        unsafe {
            // asm!(
            //     "ecall",
            //     in("a7") SYS_WRITE,
            //     in("a0") rua,
            //     in("a1") message.as_ptr(),
            //     in("a2") len,
            // );
            // syscall_multistream_select_dial(num_proto: usize, protos: Vec<String>) -> usize(fd)
            asm!(
                "ecall",
                in("a7") SYS_MULTISTREAM_SELECT_DIAL,
            );
        }
    }
}

fn try_negotiate() {
    let mut negotiator = Negotiator::new();
    negotiator.add_protocol("p1".to_string());
    negotiator.add_protocol("proto2".to_string());
    negotiator.dial(SocketAddr::from(([127, 0, 0, 1], 42666)))
}
