//! 定义与文件I/O操作相关的trait泛型
extern crate alloc;
use core::any::Any;

use alloc::{string::String, boxed::Box};
use axerrno::{AxError, AxResult};
use async_io::{AsyncRead, AsyncSeek, AsyncWrite, SeekFrom, Seek, Read, Write};
pub use async_trait::async_trait;

/// 文件系统信息
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[cfg(target_arch = "x86_64")]
pub struct Kstat {
    /// 设备
    pub st_dev: u64,
    /// inode 编号
    pub st_ino: u64,
    /// 硬链接数
    pub st_nlink: u64,
    /// 文件类型
    pub st_mode: u32,
    /// 用户id
    pub st_uid: u32,
    /// 用户组id
    pub st_gid: u32,
    /// padding
    pub _pad0: u32,
    /// 设备号
    pub st_rdev: u64,
    /// 文件大小
    pub st_size: u64,
    /// 块大小
    pub st_blksize: u32,
    /// padding
    pub _pad1: u32,
    /// 块个数
    pub st_blocks: u64,
    /// 最后一次访问时间(秒)
    pub st_atime_sec: isize,
    /// 最后一次访问时间(纳秒)
    pub st_atime_nsec: isize,
    /// 最后一次修改时间(秒)
    pub st_mtime_sec: isize,
    /// 最后一次修改时间(纳秒)
    pub st_mtime_nsec: isize,
    /// 最后一次改变状态时间(秒)
    pub st_ctime_sec: isize,
    /// 最后一次改变状态时间(纳秒)
    pub st_ctime_nsec: isize,
}
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[cfg(not(target_arch = "x86_64"))]
pub struct Kstat {
    /// 设备
    pub st_dev: u64,
    /// inode 编号
    pub st_ino: u64,
    /// 文件类型
    pub st_mode: u32,
    /// 硬链接数
    pub st_nlink: u32,
    /// 用户id
    pub st_uid: u32,
    /// 用户组id
    pub st_gid: u32,
    /// 设备号
    pub st_rdev: u64,
    /// padding
    pub _pad0: u64,
    /// 文件大小
    pub st_size: u64,
    /// 块大小
    pub st_blksize: u32,
    /// padding
    pub _pad1: u32,
    /// 块个数
    pub st_blocks: u64,
    /// 最后一次访问时间(秒)
    pub st_atime_sec: isize,
    /// 最后一次访问时间(纳秒)
    pub st_atime_nsec: isize,
    /// 最后一次修改时间(秒)
    pub st_mtime_sec: isize,
    /// 最后一次修改时间(纳秒)
    pub st_mtime_nsec: isize,
    /// 最后一次改变状态时间(秒)
    pub st_ctime_sec: isize,
    /// 最后一次改变状态时间(纳秒)
    pub st_ctime_nsec: isize,
}

use bitflags::*;

bitflags! {
    /// 指定文件打开时的权限
    #[derive(Clone, Copy, Default, Debug)]
    pub struct OpenFlags: u32 {
        /// 只读
        const RDONLY = 0;
        /// 只能写入
        const WRONLY = 1 << 0;
        /// 读写
        const RDWR = 1 << 1;
        /// 如文件不存在，可创建它
        const CREATE = 1 << 6;
        /// 确认一定是创建文件。如文件已存在，返回 EEXIST。
        const EXCLUSIVE = 1 << 7;
        /// 使打开的文件不会成为该进程的控制终端。目前没有终端设置，不处理
        const NOCTTY = 1 << 8;
        /// 同上，在不同的库中可能会用到这个或者上一个
        const TRUNC = 1 << 9;
        /// 非阻塞读写?(虽然不知道为什么但 date.lua 也要)
        /// 在 socket 中使用得较多
        const NON_BLOCK = 1 << 11;
        /// 要求把 CR-LF 都换成 LF
        const TEXT = 1 << 14;
        /// 和上面不同，要求输入输出都不进行这个翻译
        const BINARY = 1 << 15;
        /// 对这个文件的输出需符合 IO 同步一致性。可以理解为随时 fsync
        const DSYNC = 1 << 16;
        /// 如果是符号链接，不跟随符号链接去寻找文件，而是针对连接本身
        const NOFOLLOW = 1 << 17;
        /// 在 exec 时需关闭
        const CLOEXEC = 1 << 19;
        /// 是否是目录
        const DIR = 1 << 21;
    }
}

