use core::arch::asm;
use std::io::stdin;

mod ms_client;

fn main() {
    greeting_through_syscall();

    let mut negotiator = ms_client::Negotiator::new();
    negotiator.add_protocol("/nika/1.0");
    negotiator.add_protocol("/akusta/2.0");
    negotiator.add_protocol("/echo/1.0");
    // 通过log看到QEMU提供的网关地址就是10.0.2.2
    if negotiator.dial([10, 0, 2, 2], 42666) {
        // 成功连接到远程主机，proto为"/echo/1.0"，fd为连接的文件描述符
        println!(
            "Negotiation successful! {:?}",
            negotiator.selected_proto_idx
        );
        let mut buf = [0u8; 1024];
        let mut user_input = String::new();
        loop {
            print!(">>> ");
            stdin().read_line(&mut user_input).unwrap();
            negotiator.send(user_input.as_bytes());
            negotiator.recv(&mut buf);
            println!("Echo from listener: {}", String::from_utf8_lossy(&buf));
        }
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
