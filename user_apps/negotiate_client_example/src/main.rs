use core::arch::asm;

mod ms_client;
mod run_chunked_file_transfer;
mod run_daytime;
mod run_echo;

fn main() {
    greeting_through_syscall();

    // let mut negotiator = ms_client::Negotiator::new()
    //     .add_protocol("/nika/1.0")
    //     .add_protocol("/akusta/2.0")
    //     .add_protocol("/echo/1.0");
    let mut negotiator = ms_client::Negotiator::new()
        .add_protocol("/tribios/9.8")
        .add_protocol("/ain/4.4")
        .add_protocol("/chunked-file-transfer/1.0")
        .add_protocol("/daytime/1.0")
        .add_protocol("/echo/1.0");

    // 通过log看到QEMU提供的网关地址就是10.0.2.2
    if negotiator.dial([10, 0, 2, 2], 42666) {
        // 成功连接到远程主机，proto为"/echo/1.0"，fd为连接的文件描述符
        // println!(
        //     "Negotiation successful! {:?}",
        //     negotiator.selected_proto_idx
        // );
        let mut buf = [0u8; 2048];
        match negotiator.acquire_selected_protocol_identifier() {
            Some("/echo/1.0") => {
                run_echo::run(&mut negotiator, &mut buf);
            }
            Some("/daytime/1.0") => {
                run_daytime::get_remote_time(&mut negotiator, &mut buf);
            }
            Some("/chunked-file-transfer/1.0") => {
                run_chunked_file_transfer::transfer_end_poem(&mut negotiator, &mut buf);
            }
            _ => unimplemented!(),
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