impl OpenFlags {
    /// 获得文件的读/写权限
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
    /// 获取读权限
    pub fn readable(&self) -> bool {
        !self.contains(Self::WRONLY)
    }
    /// 获取写权限
    pub fn writable(&self) -> bool {
        self.contains(Self::WRONLY) || self.contains(Self::RDWR)
    }

    /// 获取创建权限
    pub fn creatable(&self) -> bool {
        self.contains(Self::CREATE)
    }
    /// 获取创建新文件权限
    /// 与上面的区别是，如果文件已存在，返回 EEXIST
    pub fn new_creatable(&self) -> bool {
        self.contains(Self::EXCLUSIVE)
    }

    /// 获取是否是目录
    pub fn is_dir(&self) -> bool {
        self.contains(Self::DIR)
    }

    /// 获取是否需要在 `exec()` 时关闭
    pub fn is_close_on_exec(&self) -> bool {
        self.contains(Self::CLOEXEC)
    }
}

impl From<usize> for OpenFlags {
    fn from(val: usize) -> Self {
        Self::from_bits_truncate(val as u32)
    }
}

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileIOType {
    /// 文件
    FileDesc,
    /// 目录
    DirDesc,
    /// 标准输入
    Stdin,
    /// 标准输出
    Stdout,
    /// 标准错误
    Stderr,
    /// 管道
    Pipe,
    /// 链接
    Link,
    /// Socket
    Socket,
    /// 其他
    Other,
}

/// 用于给虚存空间进行懒分配
#[async_trait::async_trait]
pub trait FileExt: AsyncRead + AsyncWrite + AsyncSeek + AsAny + Send + Sync + Unpin {
    /// whether the file is readable
    async fn readable(&self) -> bool;

    /// whether the file is writable
    async fn writable(&self) -> bool;

    /// whether the file is executable
    async fn executable(&self) -> bool;

    /// Read from position without changing cursor.
    async fn read_from_seek(&mut self, pos: SeekFrom, buf: &mut [u8]) -> AxResult<usize> {
        // get old position
        let old_pos = self
            .seek(SeekFrom::Current(0))
            .await
            .expect("Error get current pos in file");

        // seek to read position
        let _ = self.seek(pos).await.unwrap();

        // read
        let mut tmp = buf;
        let mut read_len = 0;
        while !tmp.is_empty() {
            let n = self.read(tmp).await?;
            read_len += n;
            let (_, rest) = tmp.split_at_mut(n);
            tmp = rest;
        }
        if read_len == 0 {
            return Err(AxError::UnexpectedEof);
        }
        // seek back to old_pos
        let new_pos = self.seek(SeekFrom::Start(old_pos)).await.unwrap();

        assert_eq!(old_pos, new_pos);

        Ok(read_len)
    }

    /// Write to position without changing cursor.
    async fn write_to_seek(&mut self, pos: SeekFrom, buf: &[u8]) -> AxResult<usize> {
        // get old position
        let old_pos = self
            .seek(SeekFrom::Current(0))
            .await
            .expect("Error get current pos in file");
        // seek to write position
        let _ = self.seek(pos).await.unwrap();

        let write_len = self.write(buf).await;

        // seek back to old_pos
        let _ = self.seek(SeekFrom::Start(old_pos)).await.unwrap();

        write_len
    }
}

/// File I/O trait. 文件I/O操作，用于设置文件描述符，值得注意的是，这里的read/write/seek都是不可变引用
///
/// 因为文件描述符读取的时候，是用到内部File成员的读取函数，自身应当为不可变，从而可以被Arc指针调用
#[async_trait::async_trait]
pub trait FileIO: AsAny + Send + Sync + Unpin {
    /// 读取操作
    async fn read(&self, _buf: &mut [u8]) -> AxResult<usize> {
        Err(AxError::Unsupported) // 如果没有实现, 则返回Unsupported
    }

    /// 写入操作
    async fn write(&self, _buf: &[u8]) -> AxResult<usize> {
        Err(AxError::Unsupported) // 如果没有实现, 则返回Unsupported
    }

    /// 刷新操作
    async fn flush(&self) -> AxResult<()> {
        Err(AxError::Unsupported) // 如果没有实现, 则返回Unsupported
    }

    /// 移动指针操作
    async fn seek(&self, _pos: SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现, 则返回Unsupported
    }

    /// whether the file is readable
    async fn readable(&self) -> bool;

    /// whether the file is writable
    async fn writable(&self) -> bool;

    /// whether the file is executable
    async fn executable(&self) -> bool;

