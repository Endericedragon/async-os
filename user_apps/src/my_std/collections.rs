pub use alloc::vec::Vec;
#[allow(unused)]
pub use hashbrown::{HashMap, HashSet};

use core::option::Option;

macro_rules! vec {
    () => {{
        Vec::new()
    }};
    ($($x:expr),*) => {{
        let mut v = Vec::new();

        $(
            v.push($x);
        )*

        v
    }}
}

#[allow(unused)]
pub struct VecDeque<T> {
    inner: Vec<Option<T>>,
    front: usize,
    back: usize,
}

#[allow(unused)]
impl<T> VecDeque<T> {
    pub fn new() -> Self {
        let inner = vec![None, None];
        Self {
            inner,
            front: 0,
            back: 0,
        }
    }

    pub fn is_full(&self) -> bool {
        self.inner[self.back].is_some()
    }

    #[inline]
    pub fn inflate(&mut self) {
        if !self.is_full() {
            return;
        }

        let original_len = self.inner.len();
        let mut new_inner = Vec::<Option<T>>::with_capacity(original_len * 2);
        for i in 0..original_len {
            new_inner.push(self.inner[i + self.front].take());
        }
        for _ in original_len..original_len * 2 {
            new_inner.push(None);
        }
        self.inner = new_inner;
        self.front = 0;
        self.back = original_len;
    }

    pub fn push_back(&mut self, value: T) {
        self.inflate();
        self.inner[self.back] = Some(value);
        self.back = (self.back + 1) % self.inner.len();
    }

    pub fn peek(&self) -> Option<&T> {
        self.inner[self.front].as_ref()
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let res = self.inner[self.front].take();
        self.front = (self.front + 1) % self.inner.len();
        // todo: shrink if necessary
        res
    }
}
