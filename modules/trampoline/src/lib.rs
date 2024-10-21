#![no_std]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(fn_align)]
#![feature(doc_cfg)]

extern crate alloc;
#[macro_use]
extern crate log;

mod arch;
mod executor_api;
mod fs_api;
mod init_api;
mod task_api;
mod trap_api;

use core::task::{Context, Poll};
pub use fs_api::fs_init;
use alloc::sync::Arc;
pub use arch::init_interrupt;
pub use init_api::*;
pub use taskctx::TrapFrame;

use riscv::register::scause::{self, Trap};
pub use task_api::*;
use taskctx::{CurrentTask, TaskState, TrapStatus};
pub use trap_api::*;
pub use executor_api::*;
/// 进入 Trampoline 的方式：
///   1. 初始化后函数调用：没有 Trap，但存在就绪任务
///   2. 内核发生 Trap：存在任务被打断（CurrentTask 不为空），或者没有任务被打断（CurrentTask 为空）
///   3. 用户态发生 Trap：任务被打断，CurrentTask 不为空
/// 
/// 内核发生 Trap 时，将 TrapFrame 保存在内核栈上
/// 在用户态发生 Trap 时，将 TrapFrame 直接保存在任务控制块中，而不是在内核栈上
#[no_mangle]
pub fn trampoline(tf: &mut TrapFrame, has_trap: bool, from_user: bool) {
    loop {
        if !from_user && has_trap {
            // 在内核中发生了 Trap，只处理中断，目前还不支持抢占，因此是否有任务被打断是不做处理的
            // warn!("here");
            let scause = scause::read();
            match scause.cause() {
                Trap::Interrupt(_interrupt) => {
                    handle_irq(tf.get_scause_code(), tf)
                },
                Trap::Exception(e) => {
                    panic!("Unsupported kernel trap {:?} @ {:#x}:\n{:#x?}", e, tf.sepc, tf)
                },
            }
            return;
        } else {
            // 用户态发生了 Trap 或者需要调度
            if let Some(task) = CurrentTask::try_get().or_else(|| {
                if let Some(task) = CurrentExecutor::get().pick_next_task() {
                    unsafe { CurrentTask::init_current(task); }
                    Some(CurrentTask::get())
                } else {
                    None
                }
            }) {
                run_task(task.as_task_ref());
            } else {
                // warn!("no task, change executor or wfi");
                // 如果当前的 Executor 中没有任务了，则切换回内核的 Executor
                turn_to_kernel_executor();
                // 没有就绪任务，等待中断
                #[cfg(feature = "irq")]
                async_axhal::arch::wait_for_irqs();
            }
        }
    }
}

pub fn run_task(task: &TaskRef) {
    let waker = taskctx::waker_from_task(task);
    let cx = &mut Context::from_waker(&waker);
    #[cfg(feature = "preempt")]
    restore_from_preempt_ctx(&task);
    // warn!("run task {} count {}", task.id_name(), Arc::strong_count(task));
    let res = task.get_fut().as_mut().poll(cx);
    match res {
        Poll::Ready(exit_code) => {
            debug!("task exit: {}, exit_code={}", task.id_name(), exit_code);
            task.set_state(TaskState::Exited);
            task.set_exit_code(exit_code);
            task.notify_waker_for_exit();
            if task.is_init() {
                assert!(Arc::strong_count(&task) == 1, "count {}", Arc::strong_count(&task));
                async_axhal::misc::terminate();
            }
            warn!("task {} count {}", task.id_name(), Arc::strong_count(&task));
            CurrentTask::clean_current();
        },
        Poll::Pending => {
            if let Some(tf) = task.utrap_frame() {
                if tf.trap_status == TrapStatus::Done {
                    tf.kernel_sp = taskctx::current_stack_top();
                    tf.scause = 0;
                    error!("return to user");
                    // 这里不能打开中断
                    async_axhal::arch::disable_irqs();
                    unsafe { tf.user_return(); }
                }
            }     
            // error!("task pending");
            CurrentTask::clean_current_without_drop();
        }
    }
}