    /// 获取类型
    async fn get_type(&self) -> FileIOType;

    /// 获取路径
    async fn get_path(&self) -> String {
        debug!("Function get_path not implemented");
        String::from("Function get_path not implemented")
    }
    /// 获取文件信息
    async fn get_stat(&self) -> AxResult<Kstat> {
        Err(AxError::Unsupported) // 如果没有实现get_stat, 则返回Unsupported
    }

    /// 截断文件到指定长度
    async fn truncate(&self, _len: usize) -> AxResult<()> {
        debug!("Function truncate not implemented");
        Err(AxError::Unsupported)
    }

    /// debug
    async fn print_content(&self) {
        debug!("Function print_content not implemented");
    }

    /// 设置文件状态
    async fn set_status(&self, _flags: OpenFlags) -> bool {
        false
    }

    /// 获取文件状态
    async fn get_status(&self) -> OpenFlags {
        OpenFlags::empty()
    }

    /// 设置 close_on_exec 位
    /// 设置成功返回false
    async fn set_close_on_exec(&self, _is_set: bool) -> bool {
        false
    }

    /// 处于“意外情况”。在 (p)select 和 (p)poll 中会使用到
    ///
    /// 当前基本默认为false
    async fn in_exceptional_conditions(&self) -> bool {
        false
    }

    /// 是否已经终止，对pipe来说相当于另一端已经关闭
    ///
    /// 对于其他文件类型来说，是在被close的时候终止，但这个时候已经没有对应的filedesc了，所以自然不会调用这个函数
    async fn is_hang_up(&self) -> bool {
        false
    }

    /// 已准备好读。对于 pipe 来说，这意味着读端的buffer内有值
    async fn ready_to_read(&self) -> bool {
        false
    }
    /// 已准备好写。对于 pipe 来说，这意味着写端的buffer未满
    async fn ready_to_write(&self) -> bool {
        false
    }

    /// To control the file descriptor
    async fn ioctl(&self, _request: usize, _arg1: usize) -> AxResult<isize> {
        Err(AxError::Unsupported)
    }
}

// pub trait AsyncFileIOExt {
//     /// 读取操作
//     fn read(self: Pin<&Self>, _cx: &mut Context<'_>, _buf: &mut [u8]) -> Poll<AxResult<usize>> {
//         Poll::Ready(Err(AxError::Unsupported)) // 如果没有实现, 则返回Unsupported
//     }

//     /// 写入操作
//     fn write(self: Pin<&Self>, _cx: &mut Context<'_>, _buf: &[u8]) -> Poll<AxResult<usize>> {
//         Poll::Ready(Err(AxError::Unsupported)) // 如果没有实现, 则返回Unsupported
//     }

//     /// 刷新操作
//     fn flush(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<AxResult<()>> {
//         Poll::Ready(Err(AxError::Unsupported)) // 如果没有实现, 则返回Unsupported
//     }

//     /// 移动指针操作
//     fn seek(self: Pin<&Self>, _cx: &mut Context<'_>, _pos: SeekFrom) -> Poll<AxResult<u64>> {
//         Poll::Ready(Err(AxError::Unsupported)) // 如果没有实现, 则返回Unsupported
//     }

//     /// whether the file is readable
//     fn readable(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<bool> {
//         Poll::Ready(false)
//     }

//     /// whether the file is writable
//     fn writable(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<bool> {
//         Poll::Ready(false)
//     }

//     /// whether the file is executable
//     fn executable(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<bool> {
//         Poll::Ready(false)
//     }

//     /// 获取类型
//     fn get_type(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<FileIOType>;

//     /// 获取路径
//     fn get_path(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<String> {
//         debug!("Function get_path not implemented");
//         Poll::Ready(String::from("Function get_path not implemented"))
//     }
//     /// 获取文件信息
//     fn get_stat(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<AxResult<Kstat>> {
//         Poll::Ready(Err(AxError::Unsupported)) // 如果没有实现get_stat, 则返回Unsupported
//     }

//     /// 截断文件到指定长度
//     fn truncate(self: Pin<&Self>, _cx: &mut Context<'_>, _len: usize) -> Poll<AxResult<()>> {
//         debug!("Function truncate not implemented");
//         Poll::Ready(Err(AxError::Unsupported))
//     }

//     /// debug
//     fn print_content(self: Pin<&Self>, _cx: &mut Context<'_>) {
//         debug!("Function print_content not implemented");
//     }

