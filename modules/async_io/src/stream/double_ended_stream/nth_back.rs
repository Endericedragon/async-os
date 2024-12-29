use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::DoubleEndedStream;

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct NthBackFuture<'a, S> {
    stream: &'a mut S,
    n: usize,
}

impl<'a, S> NthBackFuture<'a, S> {
    pub(crate) fn new(stream: &'a mut S, n: usize) -> Self {
        NthBackFuture { stream, n }
    }
}

impl<'a, S> Future for NthBackFuture<'a, S>
where
    S: DoubleEndedStream + Sized + Unpin,
{
    type Output = Option<S::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let next = core::task::ready!(Pin::new(&mut *self.stream).poll_next_back(cx));
        match next {
            Some(v) => match self.n {
                0 => Poll::Ready(Some(v)),
                _ => {
                    self.n -= 1;
                    Poll::Pending
                }
            },
            None => Poll::Ready(None),
        }
    }
}
