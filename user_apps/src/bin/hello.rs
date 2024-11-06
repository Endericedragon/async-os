#![no_std]
#![no_main]

#[macro_use]
extern crate user_apps;

use user_apps::{ctypes, get_time_of_day, my_std};

async fn test() {
    let mut tv = ctypes::TimeVal::default();
    get_time_of_day(&mut tv);
    let start_time = tv.turn_to_nanos();
    loop {
        get_time_of_day(&mut tv);
        let current_time = tv.turn_to_nanos();
        if current_time - start_time > 1_000_000_000 {
            break;
        }
    }
}

#[no_mangle]
pub fn main() -> i32 {
    println!("Hello world 2!");
    let mut tv = ctypes::TimeVal::default();
    get_time_of_day(&mut tv);
    println!("{:?}", tv);

    println!("Testing async...");
    let mut exec = my_std::weird_executor::WeirdExecutor::new();
    exec.add_task(test());
    exec.run_tasks();
    println!("Testing async done.");

    0
}
