use async_io::Result;
use core::fmt;
use crate::fops;
use super::FileExt;
use alloc::boxed::Box;

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
pub type FileType = fops::FileType;

/// Representation of the various permissions on a file.
pub type Permissions = fops::FilePerm;

/// An object providing access to an open file on the filesystem.
pub type File = fops::File;

/// Metadata information about a file.
pub struct Metadata(fops::FileAttr);

/// Options and flags which can be used to configure how a file is opened.
#[derive(Clone, Debug)]
pub struct OpenOptions(fops::OpenOptions);

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    pub const fn new() -> Self {
        OpenOptions(fops::OpenOptions::new())
    }

    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.0.read(read);
        self
    }

    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.0.write(write);
        self
    }

    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.0.append(append);
        self
    }

    /// Sets the option for truncating a previous file.
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.0.truncate(truncate);
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.0.create(create);
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.0.create_new(create_new);
        self
    }

    /// Opens a file at `path` with the options specified by `self`.
    pub async fn open(&self, path: &str) -> Result<File> {
        fops::File::open_withperm(path, &self.0).await
    }
}

impl Metadata {
    /// Returns the file type for this metadata.
    pub const fn file_type(&self) -> FileType {
        self.0.file_type()
    }

    /// Returns `true` if this metadata is for a directory. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_file`].
    pub const fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    /// Returns `true` if this metadata is for a regular file. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_dir`].
    pub const fn is_file(&self) -> bool {
        self.0.is_file()
    }

    /// Returns the size of the file, in bytes, this metadata is for.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> u64 {
        self.0.size()
    }

    /// Returns the permissions of the file this metadata is for.
    pub const fn permissions(&self) -> Permissions {
        self.0.perm()
    }

    /// Sets the permissions of the file this metadata is for.
    pub fn set_permissions(&mut self, perm: Permissions) {
        self.0.set_perm(perm)
    }

    /// Returns the total size of this file in bytes.
    pub const fn size(&self) -> u64 {
        self.0.size()
    }

    /// Returns the number of blocks allocated to the file, in 512-byte units.
    pub const fn blocks(&self) -> u64 {
        self.0.blocks()
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metadata")
            .field("file_type", &self.file_type())
            .field("is_dir", &self.is_dir())
            .field("is_file", &self.is_file())
            .field("permissions", &self.permissions())
            .finish_non_exhaustive()
    }
}

impl File {
    /// Attempts to open a file in read-only mode.
    pub async fn open(path: &str) -> Result<Self> {
        OpenOptions::new().read(true).open(path).await
    }

    /// Opens a file in write-only mode.
    pub async fn create(path: &str) -> Result<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path).await
    }

    /// Creates a new file in read-write mode; error if the file exists.
    pub async fn create_new(path: &str) -> Result<Self> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path).await
    }

    /// Returns a new OpenOptions object.
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    /// Truncates or extends the underlying file, updating the size of
    /// this file to become `size`.
    pub async fn set_len(&self, size: u64) -> Result<()> {
        // self.inner.truncate(size).await
        self.truncate(size).await
    }

    /// Queries metadata about the underlying file.
    pub async fn metadata(&self) -> Result<Metadata> {
        // self.inner.get_attr().await.map(Metadata)
        self.get_attr().await.map(Metadata)
    }
}

#[async_trait::async_trait]
impl FileExt for File {
    async fn readable(&self) -> bool {
        self.readable()
    }

    /// whether the file is writable
    async fn writable(&self) -> bool {
        self.writable()
    }

    /// whether the file is executable
    async fn executable(&self) -> bool {
        self.executable()
    }
}
