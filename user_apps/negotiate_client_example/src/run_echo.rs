use crate::ms_client::Negotiator;
use std::io::{stdin, stdout, Write as _};

pub fn run(negotiator: &mut Negotiator, buf: &mut [u8]) {
    loop {
        let mut user_input = String::new();
        print!(">>> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut user_input).unwrap();
        // Remember to remove extra \n
        negotiator.send(user_input.trim().as_bytes());
        negotiator.recv(buf);
        println!("Echo from listener: {}", String::from_utf8_lossy(&buf));
    }
}
