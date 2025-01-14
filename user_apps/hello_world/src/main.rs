// use std::{
//     io::{self, prelude::*},
//     net::{TcpListener, TcpStream},
// };

use core::arch::asm;

const MULTIHASH_IDENTITY_CODE: u64 = 0;
const MULTIHASH_SHA256_CODE: u64 = 0x12;

type Multihash = multihash::Multihash<64>;

fn main() {
    greeting_through_syscall();

    let mut emulated_key = [0u8; 64];
    let mut rng = fastrand::Rng::with_seed(20250114);
    rng.fill(&mut emulated_key);

    let hash1 = Multihash::wrap(MULTIHASH_IDENTITY_CODE, &emulated_key).unwrap();
    let hash2 = Multihash::wrap_with_syscall(MULTIHASH_SHA256_CODE, &emulated_key).unwrap();

    // unsafe {
    //     let mut hash2_mut_ptr = &mut hash2 as *mut Multihash;
    //     asm!(
    //         "ecall",
    //         in("a7") 10000,
    //         in("a0") MULTIHASH_IDENTITY_CODE,
    //         in("a1") (&emulated_key as *const u8),
    //         in("a2") emulated_key.len(),
    //         in("a3") hash2_mut_ptr,
    //     );
    // }

    println!("hash1 = {:?}", hash1);
    println!("hash2 = {:?}", hash2);

    // let listener = TcpListener::bind(("0.0.0.0", 7878)).expect("Failed to bind!");

    // loop {
    //     let Ok((mut tcp_stream, socket_addr)) = listener.accept() else {
    //         println!("Failed to accept connection!");
    //         break;
    //     };

    //     println!("Connection from: {}", socket_addr);

    //     // match echo_server(tcp_stream) {
    //     //     Err(e) => println!("client connection error: {:?}", e),
    //     //     Ok(()) => println!("client closed successfully"),
    //     // }

    //     let mut buffer = [0u8; 1024];

    //     loop {
    //         match tcp_stream.read(&mut buffer) {
    //             Ok(0) => {
    //                 // client closed connection
    //                 break;
    //             }
    //             Ok(n) => {
    //                 println!("Received {} bytes.", n);
    //                 println!("Inbox: {}", String::from_utf8_lossy(&buffer));
    //                 tcp_stream
    //                     .write_all(b"Hello, client!\n")
    //                     .expect("Failed to write to stream!");
    //             }
    //             Err(_) => {
    //                 println!("Failed to read from stream!");
    //                 break;
    //             }
    //         }
    //     }
    // }
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
