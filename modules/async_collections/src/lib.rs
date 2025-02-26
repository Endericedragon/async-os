#![no_std]
extern crate alloc;

// Could use `async_std::collections::HashMap` outsides. Great!
pub use hashbrown::{HashMap, HashSet};
// Could use `async_std::collections::Vec` outsides. Great!
#[macro_use]
pub mod vec;
pub use vec::Vec;
// Could use `async_std::collections::BinaryHeap` outsides. Great!
pub use alloc::collections::BinaryHeap;
pub use alloc::collections::VecDeque;

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
