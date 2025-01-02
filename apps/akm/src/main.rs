#![no_std]
#![no_main]

#[macro_use]
extern crate async_std;

use ak_futures_timer;

#[async_std::async_main]
async fn main() -> isize {
    println!("Testing ak_futures-timer...");

    let delay = ak_futures_timer::Delay::new(async_std::time::Duration::from_secs(2));
    delay.await;

    println!("Timer finished after 2 seconds!");

    0
}
