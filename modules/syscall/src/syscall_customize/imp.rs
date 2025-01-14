use crate::SyscallResult;

use axerrno::AxError;

/// 函数Multihash<64>::wrap的内核态实现，原函数长这样：
/// ```
/// multihash::multihash::Multihash
/// impl<const S: usize> Multihash<S>
/// pub const fn wrap(code: u64, input_digest: &[u8]) -> Result<Self, Error>
/// ```
/// 我们约定这个系统调用的规则长这样：
/// - a0 = multihash_code: u64，Multihash的code，详情请见用户态
/// - a1 = multihash_input_digest: *const u8，Multihash的输入数据，详情请见用户态
/// - a2 = multihash_input_digest_len: usize，Multihash的输入数据长度
/// - a3 = multihash_output: *mut Multihash<64>，Multihash的输出结果
pub fn syscall_multihash_wrap(args: [usize; 6]) -> SyscallResult {
    let multihash_code = args[0] as u64;
    let multihash_input_digest_bytes = args[1] as *const u8;
    let multihash_input_digest_len = args[2] as usize;
    let multihash_output = args[3] as *mut multihash::Multihash<64>;

    let multihash_input_digest = unsafe {
        core::slice::from_raw_parts(multihash_input_digest_bytes, multihash_input_digest_len)
    };

    match multihash::Multihash::<64>::wrap(multihash_code, multihash_input_digest) {
        Ok(multihash) => {
            unsafe {
                *multihash_output = multihash;
            }
            Ok(0)
        }
        Err(_) => Err(AxError::InvalidData.into()),
    }
    // todo!()
}
