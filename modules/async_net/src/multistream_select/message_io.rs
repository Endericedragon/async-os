use core::result::Result;
use core::task::{Context, Poll};
use crate::TcpSocket;

use super::protocol::{Message, ProtocolError};

pub struct MessageIO {
    socket: TcpSocket,
}

impl MessageIO {
    pub fn new(socket: TcpSocket) -> Self {
        Self { socket }
    }

    // todo: 为MessageIO实现poll_ready和start_send

    pub fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ProtocolError>> {
        todo!()
    }

    pub fn start_send(&mut self, msg: Message) -> Result<(), ProtocolError> {
        todo!()
    }
}
