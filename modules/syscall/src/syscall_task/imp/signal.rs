//! 支持信号相关的 syscall
//! 与信号处理相关的系统调用

use axhal::cpu::this_cpu_id;
// use axlog::{debug, info};
use axsignal::signal_no::SignalNo;
use axsignal::{action::SigAction, ucontext::SignalStack};
use executor::{current_executor, current_task, yield_now};

use crate::{SigMaskFlag, SyscallError, SyscallResult, SIGSET_SIZE_IN_BYTE};

/// # Arguments
/// * `signum` - usize
/// * `action` - *const SigAction
/// * `old_action` - *mut SigAction
pub async fn syscall_sigaction(args: [usize; 6]) -> SyscallResult {
    let signum = args[0];
    let action = args[1] as *const SigAction;
    let old_action = args[2] as *mut SigAction;
    info!(
        "signum: {}, action: {:X}, old_action: {:X}",
        signum, action as usize, old_action as usize
    );
    if signum == SignalNo::SIGKILL as usize || signum == SignalNo::SIGSTOP as usize {
        // 特殊参数不能被覆盖
        return Err(SyscallError::EPERM);
    }

    let current_process = current_executor().await;
    let mut signal_modules = current_process.signal_modules.lock().await;
    let signal_module = signal_modules
        .get_mut(&current_task().id().as_u64())
        .unwrap();
    let mut signal_handler = signal_module.signal_handler.lock().await;
    let old_address = old_action as usize;

    if old_address != 0 {
        // old_address非零说明要求写入到这个地址
        // 此时要检查old_address是否在某一个段中
        if current_process
            .manual_alloc_for_lazy(old_address.into())
            .await
            .is_err()
        {
            // 无法分配
            return Err(SyscallError::EPERM);
        }

        // 将原有的action存储到old_address
        unsafe {
            *old_action = *signal_handler.get_action(signum);
        }
    }

    let new_address = action as usize;
    if new_address != 0 {
        if current_process
            .manual_alloc_for_lazy(new_address.into())
            .await
            .is_err()
        {
            // 无法分配
            return Err(SyscallError::EPERM);
        }
        unsafe { signal_handler.set_action(signum, action) };
    }
    Ok(0)
}

/// 实现sigsuspend系统调用
/// TODO: 这里实现的似乎和文档有出入，应该有 BUG
/// # Arguments
/// * `mask` - *const usize
pub async fn syscall_sigsuspend(args: [usize; 6]) -> SyscallResult {
    let mask = args[0] as *const usize;
    let process = current_executor().await;
    if process
        .manual_alloc_for_lazy((mask as usize).into())
        .await
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }
    let mut signal_modules = process.signal_modules.lock().await;

    let signal_module = signal_modules
        .get_mut(&current_task().id().as_u64())
        .unwrap();
    // 设置新的掩码
    if signal_module.last_trap_frame_for_signal.is_some() {
        // 信号嵌套的情况下触发这个调用
        return Err(SyscallError::EINTR);
    }
    signal_module.signal_set.mask = unsafe { *mask };
    drop(signal_modules);
    loop {
        let mut signal_modules = process.signal_modules.lock().await;
        let signal_module = signal_modules
            .get_mut(&current_task().id().as_u64())
            .unwrap();

        if signal_module.signal_set.find_signal().is_none() {
            // 记得释放锁
            drop(signal_modules);
            yield_now().await;
            // yield_now_task();
            if process.have_signals().await.is_some() {
                return Err(SyscallError::EINTR);
            }
        } else {
            // 说明来了一个信号
            break;
        }
    }
    Err(SyscallError::EINTR)
}

/// Note: It can only be called by the signal processing function during signal processing.
pub async fn syscall_sigreturn() -> SyscallResult {
    Ok(executor::signal::signal_return().await)
}

