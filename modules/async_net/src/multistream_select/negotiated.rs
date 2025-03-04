use super::message_io::MessageIO;
use super::protocol::{HeaderLine, Protocol};
use super::protocol::{Message, ProtocolError};
use async_io::{AsyncRead, AsyncWrite};
use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use pin_project::pin_project;

pub enum NegotiationError {
    ProtocolError(ProtocolError),
    Failed,
}

impl From<ProtocolError> for NegotiationError {
    fn from(value: ProtocolError) -> Self {
        Self::ProtocolError(value)
    }
}

/// An I/O stream that has settled on an (application-layer) protocol to use.
///
/// A `Negotiated` represents an I/O stream that has _settled_ on a protocol
/// to use. In particular, it is not implied that all of the protocol negotiation
/// frames have yet been sent and / or received, just that the selected protocol
/// is fully determined. This is to allow the last protocol negotiation frames
/// sent by a peer to be combined in a single write, possibly piggy-backing
/// data from the negotiated protocol on top.
///
/// Reading from a `Negotiated` I/O stream that still has pending negotiation
/// protocol data to send implicitly triggers flushing of all yet unsent data.
#[pin_project]
#[derive(Debug)]
pub struct Negotiated<TInner> {
    #[pin]
    state: State<TInner>,
}

/// The states of a `Negotiated` I/O stream.
#[pin_project(project = StateProj)]
#[derive(Debug)]
enum State<R> {
    /// In this state, a `Negotiated` is still expecting to
    /// receive confirmation of the protocol it has optimistically
    /// settled on.
    Expecting {
        /// The underlying I/O stream.
        #[pin]
        io: MessageIO,
        /// The expected negotiation header/preamble (i.e. multistream-select version),
        /// if one is still expected to be received.
        header: Option<HeaderLine>,
        /// The expected application protocol (i.e. name and version).
        protocol: Protocol,
    },

    /// In this state, a protocol has been agreed upon and I/O
    /// on the underlying stream can commence.
    Completed {
        #[pin]
        io: R,
    },

    /// Temporary state while moving the `io` resource from
    /// `Expecting` to `Completed`.
    Invalid,
}

/// A `Future` that waits on the completion of protocol negotiation.
#[derive(Debug)]
pub struct NegotiatedComplete<TInner> {
    inner: Option<Negotiated<TInner>>,
}

impl<TInner> Future for NegotiatedComplete<TInner>
where
    // `Unpin` is required not because of implementation details but because we produce the
    // `Negotiated` as the output of the future.
    TInner: AsyncRead + AsyncWrite + Unpin,
{
    type Output = Result<Negotiated<TInner>, NegotiationError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut io = self
            .inner
            .take()
            .expect("NegotiatedFuture called after completion.");
        match Negotiated::poll(Pin::new(&mut io), cx) {
            Poll::Pending => {
                self.inner = Some(io);
                Poll::Pending
            }
            Poll::Ready(Ok(())) => Poll::Ready(Ok(io)),
            Poll::Ready(Err(err)) => {
                self.inner = Some(io);
                Poll::Ready(Err(err))
            }
        }
    }
}

impl<TInner> Negotiated<TInner> {
    /// Creates a `Negotiated` in state [`State::Completed`].
    pub(crate) fn completed(io: TInner) -> Self {
        Negotiated {
            state: State::Completed { io },
        }
    }

    /// Creates a `Negotiated` in state [`State::Expecting`] that is still
    /// expecting confirmation of the given `protocol`.
    pub(crate) fn expecting(io: MessageIO, protocol: Protocol, header: Option<HeaderLine>) -> Self {
        Negotiated {
            state: State::Expecting {
                io,
                protocol,
                header,
            },
        }
    }

    /// Polls the `Negotiated` for completion.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), NegotiationError>>
    where
        TInner: AsyncRead + AsyncWrite + Unpin,
    {
        // Flush any pending negotiation data.
        match self.as_mut().poll_flush(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => {
                // If the remote closed the stream, it is important to still
                // continue reading the data that was sent, if any.
                if e.kind() != async_io::Error::WriteZero {
                    return Poll::Ready(Err(e.into()));
                }
            }
        }

        let mut this = self.project();

        if let StateProj::Completed { .. } = this.state.as_mut().project() {
            return Poll::Ready(Ok(()));
        }

        // Read outstanding protocol negotiation messages.
        loop {
            match mem::replace(&mut *this.state, State::Invalid) {
                State::Expecting {
                    mut io,
                    header,
                    protocol,
                } => {
                    let msg = match Pin::new(&mut io).poll_next(cx)? {
                        Poll::Ready(Some(msg)) => msg,
                        Poll::Pending => {
                            *this.state = State::Expecting {
                                io,
                                header,
                                protocol,
                            };
                            return Poll::Pending;
                        }
                        Poll::Ready(None) => {
                            return Poll::Ready(Err(ProtocolError::IoError(
                                async_io::Error::UnexpectedEof.into(),
                            )
                            .into()));
                        }
                    };

                    if let Message::Header(h) = &msg {
                        if Some(h) == header.as_ref() {
                            *this.state = State::Expecting {
                                io,
                                protocol,
                                header: None,
                            };
                            continue;
                        }
                    }

                    if let Message::Protocol(p) = &msg {
                        if p.as_ref() == protocol.as_ref() {
                            tracing::debug!(protocol=%p, "Negotiated: Received confirmation for protocol");
                            *this.state = State::Completed { io };
                            return Poll::Ready(Ok(()));
                        }
                    }

                    return Poll::Ready(Err(NegotiationError::Failed));
                }

                _ => panic!("Negotiated: Invalid state"),
            }
        }
    }

    /// Returns a [`NegotiatedComplete`] future that waits for protocol
    /// negotiation to complete.
    pub fn complete(self) -> NegotiatedComplete<TInner> {
        NegotiatedComplete { inner: Some(self) }
    }
}