//     /// 设置文件状态
//     fn set_status(self: Pin<&Self>, _cx: &mut Context<'_>, _flags: OpenFlags) -> Poll<bool> {
//         Poll::Ready(false)
//     }

//     /// 获取文件状态
//     fn get_status(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<OpenFlags> {
//         Poll::Ready(OpenFlags::empty())
//     }

//     /// 设置 close_on_exec 位
//     /// 设置成功返回false
//     fn set_close_on_exec(self: Pin<&Self>, _cx: &mut Context<'_>, _is_set: bool) -> Poll<bool> {
//         Poll::Ready(false)
//     }

//     /// 处于“意外情况”。在 (p)select 和 (p)poll 中会使用到
//     ///
//     /// 当前基本默认为false
//     fn in_exceptional_conditions(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<bool> {
//         Poll::Ready(false)
//     }

//     /// 是否已经终止，对pipe来说相当于另一端已经关闭
//     ///
//     /// 对于其他文件类型来说，是在被close的时候终止，但这个时候已经没有对应的filedesc了，所以自然不会调用这个函数
//     fn is_hang_up(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<bool> {
//         Poll::Ready(false)
//     }

//     /// 已准备好读。对于 pipe 来说，这意味着读端的buffer内有值
//     fn ready_to_read(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<bool> {
//         Poll::Ready(false)
//     }
//     /// 已准备好写。对于 pipe 来说，这意味着写端的buffer未满
//     fn ready_to_write(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<bool> {
//         Poll::Ready(false)
//     }

//     /// To control the file descriptor
//     fn ioctl(self: Pin<&Self>, _cx: &mut Context<'_>, _request: usize, _arg1: usize) -> Poll<AxResult<isize>> {
//         Poll::Ready(Err(AxError::Unsupported))
//     }
// }

// impl<P> AsyncFileIOExt for Pin<P>
// where
//     P: Deref + Unpin,
//     P::Target: AsyncFileIOExt,
// {
//     fn read(self: Pin<&Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<AxResult<usize>> {
//         self.get_ref().as_ref().read(cx, buf)
//     }

//     fn write(self: Pin<&Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<AxResult<usize>> {
//         self.get_ref().as_ref().write(cx, buf)
//     }

//     fn flush(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<AxResult<()>> {
//         self.get_ref().as_ref().flush(cx)
//     }

//     fn seek(self: Pin<&Self>, cx: &mut Context<'_>, pos: SeekFrom) -> Poll<AxResult<u64>> {
//         self.get_ref().as_ref().seek(cx, pos)
//     }

//     /// whether the file is readable
//     fn readable(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//         self.get_ref().as_ref().readable(cx)
//     }

//     /// whether the file is writable
//     fn writable(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//         self.get_ref().as_ref().writable(cx)
//     }

//     /// whether the file is executable
//     fn executable(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//         self.get_ref().as_ref().executable(cx)
//     }

//     fn get_path(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<String> {
//         self.get_ref().as_ref().get_path(cx)
//     }

//     fn get_stat(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<AxResult<Kstat>> {
//         self.get_ref().as_ref().get_stat(cx)
//     }

//     fn get_status(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<OpenFlags> {
//         self.get_ref().as_ref().get_status(cx)
//     }

//     fn get_type(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<FileIOType> {
//         self.get_ref().as_ref().get_type(cx)
//     }

//     fn in_exceptional_conditions(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//         self.get_ref().as_ref().in_exceptional_conditions(cx)
//     }

//     fn ioctl(self: Pin<&Self>, cx: &mut Context<'_>, request: usize, arg1: usize) -> Poll<AxResult<isize>> {
//         self.get_ref().as_ref().ioctl(cx, request, arg1)
//     }

//     fn is_hang_up(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//         self.get_ref().as_ref().is_hang_up(cx)
//     }

//     fn print_content(self: Pin<&Self>, cx: &mut Context<'_>) {
//         self.get_ref().as_ref().print_content(cx)
//     }

//     fn ready_to_read(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//         self.get_ref().as_ref().ready_to_read(cx)
//     }

//     fn ready_to_write(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//         self.get_ref().as_ref().ready_to_write(cx)
//     }

//     fn set_close_on_exec(self: Pin<&Self>, cx: &mut Context<'_>, is_set: bool) -> Poll<bool> {
//         self.get_ref().as_ref().set_close_on_exec(cx, is_set)
//     }

//     fn set_status(self: Pin<&Self>, cx: &mut Context<'_>, flags: OpenFlags) -> Poll<bool> {
//         self.get_ref().as_ref().set_status(cx, flags)
//     }

