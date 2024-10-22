#![no_std]
#![no_main]

// use async_axsync_with_sync::Mutex;
use async_futures_timer::Delay;
use async_std::{
    collections::{BinaryHeap, HashMap},
    println, println_sync,
};
use core::time::Duration;

#[async_std::async_main]
async fn main() -> i32 {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 2);
    map.insert(3, 4);
    println_sync!("The map contains:");
    for (k, v) in map.iter() {
        async_std::println!("  {} => {}", k, v);
    }

    let mut heap = BinaryHeap::new();
    for i in 1..=10 {
        heap.push(i);
    }

    for i in (1..=10).rev() {
        let val = heap.pop();
        // println!("Popped: {:?}", val);
        assert_eq!(val, Some(i));
    }

    assert_eq!(heap.pop(), None);

    let delay_time_1 = 2;
    let delay_1 = Delay::new(Duration::from_secs(delay_time_1));
    let delay_time_2 = 6;
    Delay::new(Duration::from_secs(delay_time_2)).await;
    println!("Waiting for {} second...", delay_time_1);
    delay_1.await;
    println!("Done!");

    // let mutex = Mutex::new(32);
    // println_sync!("Can you reach the finals?");
    // let rua = mutex.lock();
    // println!("{}", *rua);
    // println_sync!("Can you reach the finals?");

    0
}
