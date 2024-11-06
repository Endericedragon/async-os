extern crate alloc;

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
// use futures_timer::Delay;

// async fn test_async(sec: u64) {
//     println!("Waiting for {} sec...", sec);
//     Delay::new(core::time::Duration::from_secs(sec)).await;
//     println!("{} second(s) elapsed.", sec);
// }

use super::collections::VecDeque;
use spin::Mutex;
pub struct WeirdExecutor<'a> {
    queue: Mutex<VecDeque<(Pin<Box<dyn Future<Output = ()>>>, &'a Waker, Context<'a>)>>,
}

impl<'a> WeirdExecutor<'a> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn add_task(&mut self, task: impl Future<Output = ()> + 'static) {
        let waker = Waker::noop();
        let cx = Context::from_waker(&waker);
        self.queue.lock().push_back((Box::pin(task), waker, cx));
    }

    pub fn run_tasks(&mut self) {
        let mut queue = self.queue.lock();
        while let Some(task_info) = queue.pop_front() {
            let (mut task, waker, mut cx) = task_info;
            if let Poll::Pending = task.as_mut().poll(&mut cx) {
                queue.push_back((task, waker, cx));
            }
        }
    }
}

// fn a_small_test() {
//     // 执行一个协程的简单测试
//     let waker = Waker::noop();
//     let mut context = Context::from_waker(&waker);

//     let mut task = Box::pin(test_async(1));
//     while let Poll::Pending = task.as_mut().poll(&mut context) {}
//     println!("All tests are done.");

//     // 奇怪执行器测试
//     let mut executor = WeirdExecutor::new();
//     executor.add_task(test_async(2));
//     executor.add_task(test_async(1));
//     executor.run_tasks();
//     println!("All tests are done.");
// }