//     fn truncate(self: Pin<&Self>, cx: &mut Context<'_>, len: usize) -> Poll<AxResult<()>> {
//         self.get_ref().as_ref().truncate(cx, len)
//     }
// }

// macro_rules! deref_async_file_io_ext {
//     () => {
//         fn read(self: Pin<&Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<AxResult<usize>> {
//             Pin::new(&**self).read(cx, buf)
//         }
    
//         fn write(self: Pin<&Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<AxResult<usize>> {
//             Pin::new(&**self).write(cx, buf)
//         }
    
//         fn flush(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<AxResult<()>> {
//             Pin::new(&**self).flush(cx)
//         }
    
//         fn seek(self: Pin<&Self>, cx: &mut Context<'_>, pos: SeekFrom) -> Poll<AxResult<u64>> {
//             Pin::new(&**self).seek(cx, pos)
//         }

//         /// whether the file is readable
//         fn readable(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//             Pin::new(&**self).readable(cx)
//         }

//         /// whether the file is writable
//         fn writable(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//             Pin::new(&**self).writable(cx)
//         }

//         /// whether the file is executable
//         fn executable(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//             Pin::new(&**self).executable(cx)
//         }

//         fn get_path(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<String> {
//             Pin::new(&**self).get_path(cx)
//         }
    
//         fn get_stat(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<AxResult<Kstat>> {
//             Pin::new(&**self).get_stat(cx)
//         }
    
//         fn get_status(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<OpenFlags> {
//             Pin::new(&**self).get_status(cx)
//         }
    
//         fn get_type(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<FileIOType> {
//             Pin::new(&**self).get_type(cx)
//         }
    
//         fn in_exceptional_conditions(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//             Pin::new(&**self).in_exceptional_conditions(cx)
//         }
    
//         fn ioctl(self: Pin<&Self>, cx: &mut Context<'_>, request: usize, arg1: usize) -> Poll<AxResult<isize>> {
//             Pin::new(&**self).ioctl(cx, request, arg1)
//         }
    
//         fn is_hang_up(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//             Pin::new(&**self).is_hang_up(cx)
//         }
    
//         fn print_content(self: Pin<&Self>, cx: &mut Context<'_>) {
//             Pin::new(&**self).print_content(cx)
//         }
    
//         fn ready_to_read(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//             Pin::new(&**self).ready_to_read(cx)
//         }
    
//         fn ready_to_write(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<bool> {
//             Pin::new(&**self).ready_to_write(cx)
//         }
    
//         fn set_close_on_exec(self: Pin<&Self>, cx: &mut Context<'_>, is_set: bool) -> Poll<bool> {
//             Pin::new(&**self).set_close_on_exec(cx, is_set)
//         }
    
//         fn set_status(self: Pin<&Self>, cx: &mut Context<'_>, flags: OpenFlags) -> Poll<bool> {
//             Pin::new(&**self).set_status(cx, flags)
//         }
    
//         fn truncate(self: Pin<&Self>, cx: &mut Context<'_>, len: usize) -> Poll<AxResult<()>> {
//             Pin::new(&**self).truncate(cx, len)
//         }
//     };
// }

// impl<T: ?Sized + AsyncFileIOExt + Unpin> AsyncFileIOExt for &T {
//     deref_async_file_io_ext!();
// }

// impl<T: ?Sized + AsyncFileIOExt + Unpin> AsyncFileIOExt for Arc<T> {
//     deref_async_file_io_ext!();
// }

// // pub type FileIO = dyn AsyncFileIOExt + Send + Sync + Unpin;

// #[async_trait::async_trait]
// pub trait FileIO: AsAny + Send + Sync {
//     /// 读取操作
//     async fn read(&self, _buf: &mut [u8]) -> AxResult<usize> {
//         Err(AxError::Unsupported) // 如果没有实现, 则返回Unsupported
//     }

//     /// 写入操作
//     async fn write(&self, _buf: &[u8]) -> AxResult<usize> {
//         Err(AxError::Unsupported) // 如果没有实现, 则返回Unsupported
//     }

//     /// 刷新操作
//     async fn flush(&self) -> AxResult<()> {
//         Err(AxError::Unsupported) // 如果没有实现, 则返回Unsupported
//     }

//     /// 移动指针操作
//     async fn seek(&self, _pos: SeekFrom) -> AxResult<u64> {
//         Err(AxError::Unsupported) // 如果没有实现, 则返回Unsupported
//     }

