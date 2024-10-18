use core::future::Future;
use core::hash::Hash;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use core::time::Duration;
use core::{future, mem};

use futures_timer::Delay;
use futures_util::future::BoxFuture;
use futures_util::stream::FuturesUnordered;
use futures_util::{FutureExt, StreamExt};

use crate::{PushError, Timeout};

/// Represents a map of [`Future`]s.
///
/// Each future must finish within the specified time and the map never outgrows its capacity.
pub struct FuturesMap<ID, O> {
    timeout: Duration,
    capacity: usize,
    inner: FuturesUnordered<TaggedFuture<ID, TimeoutFuture<BoxFuture<'static, O>>>>,
    empty_waker: Option<Waker>,
    full_waker: Option<Waker>,
}

impl<ID, O> FuturesMap<ID, O> {
    pub fn new(timeout: Duration, capacity: usize) -> Self {
        Self {
            timeout,
            capacity,
            inner: Default::default(),
            empty_waker: None,
            full_waker: None,
        }
    }
}

impl<ID, O> FuturesMap<ID, O>
where
    ID: Clone + Hash + Eq + Send + Unpin + 'static,
    O: 'static,
{
    /// Push a future into the map.
    ///
    /// This method inserts the given future with defined `future_id` to the set.
    /// If the length of the map is equal to the capacity, this method returns [PushError::BeyondCapacity],
    /// that contains the passed future. In that case, the future is not inserted to the map.
    /// If a future with the given `future_id` already exists, then the old future will be replaced by a new one.
    /// In that case, the returned error [PushError::Replaced] contains the old future.
    pub fn try_push<F>(&mut self, future_id: ID, future: F) -> Result<(), PushError<BoxFuture<O>>>
    where
        F: Future<Output = O> + Send + 'static,
    {
        if self.inner.len() >= self.capacity {
            return Err(PushError::BeyondCapacity(future.boxed()));
        }

        if let Some(waker) = self.empty_waker.take() {
            waker.wake();
        }

        let old = self.remove(future_id.clone());
        self.inner.push(TaggedFuture {
            tag: future_id,
            inner: TimeoutFuture {
                inner: future.boxed(),
                timeout: Delay::new(self.timeout),
                cancelled: false,
            },
        });
        match old {
            None => Ok(()),
            Some(old) => Err(PushError::Replaced(old)),
        }
    }

    pub fn remove(&mut self, id: ID) -> Option<BoxFuture<'static, O>> {
        let tagged = self.inner.iter_mut().find(|s| s.tag == id)?;

        let inner = mem::replace(&mut tagged.inner.inner, future::pending().boxed());
        tagged.inner.cancelled = true;

        Some(inner)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[allow(unknown_lints, clippy::needless_pass_by_ref_mut)] // &mut Context is idiomatic.
    pub fn poll_ready_unpin(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.inner.len() < self.capacity {
            return Poll::Ready(());
        }

        self.full_waker = Some(cx.waker().clone());

        Poll::Pending
    }

    pub fn poll_unpin(&mut self, cx: &mut Context<'_>) -> Poll<(ID, Result<O, Timeout>)> {
        loop {
            let maybe_result = futures_util::ready!(self.inner.poll_next_unpin(cx));

            match maybe_result {
                None => {
                    self.empty_waker = Some(cx.waker().clone());
                    return Poll::Pending;
                }
                Some((id, Ok(output))) => return Poll::Ready((id, Ok(output))),
                Some((id, Err(TimeoutError::Timeout))) => {
                    return Poll::Ready((id, Err(Timeout::new(self.timeout))))
                }
                Some((_, Err(TimeoutError::Cancelled))) => continue,
            }
        }
    }
}

struct TimeoutFuture<F> {
    inner: F,
    timeout: Delay,

    cancelled: bool,
}

impl<F> Future for TimeoutFuture<F>
where
    F: Future + Unpin,
{
    type Output = Result<F::Output, TimeoutError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.cancelled {
            return Poll::Ready(Err(TimeoutError::Cancelled));
        }

        if self.timeout.poll_unpin(cx).is_ready() {
            return Poll::Ready(Err(TimeoutError::Timeout));
        }

        self.inner.poll_unpin(cx).map(Ok)
    }
}

enum TimeoutError {
    Timeout,
    Cancelled,
}

struct TaggedFuture<T, F> {
    tag: T,
    inner: F,
}

impl<T, F> Future for TaggedFuture<T, F>
where
    T: Clone + Unpin,
    F: Future + Unpin,
{
    type Output = (T, F::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let output = futures_util::ready!(self.inner.poll_unpin(cx));

        Poll::Ready((self.tag.clone(), output))
    }
}
