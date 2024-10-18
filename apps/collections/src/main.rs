#![no_std]
#![no_main]

extern crate async_std;
use async_std::{collections::{BinaryHeap, HashMap}, println};

#[async_std::async_main]
async fn main() -> i32 {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 2);
    map.insert(3, 4);
    println!("The map contains:");
    for (k, v) in map.iter() {
        async_std::println!("  {} => {}", k, v);
    }

    let mut heap = BinaryHeap::new();
    for i in 1..=10 {
        heap.push(i);
    }

    for i in (1..=10).rev() {
        let val = heap.pop();
        println!("Popped: {:?}", val);
        assert_eq!(val, Some(i));
    }

    assert_eq!(heap.pop(), None);

    0
}
