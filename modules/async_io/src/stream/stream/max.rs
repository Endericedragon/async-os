use core::cmp::{Ord, Ordering};
use core::future::Future;
use core::pin::Pin;

use pin_project_lite::pin_project;

use super::AsyncStream;
use core::task::{Context, Poll};

pin_project! {
    #[doc(hidden)]
    #[allow(missing_debug_implementations)]
    pub struct MaxFuture<S, T> {
        #[pin]
        stream: S,
        max: Option<T>,
    }
}

impl<S, T> MaxFuture<S, T> {
    pub(super) fn new(stream: S) -> Self {
        Self { stream, max: None }
    }
}

impl<S> Future for MaxFuture<S, S::Item>
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
                match this.max.take() {
                    None => *this.max = Some(new),

                    Some(old) => match new.cmp(&old) {
                        Ordering::Greater => *this.max = Some(new),
                        _ => *this.max = Some(old),
                    },
                }
                Poll::Pending
            }
            None => Poll::Ready(this.max.take()),
        }
    }
}
