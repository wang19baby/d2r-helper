//! Tests for MmapFile, including the take() method introduced in
//! Sprint 2 W5 to fix Windows mmap + std::fs::write conflicts.
//!
//! `take()` consumes the MmapFile, drops the underlying mmap, and returns
//! the original path. After calling take(), writing to the returned path
//! must succeed (this is the bug it fixes on Windows os error 1224).
//!
//! Sprint 2 W8: removed unused `use std::io::Write` import (File::write_all
//! works via the Write trait auto-imported by File).

use d2r_marketplace_lib::core::MmapFile;
use std::fs;

fn nanos() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
}

fn fresh_temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir()
        .join(format!("d2r_mmap_{}_{}_{}.bin", std::process::id(), name, nanos()))
}

/// MmapFile::take() returns the original path.
#[test]
fn test_take_returns_path() {
    let path = fresh_temp_path("returns");
    let data = b"hello mmap take\n";
    fs::write(&path, data).expect("write fixture");

    let mf = MmapFile::open(&path).expect("open");
    let taken = mf.take();
    assert!(taken.is_some(), "open() must produce Some(path)");
    assert_eq!(taken.unwrap(), path, "taken path must equal open path");
}

/// After take(), the file is still writable from the same process.
/// This locks the Windows mmap+write fix: a held mmap would block this
/// with os error 1224; take() releases the mmap so write succeeds.
#[test]
fn test_take_releases_mmap_so_write_succeeds() {
    let path = fresh_temp_path("release");
    let initial = b"first contents\n";
    fs::write(&path, initial).expect("write fixture");

    let mf = MmapFile::open(&path).expect("open");
    let taken_path = mf.take().expect("take");

    // The fix: after take(), std::fs::write must succeed.
    // Without take() (i.e. with mmap still held), this fails on Windows.
    let replacement = b"second contents after take\n";
    fs::write(&taken_path, replacement).expect("write after take must succeed");

    // Verify the file was actually updated.
    let read_back = fs::read(&taken_path).expect("read back");
    assert_eq!(read_back, replacement, "file content must reflect the write");
}

/// Reading the file via a fresh MmapFile still works after take().
#[test]
fn test_take_then_fresh_open_still_works() {
    let path = fresh_temp_path("reopen");
    fs::write(&path, b"preserve me\n").expect("write fixture");

    let mf = MmapFile::open(&path).expect("open");
    let _ = mf.take();

    let mf2 = MmapFile::open(&path).expect("re-open after take");
    assert_eq!(mf2.as_slice(), b"preserve me\n", "fresh mmap reads the same bytes");
}

/// MmapFile::from_file (no path) → take() returns None.
#[test]
fn test_take_from_file_returns_none() {
    use std::io::Write;
    let path = fresh_temp_path("fromfile");
    let mut f = fs::File::create(&path).expect("create");
    f.write_all(b"x").expect("write byte");
    drop(f);

    let f = fs::File::open(&path).expect("reopen");
    let mf = MmapFile::from_file(f).expect("from_file mmap");
    let taken = mf.take();
    assert!(taken.is_none(), "from_file (no path) → take() must be None");
}
