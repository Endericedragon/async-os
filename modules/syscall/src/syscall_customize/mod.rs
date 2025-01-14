//! 自定义的一些系统调用，例如对multihash和bs58的支持等。

use crate::SyscallResult;

mod customize_syscall_id;
pub use customize_syscall_id::CustomizeSyscallId::{self, *};

mod imp;
use imp::*;

pub async fn customize_syscall(syscall_id: CustomizeSyscallId, args: [usize; 6]) -> SyscallResult {
    match syscall_id {
        MULTIHASH_WRAP => {
            syscall_multihash_wrap(args)
        }
    }
}
