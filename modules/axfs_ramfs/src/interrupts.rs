use alloc::{collections::BTreeMap, format, string::String};
use async_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType, VfsResult};
use axerrno::AxError;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Default)]
pub struct InterruptCounter(BTreeMap<usize, usize>);

impl InterruptCounter {
    pub fn record(&mut self, key: usize) {
        self.0.entry(key).and_modify(|cnt| *cnt += 1).or_insert(1);
    }

    pub fn content(&self) -> String {
        let mut content = String::new();

        for line in self
            .0
            .iter()
            .map(|(key, value)| format!("{}: {}", key, value))
        {
            content.push_str(&line);
            content.push('\n');
        }

        content
    }
}

lazy_static! {
    /// To record the interrupt count
    pub static ref INTERRUPT: Mutex<InterruptCounter> =
        Mutex::new(InterruptCounter(BTreeMap::default()));
}

#[derive(Default)]
/// The file node in the RAM filesystem, which records the interrupt count.
pub struct Interrupts;

impl VfsNodeOps for Interrupts {
    fn poll_get_attr(self: Pin<&Self>, _cx: &mut Context<'_>) -> Poll<VfsResult<VfsNodeAttr>> {
        Poll::Ready(Ok(VfsNodeAttr::new(
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        )))
    }

    fn poll_read_at(
        self: Pin<&Self>,
        _cx: &mut Context<'_>,
        offset: u64,
        buf: &mut [u8],
    ) -> Poll<VfsResult<usize>> {
        let content = INTERRUPT.lock().content();
        let bytes = &content.as_bytes();

        let offset = offset as usize;
        if offset > bytes.len() {
            return Poll::Ready(Err(AxError::InvalidInput));
        }

        let len = if buf.len() < bytes.len() - offset {
            buf.len()
        } else {
            bytes.len() - offset
        };

        buf[..len].copy_from_slice(&bytes[offset..len + offset]);

        Poll::Ready(Ok(len))
    }

    fn poll_write_at(
        self: Pin<&Self>,
        _cx: &mut Context<'_>,
        _offset: u64,
        _buf: &[u8],
    ) -> Poll<VfsResult<usize>> {
        Poll::Ready(Err(AxError::Io))
    }

    fn poll_truncate(self: Pin<&Self>, _cx: &mut Context<'_>, _size: u64) -> Poll<VfsResult> {
        Poll::Ready(Err(AxError::Io))
    }

    async_vfs::impl_vfs_non_dir_default! {}
}
