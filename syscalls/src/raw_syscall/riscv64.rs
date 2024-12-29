// On riscv64, the following registers are used for args 1-6:
// arg1: %a0
// arg2: %a1
// arg3: %a2
// arg4: %a3
// arg5: %a4
// arg6: %a5
//
// %a7 is used for the syscall number.
//
// %a0 is reused for the syscall return value.
//
// No other registers are clobbered.
use super::AsyncFlags;
use core::{arch::asm, task::Waker};

/// Issues a raw async system call with 0 arguments.
///
/// # Safety
///
/// Running a system call is inherently unsafe. It is the caller's
/// responsibility to ensure safety.
#[inline]
pub unsafe fn syscall0(n: usize, ret_ptr: Option<(usize, &Waker)>) -> usize {
    let mut ret: usize;
    let (ret_ptr, task_ptr, flag) = if let Some((ret_ptr, waker)) = ret_ptr {
        (ret_ptr, waker.data() as usize, AsyncFlags::ASYNC as usize)
    } else {
        (0, 0, AsyncFlags::SYNC as usize)
    };
    asm!(
        "ecall",
        in("a7") n,
        in("t0") flag,
        in("t1") ret_ptr,
        in("t2") task_ptr,
        out("a0") ret,
        options(nostack, preserves_flags)
    );
    ret
}

/// Issues a raw system call with 1 argument.
///
/// # Safety
///
/// Running a system call is inherently unsafe. It is the caller's
/// responsibility to ensure safety.
#[inline]
pub unsafe fn syscall1(n: usize, arg1: usize, ret_ptr: Option<(usize, &Waker)>) -> usize {
    let mut ret: usize;
    let (ret_ptr, task_ptr, flag) = if let Some((ret_ptr, waker)) = ret_ptr {
        (ret_ptr, waker.data() as usize, AsyncFlags::ASYNC as usize)
    } else {
        (0, 0, AsyncFlags::SYNC as usize)
    };
    asm!(
        "ecall",
        in("a7") n,
        inlateout("a0") arg1 => ret,
        in("t0") flag,
        in("t1") ret_ptr,
        in("t2") task_ptr,
        options(nostack, preserves_flags)
    );
    ret
}

/// Issues a raw system call with 2 arguments.
///
/// # Safety
///
/// Running a system call is inherently unsafe. It is the caller's
/// responsibility to ensure safety.
#[inline]
pub unsafe fn syscall2(
    n: usize,
    arg1: usize,
    arg2: usize,
    ret_ptr: Option<(usize, &Waker)>,
) -> usize {
    let mut ret: usize;
    let (ret_ptr, task_ptr, flag) = if let Some((ret_ptr, waker)) = ret_ptr {
        (ret_ptr, waker.data() as usize, AsyncFlags::ASYNC as usize)
    } else {
        (0, 0, AsyncFlags::SYNC as usize)
    };
    asm!(
        "ecall",
        in("a7") n,
        inlateout("a0") arg1 => ret,
        in("a1") arg2,
        in("t0") flag,
        in("t1") ret_ptr,
        in("t2") task_ptr,
        options(nostack, preserves_flags)
    );
    ret
}

/// Issues a raw system call with 3 arguments.
///
/// # Safety
///
/// Running a system call is inherently unsafe. It is the caller's
/// responsibility to ensure safety.
#[inline]
pub unsafe fn syscall3(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    ret_ptr: Option<(usize, &Waker)>,
) -> usize {
    let mut ret: usize;
    let (ret_ptr, task_ptr, flag) = if let Some((ret_ptr, waker)) = ret_ptr {
        (ret_ptr, waker.data() as usize, AsyncFlags::ASYNC as usize)
    } else {
        (0, 0, AsyncFlags::SYNC as usize)
    };
    asm!(
        "ecall",
        in("a7") n,
        inlateout("a0") arg1 => ret,
        in("a1") arg2,
        in("a2") arg3,
        in("t0") flag,
        in("t1") ret_ptr,
        in("t2") task_ptr,
        options(nostack, preserves_flags)
    );
    ret
}

/// Issues a raw system call with 4 arguments.
///
/// # Safety
///
/// Running a system call is inherently unsafe. It is the caller's
/// responsibility to ensure safety.
#[inline]
pub unsafe fn syscall4(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    ret_ptr: Option<(usize, &Waker)>,
) -> usize {
    let mut ret: usize;
    let (ret_ptr, task_ptr, flag) = if let Some((ret_ptr, waker)) = ret_ptr {
        (ret_ptr, waker.data() as usize, AsyncFlags::ASYNC as usize)
    } else {
        (0, 0, AsyncFlags::SYNC as usize)
    };
    asm!(
        "ecall",
        in("a7") n,
        inlateout("a0") arg1 => ret,
        in("a1") arg2,
        in("a2") arg3,
        in("a3") arg4,
        in("t0") flag,
        in("t1") ret_ptr,
        in("t2") task_ptr,
        options(nostack, preserves_flags)
    );
    ret
}

/// Issues a raw system call with 5 arguments.
///
/// # Safety
///
/// Running a system call is inherently unsafe. It is the caller's
/// responsibility to ensure safety.
#[inline]
pub unsafe fn syscall5(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    ret_ptr: Option<(usize, &Waker)>,
) -> usize {
    let mut ret: usize;
    let (ret_ptr, task_ptr, flag) = if let Some((ret_ptr, waker)) = ret_ptr {
        (ret_ptr, waker.data() as usize, AsyncFlags::ASYNC as usize)
    } else {
        (0, 0, AsyncFlags::SYNC as usize)
    };
    asm!(
        "ecall",
        in("a7") n,
        inlateout("a0") arg1 => ret,
        in("a1") arg2,
        in("a2") arg3,
        in("a3") arg4,
        in("a4") arg5,
        in("t0") flag,
        in("t1") ret_ptr,
        in("t2") task_ptr,
        options(nostack, preserves_flags)
    );
    ret
}

/// Issues a raw system call with 6 arguments.
///
/// # Safety
///
/// Running a system call is inherently unsafe. It is the caller's
/// responsibility to ensure safety.
#[inline]
pub unsafe fn syscall6(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
    ret_ptr: Option<(usize, &Waker)>,
) -> usize {
    let mut ret: usize;
    let (ret_ptr, task_ptr, flag) = if let Some((ret_ptr, waker)) = ret_ptr {
        (ret_ptr, waker.data() as usize, AsyncFlags::ASYNC as usize)
    } else {
        (0, 0, AsyncFlags::SYNC as usize)
    };
    asm!(
        "ecall",
        in("a7") n,
        inlateout("a0") arg1 => ret,
        in("a1") arg2,
        in("a2") arg3,
        in("a3") arg4,
        in("a4") arg5,
        in("a5") arg6,
        in("t0") flag,
        in("t1") ret_ptr,
        in("t2") task_ptr,
        options(nostack, preserves_flags)
    );
    ret
}
