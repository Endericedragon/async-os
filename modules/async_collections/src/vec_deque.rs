use core::iter::Iterator;

use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct VecDeque<T> {
    inner: alloc::collections::VecDeque<T>,
}

impl<T> VecDeque<T> {
    pub fn new() -> Self {
        Self {
            inner: alloc::collections::VecDeque::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: alloc::collections::VecDeque::with_capacity(capacity),
        }
    }
}

impl<T> From<super::Vec<T>> for VecDeque<T> {
    fn from(value: super::Vec<T>) -> Self {
        Self {
            inner: alloc::collections::VecDeque::from(value),
        }
    }
}

impl<T> Deref for VecDeque<T> {
    type Target = alloc::collections::VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for VecDeque<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Iterator for VecDeque<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }
}

#[macro_export]
macro_rules! vec_deque {
    () => {{
        VecDeque::new()
    }};

    ($($x:expr),+) => {{
        let mut res = VecDeque::new();
        $(res.push_back($x);)+
        res
    }}
}
