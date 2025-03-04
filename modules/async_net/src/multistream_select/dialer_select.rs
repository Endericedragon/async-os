use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use async_collections::VecDeque;
use async_io::{AsyncRead, AsyncWrite};

use crate::TcpSocket;

use super::message_io::MessageIO;
use super::negotiated::NegotiationError;
use super::protocol::{HeaderLine, Message, Protocol};
use super::Version;
use alloc::string::String;

pub async fn dialer_select<R>(
    async_rwer: TcpSocket,
    protocols: VecDeque<String>,
    version: Version,
) -> DialerSelectFuture
where
    R: AsyncRead + AsyncWrite,
{
    DialerSelectFuture {
        protocols,
        version,
        state: State::SendHeader {
            io: MessageIO::new(async_rwer),
        },
    }
}

/// A `Future` returned by [`dialer_select_proto`] which negotiates
/// a protocol iteratively by considering one protocol after the other.
#[pin_project::pin_project]
pub struct DialerSelectFuture {
    // TODO: It would be nice if eventually N = I::Item = Protocol.
    // protocols: alloc::vec::IntoIter<String>,
    protocols: VecDeque<String>,
    state: State,
    version: Version,
}

enum State {
    SendHeader { io: MessageIO },
    SendProtocol { io: MessageIO, protocol: String },
    FlushProtocol { io: MessageIO, protocol: String },
    AwaitProtocol { io: MessageIO, protocol: String },
    Done,
}

// todo: 实现MessageIO，其底层应该是一个异步的网络Socket才对
// todo: 我们不妨使用面向编译器的代码编写方法，先把业务逻辑写出来，再根据LSP报错增改代码
// todo: 好消息：async_net中的TcpSocket和UdpSocket全都有实现AsyncRead和AsyncWrite，可以考虑包装一下直接用

impl Future for DialerSelectFuture {
    type Output = Result<String, NegotiationError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        loop {
            match core::mem::replace(this.state, State::Done) {
                State::SendHeader { mut io } => {
                    match Pin::new(&mut io).poll_ready(cx)? {
                        Poll::Ready(()) => {}
                        Poll::Pending => {
                            *this.state = State::SendHeader { io };
                            return Poll::Pending;
                        }
                    }

                    let h = HeaderLine::from(*this.version);
                    if let Err(err) = Pin::new(&mut io).start_send(Message::Header(h)) {
                        return Poll::Ready(Err(From::from(err)));
                    }

                    let protocol = this.protocols.next().ok_or(NegotiationError::Failed)?;

                    // The dialer always sends the header and the first protocol
                    // proposal in one go for efficiency.
                    *this.state = State::SendProtocol { io, protocol };
                }
                State::SendProtocol { io, protocol } => {
                    match Pin::new(&mut io).poll_ready(cx)? {
                        Poll::Ready(()) => {}
                        Poll::Pending => {
                            *this.state = State::SendProtocol { io, protocol };
                            return Poll::Pending;
                        }
                    }

                    let p = Protocol::try_from(protocol.as_ref())?;
                    if let Err(err) = Pin::new(&mut io).start_send(Message::Protocol(p.clone())) {
                        return Poll::Ready(Err(From::from(err)));
                    }
                    tracing::debug!(protocol=%p, "Dialer: Proposed protocol");

                    if this.protocols.front().is_some() {
                        *this.state = State::FlushProtocol { io, protocol }
                    } else {
                        match this.version {
                            Version::V1 => *this.state = State::FlushProtocol { io, protocol },
                            // This is the only effect that `V1Lazy` has compared to `V1`:
                            // Optimistically settling on the only protocol that
                            // the dialer supports for this negotiation. Notably,
                            // the dialer expects a regular `V1` response.
                            Version::V1Lazy => {
                                tracing::debug!(protocol=%p, "Dialer: Expecting proposed protocol");
                                let hl = HeaderLine::from(Version::V1Lazy);
                                let io = Negotiated::expecting(io.into_reader(), p, Some(hl));
                                return Poll::Ready(Ok((protocol, io)));
                            }
                        }
                    }
                }
                State::FlushProtocol { io, protocol } => todo!(),
                State::AwaitProtocol { io, protocol } => todo!(),
                State::Done => todo!(),
            }
        }
        todo!()
    }
}