/// # Arguments
/// * `flag` - SigMaskFlag
/// * `new_mask` - *const usize
/// * `old_mask` - *mut usize
/// * `sigsetsize` - usize, specifies the size in bytes of the signal sets in set and oldset, which is equal to sizeof(kernel_sigset_t)
pub async fn syscall_sigprocmask(args: [usize; 6]) -> SyscallResult {
    let flag = SigMaskFlag::from(args[0]);
    let new_mask = args[1] as *const usize;
    let old_mask = args[2] as *mut usize;
    let sigsetsize = args[3];
    if sigsetsize != SIGSET_SIZE_IN_BYTE {
        // 若sigsetsize不是正确的大小，则返回错误
        return Err(SyscallError::EINVAL);
    }

    let current_process = current_executor().await;
    if old_mask as usize != 0
        && current_process
            .manual_alloc_for_lazy((old_mask as usize).into())
            .await
            .is_err()
    {
        return Err(SyscallError::EFAULT);
    }
    if new_mask as usize != 0
        && current_process
            .manual_alloc_for_lazy((new_mask as usize).into())
            .await
            .is_err()
    {
        return Err(SyscallError::EPERM);
    }

    let mut signal_modules = current_process.signal_modules.lock().await;
    let signal_module = signal_modules
        .get_mut(&current_task().id().as_u64())
        .unwrap();
    if old_mask as usize != 0 {
        unsafe {
            *old_mask = signal_module.signal_set.mask;
        }
    }

    if new_mask as usize != 0 {
        let now_mask = unsafe { *new_mask };
        match flag {
            SigMaskFlag::Block => {
                signal_module.signal_set.mask |= now_mask;
            }
            SigMaskFlag::Unblock => {
                signal_module.signal_set.mask &= !now_mask;
            }
            SigMaskFlag::Setmask => {
                signal_module.signal_set.mask = now_mask;
            }
        }
    }
    Ok(0)
}

/// 向pid指定的进程发送信号
///
/// 由于处理信号的单位在线程上，所以若进程中有多个线程，则会发送给主线程
/// # Arguments
/// * `pid` - isize
/// * `signum` - isize
pub async fn syscall_kill(args: [usize; 6]) -> SyscallResult {
    let pid = args[0] as isize;
    let signum = args[1] as isize;
    if pid > 0 && signum > 0 {
        // 不关心是否成功
        let _ = executor::signal::send_signal_to_process(pid, signum, None).await;
        Ok(0)
    } else if pid == 0 {
        Err(SyscallError::ESRCH)
    } else {
        Err(SyscallError::EINVAL)
    }
}

/// 向tid指定的线程发送信号
/// # Arguments
/// * `tid` - isize
/// * `signum` - isize
pub async fn syscall_tkill(args: [usize; 6]) -> SyscallResult {
    let tid = args[0] as isize;
    let signum = args[1] as isize;
    debug!(
        "cpu: {}, send singal: {} to: {}",
        this_cpu_id(),
        signum,
        tid
    );
    if tid > 0 && signum > 0 {
        let _ = executor::signal::send_signal_to_thread(tid, signum).await;
        Ok(0)
    } else {
        Err(SyscallError::EINVAL)
    }
}

/// 向tid指定的线程组发送信号
pub async fn syscall_tgkill(args: [usize; 6]) -> SyscallResult {
    let tgid = args[0] as isize;
    let tid = args[1] as isize;
    let signum = args[2] as isize;
    debug!(
        "cpu: {}, send singal: {} to: {}",
        this_cpu_id(),
        signum,
        tid
    );
    if tgid > 0 && tid > 0 && signum > 0 {
        let _ = executor::signal::send_signal_to_thread(tid, signum).await;
        Ok(0)
    } else {
        Err(SyscallError::EINVAL)
    }
}

/// Set and get the alternate signal stack
pub async fn syscall_sigaltstack(args: [usize; 6]) -> SyscallResult {
    let current_process = current_executor().await;
    let ss = args[0] as *const SignalStack;
    let old_ss = args[1] as *mut SignalStack;
    if !ss.is_null()
        && current_process
            .manual_alloc_type_for_lazy(ss)
            .await
            .is_err()
    {
        return Err(SyscallError::EFAULT);
    }
    let task_id = current_task().id().as_u64();
    let mut signal_modules = current_process.signal_modules.lock().await;

    if !old_ss.is_null() {
        if current_process
            .manual_alloc_type_for_lazy(old_ss)
            .await
            .is_err()
        {
            return Err(SyscallError::EFAULT);
        }
        unsafe {
            *old_ss = signal_modules.get(&task_id).unwrap().alternate_stack;
        }
    }

    if !ss.is_null() {
        signal_modules.get_mut(&task_id).unwrap().alternate_stack = unsafe { *ss };
    }

    Ok(0)
}
