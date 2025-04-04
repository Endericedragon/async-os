use crate::ms_client::Negotiator;
use parity_scale_codec::{Decode, Encode};
use std::io::Read;

const FRAME_SIZE_LIMIT: usize = 1024;

#[derive(Encode, Decode)]
enum ChunkedFileTransferMessage {
    // used by sender
    Init {
        filename: String,
        size: i64,
    },
    Chunk {
        id: i64,
        data: [u8; FRAME_SIZE_LIMIT],
        length: i64,
    },
    Checksum(u32),
    // used by receiver
    ServerReady,
    TransferDone,
    TransferError,
    ChecksumOk,
    ChecksumError,
}

fn get_message_from_negotiator(
    negotiator: &mut Negotiator,
    buf: &mut [u8],
) -> ChunkedFileTransferMessage {
    let length = negotiator.recv(buf) as usize;
    ChunkedFileTransferMessage::decode(&mut &buf[..length]).expect("Failed to decode message!")
}

pub fn transfer_end_poem(negotiator: &mut Negotiator, buf: &mut [u8]) {
    let mut f = match std::fs::File::open("/end_poem.txt") {
        Ok(f) => {
            println!("Poem file opened!");
            f
        }
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return;
        }
    };

    assert!(
        negotiator.send(
            &ChunkedFileTransferMessage::Init {
                filename: String::from("end_poem.txt"),
                size: f.metadata().unwrap().len() as i64,
            }
            .encode()
        ) > 0
    );

    assert!(matches!(
        get_message_from_negotiator(negotiator, buf),
        ChunkedFileTransferMessage::ServerReady
    ));

    let mut hasher = crc32fast::Hasher::new();
    let mut chunk_id = 0;
    loop {
        let mut chunk = [0u8; FRAME_SIZE_LIMIT];
        let num_byte_read = Read::read(&mut f, &mut chunk).unwrap();
        if num_byte_read == 0 {
            break;
        }
        hasher.update(&chunk[..num_byte_read]);
        let chunk_msg = ChunkedFileTransferMessage::Chunk {
            id: chunk_id,
            data: chunk,
            length: num_byte_read as i64,
        };
        let encoded_chunk_msg = chunk_msg.encode();
        assert!(negotiator.send(&encoded_chunk_msg) > 0);
        chunk_id += 1;
        let length = negotiator.recv(buf) as usize;
        let msg = ChunkedFileTransferMessage::decode(&mut &buf[..length]).unwrap();
        match msg {
            ChunkedFileTransferMessage::TransferDone => {
                println!("Transfered chunk {}.", chunk_id);
                continue;
            }
            ChunkedFileTransferMessage::TransferError => {
                eprintln!("Transfer error!");
                return;
            }
            _ => unreachable!(),
        }
    }
    assert!(negotiator.send(&ChunkedFileTransferMessage::Checksum(hasher.finalize()).encode()) > 0);
    // let length = negotiator.recv(buf) as usize;
    match get_message_from_negotiator(negotiator, buf) {
        ChunkedFileTransferMessage::ChecksumOk => {
            println!("Transfer done!");
        }
        ChunkedFileTransferMessage::ChecksumError => {
            eprintln!("Checksum error!");
        }
        _ => unreachable!(),
    }
}
