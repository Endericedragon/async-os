//! Device filesystem used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! The implementation is based on [`axfs_vfs`].

#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, feature(noop_waker))]

extern crate alloc;

mod dir;
mod null;
mod random;
#[cfg(test)]
mod tests;
mod zero;
pub use self::dir::DirNode;

pub use self::null::NullDev;
pub use self::random::RandomDev;
pub use self::zero::ZeroDev;
use alloc::sync::Arc;
use async_vfs::{VfsNodeOps, VfsNodeRef, VfsOps, VfsResult};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use spin::once::Once;

/// A device filesystem that implements [`axfs_vfs::VfsOps`].
pub struct DeviceFileSystem {
    parent: Once<VfsNodeRef>,
    root: Arc<DirNode>,
}

impl DeviceFileSystem {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            parent: Once::new(),
            root: DirNode::new(None),
        }
    }

    /// Create a subdirectory at the root directory.
    pub fn mkdir(&self, name: &'static str) -> Arc<DirNode> {
        self.root.mkdir(name)
    }

    /// Add a node to the root directory.
    ///
    /// The node must implement [`axfs_vfs::VfsNodeOps`], and be wrapped in [`Arc`].
    pub fn add(&self, name: &'static str, node: VfsNodeRef) {
        self.root.add(name, node);
    }
}

impl VfsOps for DeviceFileSystem {
    fn poll_mount(
        self: Pin<&Self>,
        _cx: &mut Context<'_>,
        _path: &str,
        mount_point: &VfsNodeRef,
    ) -> Poll<VfsResult> {
        if let Some(parent) = mount_point.parent() {
            self.root.set_parent(Some(self.parent.call_once(|| parent)));
        } else {
            self.root.set_parent(None);
        }
        Poll::Ready(Ok(()))
    }

    fn root_dir(&self) -> VfsNodeRef {
        self.root.clone()
    }
}

impl Default for DeviceFileSystem {
    fn default() -> Self {
        Self::new()
    }
}
