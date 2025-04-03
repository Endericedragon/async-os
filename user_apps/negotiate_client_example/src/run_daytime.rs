use crate::ms_client::Negotiator;

pub fn get_remote_time(negotiator: &mut Negotiator, buf: &mut [u8]) {
    negotiator.send(b"_dummy_");
    let length = negotiator.recv(buf) as usize;
    let returned_time =
        chrono::DateTime::parse_from_rfc3339(&String::from_utf8_lossy(&buf[..length])).unwrap();
    println!("Returned time from listener: {}", returned_time);
}
