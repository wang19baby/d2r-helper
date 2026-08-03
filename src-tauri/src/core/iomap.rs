//! Zero-copy file I/O via memory mapping.

use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

pub type IoResult<T> = Result<T, IoError>;

#[derive(Debug)]
pub enum IoError {
    Open(std::io::Error),
    Mmap(std::io::Error),
    NoPath,
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::Open(e) => write!(f, "Failed to open file: {e}"),
            IoError::Mmap(e) => write!(f, "Failed to mmap file: {e}"),
            IoError::NoPath => write!(f, "No file path available"),
        }
    }
}

impl std::error::Error for IoError {}

/// Thread-safe, zero-copy memory-mapped file.
#[derive(Debug)]
pub struct MmapFile {
    path: Option<std::path::PathBuf>,
    mmap: Arc<Mmap>,
    mtime: std::time::SystemTime,
}

impl MmapFile {
    /// Open and memory-map a file. Fails if the file does not exist.
    pub fn open(path: impl AsRef<Path>) -> IoResult<Self> {
        let path_ref = path.as_ref();
        let path = path_ref.to_path_buf();
        let file = File::open(&path).map_err(IoError::Open)?;
        let meta = file.metadata().map_err(IoError::Open)?;
        let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
        let mmap = unsafe { Mmap::map(&file) }.map_err(IoError::Mmap)?;
        Ok(Self { path: Some(path), mmap: Arc::new(mmap), mtime })
    }

    /// Memory-map an already-open `File`.
    pub fn from_file(file: File) -> IoResult<Self> {
        let mtime = file.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
        let mmap = unsafe { Mmap::map(&file) }.map_err(IoError::Mmap)?;
        Ok(Self { path: None, mmap: Arc::new(mmap), mtime })
    }

    /// Returns a zero-copy slice, re-mmapping if the file was modified on disk.
    pub fn slice(&mut self) -> IoResult<&[u8]> {
        if let Some(ref p) = self.path
            && let Ok(meta) = std::fs::metadata(p)
                && let Ok(ct) = meta.modified()
                    && ct > self.mtime {
                        *self = Self::open(p)?;
                    }
        Ok(&self.mmap[..])
    }

    /// Zero-copy slice without mtime revalidation.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[..]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }

    #[inline]
    pub fn mtime(&self) -> std::time::SystemTime {
        self.mtime
    }

    /// Returns true if the file has been modified since this MmapFile was created.
    pub fn is_stale(&self) -> bool {
        if let Some(ref p) = self.path
            && let Ok(meta) = std::fs::metadata(p)
                && let Ok(ct) = meta.modified() {
                    return ct > self.mtime;
                }
        false
    }

    /// Force re-mmap from disk.
    pub fn refresh(&mut self) -> IoResult<()> {
        if let Some(ref p) = self.path.clone() {
            *self = Self::open(p)?;
        }
        Ok(())
    }

    /// Consume the MmapFile, drop the underlying mmap, and return the
    /// original path (if any). Use this to release the OS file handle
    /// before writing to the same file — required on Windows where an
    /// active mmap blocks `std::fs::write` with os error 1224.
    pub fn take(self) -> Option<std::path::PathBuf> {
        drop(self.mmap);
        self.path
    }
}

impl AsRef<[u8]> for MmapFile {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Trait for zero-copy data sources
// ──────────────────────────────────────────────────────────────────────────────

/// A data source that can provide `&[u8]` without heap allocation.
pub trait D2IDataSource {
    fn as_d2i_bytes(&self) -> &[u8];
    fn has_path(&self) -> bool;
    fn path_str(&self) -> Option<&str>;
}


impl D2IDataSource for MmapFile {
    #[inline]
    fn as_d2i_bytes(&self) -> &[u8] {
        self.as_slice()
    }
    #[inline]
    fn has_path(&self) -> bool {
        self.path.is_some()
    }
    #[inline]
    fn path_str(&self) -> Option<&str> {
        self.path.as_ref().and_then(|p| p.to_str())
    }
}

impl D2IDataSource for [u8] {
    #[inline]
    fn as_d2i_bytes(&self) -> &[u8] {
        self
    }
    #[inline]
    fn has_path(&self) -> bool {
        false
    }
    #[inline]
    fn path_str(&self) -> Option<&str> {
        None
    }
}

impl D2IDataSource for Vec<u8> {
    #[inline]
    fn as_d2i_bytes(&self) -> &[u8] {
        self.as_slice()
    }
    #[inline]
    fn has_path(&self) -> bool {
        false
    }
    #[inline]
    fn path_str(&self) -> Option<&str> {
        None
    }
}

impl<'a> D2IDataSource for std::borrow::Cow<'a, [u8]> {
    #[inline]
    fn as_d2i_bytes(&self) -> &[u8] {
        self
    }
    #[inline]
    fn has_path(&self) -> bool {
        false
    }
    #[inline]
    fn path_str(&self) -> Option<&str> {
        None
    }
}

impl<T: D2IDataSource> D2IDataSource for &T {
    #[inline]
    fn as_d2i_bytes(&self) -> &[u8] {
        (**self).as_d2i_bytes()
    }
    #[inline]
    fn has_path(&self) -> bool {
        (**self).has_path()
    }
    #[inline]
    fn path_str(&self) -> Option<&str> {
        (**self).path_str()
    }
}
