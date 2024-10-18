use async_std::collections::HashMap;
use core::future::Future;
use core::task::{ready, Context, Poll};
use core::time::Duration;

use futures_util::future::BoxFuture;

use crate::{FuturesMap, PushError, Timeout};

/// Represents a list of tuples of a [Future] and an associated piece of data.
///
/// Each future must finish within the specified time and the list never outgrows its capacity.
pub struct FuturesTupleSet<O, D> {
    id: u32,
    inner: FuturesMap<u32, O>,
    data: HashMap<u32, D>,
}

impl<O, D> FuturesTupleSet<O, D> {
    pub fn new(timeout: Duration, capacity: usize) -> Self {
        Self {
            id: 0,
            inner: FuturesMap::new(timeout, capacity),
            data: HashMap::new(),
        }
    }
}

impl<O, D> FuturesTupleSet<O, D>
where
    O: 'static,
{
    /// Push a future into the list.
    ///
    /// This method adds the given future to the list.
    /// If the length of the list is equal to the capacity, this method returns a error that contains the passed future.
    /// In that case, the future is not added to the set.
    pub fn try_push<F>(&mut self, future: F, data: D) -> Result<(), (BoxFuture<O>, D)>
    where
        F: Future<Output = O> + Send + 'static,
    {
        self.id = self.id.wrapping_add(1);

        match self.inner.try_push(self.id, future) {
            Ok(()) => {}
            Err(PushError::BeyondCapacity(w)) => return Err((w, data)),
            Err(PushError::Replaced(_)) => unreachable!("we never reuse IDs"),
        }
        self.data.insert(self.id, data);

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn poll_ready_unpin(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.inner.poll_ready_unpin(cx)
    }

    pub fn poll_unpin(&mut self, cx: &mut Context<'_>) -> Poll<(Result<O, Timeout>, D)> {
        let (id, res) = ready!(self.inner.poll_unpin(cx));
        let data = self.data.remove(&id).expect("must have data for future");

        Poll::Ready((res, data))
    }
}
