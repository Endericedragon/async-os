use core::arch::asm;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    time::{Instant, SystemTime},
};

mod ms_client;
mod run_chunked_file_transfer;
mod run_daytime;
mod run_echo;

fn main() {
    greeting_through_syscall();

    // try_all_protocols();
    // try_protocols_separately();
    try_cftp();
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

#[allow(unused)]
fn try_cftp() {
    let mut negotiator = ms_client::Negotiator::new()
        .add_protocol("/nika/1.0")
        .add_protocol("/akusta/1.0")
        .add_protocol("/chunked-file-transfer/1.0");
        // .add_protocol("/echo/1.0");
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
                run_chunked_file_transfer::transfer_file(
                    "audacity-win-3.7.3-64bit.exe",
                    &mut negotiator,
                    &mut buf,
                ); // bigger file
                   // run_chunked_file_transfer::transfer_file("end_poem.txt", &mut negotiator, &mut buf); // smaller file
            }
            Some(other_protocol) => unimplemented!("Unsupported protocol {}!", other_protocol),
            None => unimplemented!("No protocol selected!"),
        }
    } else {
        eprintln!("Failed to negotiate with remote server!");
    }
}

#[allow(unused)]
fn try_all_protocols() {
    println!(
        "{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros()
    );
    let mut negotiator = ms_client::Negotiator::new();
    let mut f = File::open("proto_ids.txt").expect("Failed to open file!");
    let mut reader = BufReader::new(&mut f);
    let mut line = String::new();
    let mut counter: i32 = 0;
    while reader.read_line(&mut line).expect("Failed to read line!") > 0 {
        negotiator = negotiator.add_protocol(&line.trim());
        line.clear();
        counter += 1;
        if counter >= 100 {
            break;
        }
    }
    negotiator = negotiator.add_protocol("/daytime/1.0");

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
                run_chunked_file_transfer::transfer_file("end_poem.txt", &mut negotiator, &mut buf);
            }
            Some(other_protocol) => unimplemented!("Unsupported protocol {}!", other_protocol),
            None => unimplemented!("No protocol selected!"),
        }
    } else {
        eprintln!("Failed to negotiate with remote server!");
    }
    println!(
        "{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros()
    );
}

#[allow(unused)]
fn try_protocols_separately() {
    let mut f = File::open("proto_ids.txt").expect("Failed to open file!");
    let mut reader = BufReader::new(&mut f);
    let mut line = String::new();
    while reader.read_line(&mut line).expect("Failed to read line!") > 0 {
        let mut negotiator = ms_client::Negotiator::new().add_protocol(&line.trim());
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
                    run_chunked_file_transfer::transfer_file(
                        "end_poem.txt",
                        &mut negotiator,
                        &mut buf,
                    );
                }
                Some(other_protocol) => unimplemented!("Unsupported protocol {}!", other_protocol),
                None => unimplemented!("No protocol selected!"),
            }
        } else {
            eprintln!("Failed to negotiate with remote server!");
        }
        line.clear();
    }
}
