use core::future::Future;
use core::mem;
use core::pin::Pin;

use crate::{self as io, AsyncRead};
use core::task::{Context, Poll};

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct ReadExactFuture<'a, T: Unpin + ?Sized> {
    pub(crate) reader: &'a mut T,
    pub(crate) buf: &'a mut [u8],
}

impl<T: AsyncRead + Unpin + ?Sized> Future for ReadExactFuture<'_, T> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader, buf } = &mut *self;

        while !buf.is_empty() {
            let n = core::task::ready!(Pin::new(&mut *reader).read(cx, buf))?;
            let (_, rest) = mem::replace(buf, &mut []).split_at_mut(n);
            *buf = rest;

            if n == 0 {
                return Poll::Ready(Err(io::Error::UnexpectedEof.into()));
            }
        }

        Poll::Ready(Ok(()))
    }
}
