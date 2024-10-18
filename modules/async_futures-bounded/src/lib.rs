#![no_std]
#![feature(doc_cfg)]
#![feature(async_iterator)]
// Introduce this to use core::error
#![feature(error_in_core)]

extern crate alloc;

mod futures_map;
mod futures_set;
mod futures_tuple_set;
mod stream_map;
mod stream_set;

pub use futures_map::FuturesMap;
pub use futures_set::FuturesSet;
pub use futures_tuple_set::FuturesTupleSet;
pub use stream_map::StreamMap;
pub use stream_set::StreamSet;

use alloc::fmt;
use alloc::fmt::Formatter;
use core::time::Duration;

/// A future failed to complete within the given timeout.
#[derive(Debug)]
pub struct Timeout {
    limit: Duration,
}

impl Timeout {
    fn new(duration: Duration) -> Self {
        Self { limit: duration }
    }
}

impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "future failed to complete within {:?}", self.limit)
    }
}

/// Error of a future pushing
#[derive(PartialEq, Debug)]
pub enum PushError<T> {
    /// The length of the set is equal to the capacity
    BeyondCapacity(T),
    /// The map already contained an item with this key.
    ///
    /// The old item is returned.
    Replaced(T),
}

impl core::error::Error for Timeout {}