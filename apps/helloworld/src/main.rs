#![no_std]
#![no_main]

extern crate async_std as std;
use async_std::sync::Mutex;

static A: Mutex<i32> = Mutex::new(23);

use core::time::Duration;

#[async_std::async_main]
async fn main() -> i32 {
    async_test().await;

    0
}

async fn async_test() {
    let mut b = A.lock().await;
    async_std::println!("Mutex locked: {:?}", *b);
    *b = 34;
    // drop(b);
    async_std::println!("Creating thread join handle...");
    let j = async_std::thread::spawn(async {
        let a = A.lock().await;
        async_std::println!("spawn Mutex locked: {:?}", *a);
        32
    })
    .join();
    async_std::thread::sleep(Duration::from_secs(1)).await;
    async_std::println!("Variable b dropped.");
    drop(b);
    let res = j.await.unwrap();
    async_std::println!("res {}", res);
    async_std::thread::sleep(Duration::from_secs(1)).await;
    for i in 0..100 {
        async_std::println!("for test preempt {}", i);
    }
}
