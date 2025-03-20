use core::arch::asm;

mod ms_client;

fn main() {
    greeting_through_syscall();

    let mut negotiator = ms_client::Negotiator::new();
    negotiator.add_protocol("/nika/1.0");
    negotiator.add_protocol("/echo/2.0");
    negotiator.add_protocol("/echo/1.0");
    if negotiator.dial([127, 0, 0, 1], 42666) {
        // 成功连接到远程主机，proto为"/echo/1.0"，fd为连接的文件描述符
        let mut buf = [0u8; 1024];
        negotiator.send(b"hello");
        negotiator.recv(&mut buf);
        println!("Received: {}", String::from_utf8_lossy(&buf));
    } else {
        eprintln!("Failed to negotiate with remote server!");
    }
}

fn greeting_through_syscall() {
    let message = "Greetings!\n".as_bytes();
    let len = message.len();
    const SYS_WRITE: usize = 64;
    let rua = 1usize;
    // const STDOUT: usize = 1;
    unsafe {
        asm!(
            "ecall",
            in("a7") SYS_WRITE,
            in("a0") rua,
            in("a1") message.as_ptr(),
            in("a2") len,
        );
    }
}
