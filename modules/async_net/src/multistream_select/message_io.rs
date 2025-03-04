use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use bytes::BytesMut;
use core::fmt::Debug;
use core::future::Future;
use core::result::Result;
use core::task::{Context, Poll};
use futures::task::noop_waker_ref;

use crate::TcpSocket;

use super::protocol::{Message, ProtocolError};

pub struct MessageIO {
    socket: Arc<TcpSocket>,
}

impl Debug for MessageIO {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MessageIO")
            .field("socket", &("TcpSocket {...}".to_string()))
            .finish()
    }
}

impl MessageIO {
    pub fn new(socket: TcpSocket) -> Self {
        Self {
            socket: Arc::new(socket),
        }
    }

    /// 确定内部的TcpSocket是否可写。必须可写，才能调用`start_send`方法。
    /// 原版的poll_ready方法会在返回Poll::Ready(Ok(()))之前，把缓冲区中所有的信息全部发送掉。
    /// 这是因为原版的底层并不是TcpSocket，而是一个抽象的接口外加一个缓冲区，所以工作原理不同。
    pub fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ProtocolError>> {
        async fn wrapper(socket: &TcpSocket) -> bool {
            match socket.poll().await {
                Ok(poll_state) => poll_state.writable,
                Err(err) => panic!("[poll_ready]: Poll error: {}", err),
            }
        }

        match Box::pin(wrapper(&self.socket)).as_mut().poll(cx) {
            Poll::Ready(true) => return Poll::Ready(Ok(())),
            Poll::Ready(false) | Poll::Pending => return Poll::Pending,
        }
    }

    /// 利用内部的TcpSocket发送消息。
    /// 原版的start_send方法只是把要发送的数据写入缓冲区而已，并不会直接发送掉。
    /// 这是因为原版的底层并不是TcpSocket，而是一个抽象的接口外加一个缓冲区，所以工作原理不同。
    pub fn start_send(&mut self, msg: Message) -> Result<(), ProtocolError> {
        let mut buf = BytesMut::new();
        msg.encode(&mut buf)?;
        // 在同步代码中强行运行异步代码，只能阻塞地运行
        let mut cx = Context::from_waker(noop_waker_ref());
        let mut pinned_task = Box::pin(self.socket.send(&buf));
        loop {
            match pinned_task.as_mut().poll(&mut cx) {
                Poll::Ready(Ok(_num_byte_sent)) => return Ok(()),
                Poll::Ready(Err(err)) => panic!("[start_send]: Send error: {}", err),
                Poll::Pending => (),
            }
        }
    }
}
