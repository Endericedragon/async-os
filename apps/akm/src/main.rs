#![no_std]
#![no_main]

#[macro_use]
extern crate async_std;

use ak_futures_timer;

mod tests_for_futures_map;

#[async_std::async_main]
async fn main() -> isize {
    println!("Testing ak_futures-timer...");

    let delay = ak_futures_timer::Delay::new(async_std::time::Duration::from_secs(1));
    delay.await;

    println!("Timer finished after 1 seconds!");

    println!("Testing ak_futures-bounded...");

    tests_for_futures_map::cannot_push_more_than_capacity_tasks();
    tests_for_futures_map::cannot_push_the_same_id_few_times();
    tests_for_futures_map::futures_timeout().await;
    tests_for_futures_map::resources_of_removed_future_are_cleaned_up();
    tests_for_futures_map::replaced_pending_future_is_polled().await;
    tests_for_futures_map::backpressure().await;

    println!("All tests passed!");
    0
}
