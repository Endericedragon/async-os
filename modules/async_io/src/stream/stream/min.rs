use core::cmp::{Ord, Ordering};
use core::future::Future;
use core::pin::Pin;

use pin_project_lite::pin_project;

use super::AsyncStream;
use core::task::{Context, Poll};

pin_project! {
    #[doc(hidden)]
    #[allow(missing_debug_implementations)]
    pub struct MinFuture<S, T> {
        #[pin]
        stream: S,
        min: Option<T>,
    }
}

impl<S, T> MinFuture<S, T> {
    pub(super) fn new(stream: S) -> Self {
        Self { stream, min: None }
    }
}

impl<S> Future for MinFuture<S, S::Item>
where
    S: AsyncStream,
    S::Item: Ord,
{
    type Output = Option<S::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let next = core::task::ready!(this.stream.poll_next(cx));

        match next {
            Some(new) => {
                match this.min.take() {
                    None => *this.min = Some(new),

                    Some(old) => match new.cmp(&old) {
                        Ordering::Less => *this.min = Some(new),
                        _ => *this.min = Some(old),
                    },
                }
                Poll::Pending
            }
            None => Poll::Ready(this.min.take()),
        }
    }
}
