use core::pin::Pin;

use pin_project_lite::pin_project;

use super::Stream;
use super::{super::into_stream::IntoStream, AsyncStream};
use crate::stream::stream::map::Map;
use core::task::{Context, Poll};

pin_project! {
    /// A stream that maps each element to a stream, and yields the elements of the produced
    /// streams.
    ///
    /// This `struct` is created by the [`flat_map`] method on [`Stream`]. See its
    /// documentation for more.
    ///
    /// [`flat_map`]: trait.Stream.html#method.flat_map
    /// [`Stream`]: trait.Stream.html
    pub struct FlatMap<S, U, F> {
        #[pin]
        stream: Map<S, F>,
        #[pin]
        inner_stream: Option<U>,
    }
}

impl<S, U, F> FlatMap<S, U, F>
where
    S: AsyncStream,
    U: IntoStream,
    F: FnMut(S::Item) -> U,
{
    pub(super) fn new(stream: S, f: F) -> Self {
        Self {
            stream: stream.map(f),
            inner_stream: None,
        }
    }
}

impl<S, U, F> AsyncStream for FlatMap<S, U, F>
where
    S: AsyncStream,
    U: AsyncStream,
    F: FnMut(S::Item) -> U,
{
    type Item = U::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            if let Some(inner) = this.inner_stream.as_mut().as_pin_mut() {
                match core::task::ready!(inner.poll_next(cx)) {
                    item @ Some(_) => return Poll::Ready(item),
                    None => this.inner_stream.set(None),
                }
            }

            match core::task::ready!(this.stream.as_mut().poll_next(cx)) {
                inner @ Some(_) => this.inner_stream.set(inner.map(IntoStream::into_stream)),
                None => return Poll::Ready(None),
            }
        }
    }
}