//     /// whether the file is readable
//     async fn readable(&self) -> bool;

//     /// whether the file is writable
//     async fn writable(&self) -> bool;

//     /// whether the file is executable
//     async fn executable(&self) -> bool;

//     /// 获取类型
//     async fn get_type(&self) -> FileIOType;

//     /// 获取路径
//     async fn get_path(&self) -> String {
//         debug!("Function get_path not implemented");
//         String::from("Function get_path not implemented")
//     }
//     /// 获取文件信息
//     async fn get_stat(&self) -> AxResult<Kstat> {
//         Err(AxError::Unsupported) // 如果没有实现get_stat, 则返回Unsupported
//     }

//     /// 截断文件到指定长度
//     async fn truncate(&self, _len: usize) -> AxResult<()> {
//         debug!("Function truncate not implemented");
//         Err(AxError::Unsupported)
//     }

//     /// debug
//     async fn print_content(&self) {
//         debug!("Function print_content not implemented");
//     }

//     /// 设置文件状态
//     async fn set_status(&self, _flags: OpenFlags) -> bool {
//         false
//     }

//     /// 获取文件状态
//     async fn get_status(&self) -> OpenFlags {
//         OpenFlags::empty()
//     }

//     /// 设置 close_on_exec 位
//     /// 设置成功返回false
//     async fn set_close_on_exec(&self, _is_set: bool) -> bool {
//         false
//     }

//     /// 处于“意外情况”。在 (p)select 和 (p)poll 中会使用到
//     ///
//     /// 当前基本默认为false
//     async fn in_exceptional_conditions(&self) -> bool {
//         false
//     }

//     /// 是否已经终止，对pipe来说相当于另一端已经关闭
//     ///
//     /// 对于其他文件类型来说，是在被close的时候终止，但这个时候已经没有对应的filedesc了，所以自然不会调用这个函数
//     async fn is_hang_up(&self) -> bool {
//         false
//     }

//     /// 已准备好读。对于 pipe 来说，这意味着读端的buffer内有值
//     async fn ready_to_read(&self) -> bool {
//         false
//     }
//     /// 已准备好写。对于 pipe 来说，这意味着写端的buffer未满
//     async fn ready_to_write(&self) -> bool {
//         false
//     }

//     /// To control the file descriptor
//     async fn ioctl(&self, _request: usize, _arg1: usize) -> AxResult<isize> {
//         Err(AxError::Unsupported)
//     }
// }

/// `FileExt` 需要满足 `AsAny` 的要求，即可以转化为 `Any` 类型，从而能够进行向下类型转换。
pub trait AsAny {
    /// 把当前对象转化为 `Any` 类型，供后续 downcast 使用
    fn as_any(&self) -> &dyn Any;
    /// 供 downcast_mut 使用
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
bitflags! {
    /// 指定文件打开时的权限
    #[derive(Clone, Copy)]
    pub struct AccessMode: u16 {
        /// 用户读权限
        const S_IRUSR = 1 << 8;
        /// 用户写权限
        const S_IWUSR = 1 << 7;
        /// 用户执行权限
        const S_IXUSR = 1 << 6;
        /// 用户组读权限
        const S_IRGRP = 1 << 5;
        /// 用户组写权限
        const S_IWGRP = 1 << 4;
        /// 用户组执行权限
        const S_IXGRP = 1 << 3;
        /// 其他用户读权限
        const S_IROTH = 1 << 2;
        /// 其他用户写权限
        const S_IWOTH = 1 << 1;
        /// 其他用户执行权限
        const S_IXOTH = 1 << 0;
    }
}

impl From<usize> for AccessMode {
    fn from(val: usize) -> Self {
        Self::from_bits_truncate(val as u16)
    }
}

/// IOCTL系统调用支持
#[allow(missing_docs)]
pub const TCGETS: usize = 0x5401;
#[allow(missing_docs)]
pub const TIOCGPGRP: usize = 0x540F;
#[allow(missing_docs)]
pub const TIOCSPGRP: usize = 0x5410;
#[allow(missing_docs)]
pub const TIOCGWINSZ: usize = 0x5413;
#[allow(missing_docs)]
pub const FIONBIO: usize = 0x5421;
#[allow(missing_docs)]
pub const FIOCLEX: usize = 0x5451;
#[repr(C)]
#[derive(Clone, Copy, Default)]
/// the size of the console window
pub struct ConsoleWinSize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}
