use core::future::Future;
use core::pin::Pin;

use alloc::vec::Vec;

use crate::{self as io, AsyncRead};
use core::task::{Context, Poll};

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct ReadToEndFuture<'a, T: Unpin + ?Sized> {
    pub(crate) reader: &'a mut T,
    pub(crate) buf: &'a mut Vec<u8>,
    pub(crate) start_len: usize,
}

impl<T: AsyncRead + Unpin + ?Sized> Future for ReadToEndFuture<'_, T> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            reader,
            buf,
            start_len,
        } = &mut *self;
        read_to_end_internal(Pin::new(reader), cx, buf, *start_len)
    }
}

// This uses an adaptive system to extend the vector when it fills. We want to
// avoid paying to allocate and zero a huge chunk of memory if the reader only
// has 4 bytes while still making large reads if the reader does have a ton
// of data to return. Simply tacking on an extra DEFAULT_BUF_SIZE space every
// time is 4,500 times (!) slower than this if the reader has a very small
// amount of data to return.
//
// Because we're extending the buffer with uninitialized data for trusted
// readers, we need to make sure to truncate that if any of this panics.
pub fn read_to_end_internal<R: AsyncRead + ?Sized>(
    mut rd: Pin<&mut R>,
    cx: &mut Context<'_>,
    buf: &mut Vec<u8>,
    start_len: usize,
) -> Poll<io::Result<usize>> {
    let mut probe = [0u8; 32];
    loop {
        match core::task::ready!(rd.as_mut().read(cx, &mut probe)) {
            Ok(0) => return Poll::Ready(Ok(buf.len() - start_len)),
            Ok(n) => buf.extend_from_slice(&probe[..n]),
            Err(e) => return Poll::Ready(Err(e)),
        }
    }
    // struct Guard<'a> {
    //     buf: &'a mut Vec<u8>,
    //     len: usize,
    // }

    // impl Drop for Guard<'_> {
    //     fn drop(&mut self) {
    //         unsafe {
    //             self.buf.set_len(self.len);
    //         }
    //     }
    // }

    // let mut g = Guard {
    //     len: buf.len(),
    //     buf,
    // };
    // let ret;
    // loop {
    //     if g.len == g.buf.len() {
    //         unsafe {
    //             g.buf.reserve(32);
    //             let capacity = g.buf.capacity();
    //             g.buf.set_len(capacity);
    //             super::initialize(&rd, &mut g.buf[g.len..]);
    //         }
    //     }

    //     match core::task::ready!(rd.as_mut().read(cx, &mut g.buf[g.len..])) {
    //         Ok(0) => {
    //             ret = Poll::Ready(Ok(g.len - start_len));
    //             break;
    //         }
    //         Ok(n) => g.len += n,
    //         Err(e) => {
    //             ret = Poll::Ready(Err(e));
    //             break;
    //         }
    //     }
    // }

    // ret
}
