use core::cmp::Ordering;
use core::future::Future;
use core::pin::Pin;

use pin_project_lite::pin_project;

use super::AsyncStream;
use core::task::{Context, Poll};

pin_project! {
    #[doc(hidden)]
    #[allow(missing_debug_implementations)]
    pub struct MaxByFuture<S, F, T> {
        #[pin]
        stream: S,
        compare: F,
        max: Option<T>,
    }
}

impl<S, F, T> MaxByFuture<S, F, T> {
    pub(super) fn new(stream: S, compare: F) -> Self {
        Self {
            stream,
            compare,
            max: None,
        }
    }
}

impl<S, F> Future for MaxByFuture<S, F, S::Item>
where
    S: AsyncStream,
    F: FnMut(&S::Item, &S::Item) -> Ordering,
{
    type Output = Option<S::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let next = core::task::ready!(this.stream.poll_next(cx));

        match next {
            Some(new) => {
                match this.max.take() {
                    None => *this.max = Some(new),
                    Some(old) => match (this.compare)(&new, &old) {
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
