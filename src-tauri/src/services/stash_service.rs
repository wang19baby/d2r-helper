//! StashService — encapsulates shared stash (.d2i) file read/modify/write.
//!
//! Before this service, the same `std::fs::read → split → find_stackable →
//! update_stash_items → reassemble → write` pattern was copy-pasted across
//! marketplace.rs, warehouse.rs, and stash.rs.  This centralises it.

use crate::protocol::d2i::page::find_stackable_page;
use crate::protocol::d2i::parser::update_stackable_items_v2;

/// Error type unifying I/O, parse, and out-of-stock failures.
#[derive(Debug)]
pub enum StashError {
    Io(String),
    Parse(String),
    NoStackablePage,
    ItemNotFound,
}

impl std::fmt::Display for StashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "{}", msg),
            Self::Parse(msg) => write!(f, "{}", msg),
            Self::NoStackablePage => write!(f, "No stackable page found"),
            Self::ItemNotFound => write!(f, "Item not found in stash"),
        }
    }
}

impl From<StashError> for String {
    fn from(e: StashError) -> Self {
        e.to_string()
    }
}

pub struct StashService;

impl StashService {
    /// Add or remove items from the stackable page.
    ///
    /// * `stash_path` — full filesystem path to the `.d2i` file.
    /// * `item_code` — 3-char D2 item code (e.g. `"r01"` for El Rune).
    /// * `delta` — positive = add items (buy/deposit), negative = remove (sell/withdraw).
    ///
    /// Returns the number of items removed (delta < 0) or added (delta > 0).
    pub fn modify_stackable(
        stash_path: &str,
        item_code: &str,
        delta: i32,
    ) -> Result<i32, StashError> {
        if delta == 0 {
            return Ok(0);
        }
        let stash_mmap = crate::core::MmapFile::open(stash_path)
            .map_err(|e| StashError::Io(format!("Failed to mmap stash: {}", e)))?;
        let stash_data: &[u8] = stash_mmap.as_slice();

        let file = crate::protocol::d2i::parser::parse_file(stash_data)
            .map_err(|e| StashError::Parse(format!("Failed to parse stash: {}", e)))?;

        // Release the mmap now that the page tree is owned. Required on
        // Windows where an active mmap blocks std::fs::write (os error 1224).
        drop(stash_mmap);

        let stackable_page = find_stackable_page(&file.pages)
            .ok_or(StashError::NoStackablePage)?
            .clone();

        let add = delta > 0;
        let abs_delta = delta.unsigned_abs() as usize;

        let (_items, updated_page_data) = update_stackable_items_v2(
            &stackable_page,
            item_code,
            delta,
            add,
        ).map_err(StashError::Parse)?;

        // Reassemble pages — ONLY replace the stackable page's bytes in the
        // original file, keeping all other pages (including page 0, which may
        // have been modified by warehouse_withdraw) intact.
        let orig_bytes = std::fs::read(stash_path)
            .map_err(|e| StashError::Io(format!("Failed to re-read stash: {}", e)))?;
        let stackable_off = stackable_page.offset;
        let stackable_sz = stackable_page.size;
        let new_data_sz = updated_page_data.len();
        let final_data = if new_data_sz == stackable_sz {
            let mut buf = orig_bytes;
            buf[stackable_off..stackable_off + stackable_sz].copy_from_slice(&updated_page_data);
            buf
        } else {
            let mut buf = Vec::with_capacity(orig_bytes.len() + new_data_sz - stackable_sz);
            buf.extend_from_slice(&orig_bytes[..stackable_off]);
            buf.extend_from_slice(&updated_page_data);
            buf.extend_from_slice(&orig_bytes[stackable_off + stackable_sz..]);
            buf
        };
        std::fs::write(stash_path, &final_data)
            .map_err(|e| StashError::Io(format!("Failed to write stash: {}", e)))?;

        Ok(abs_delta as i32)
    }

    /// Resolve the shared stash file path from a save folder.
    /// Tries modern names first (ModernSharedStashSoftCoreV2.d2i), then
    /// legacy names (SharedStashSoftCoreV2.d2i) for older D2R installations.
    pub fn resolve_stash_path(save_folder: &str) -> Option<String> {
        let candidates = [
            "ModernSharedStashSoftCoreV2.d2i",
            "ModernSharedStashHardCoreV2.d2i",
            "SharedStashSoftCoreV2.d2i",
            "SharedStashHardCoreV2.d2i",
        ];
        for f in &candidates {
            let p = std::path::Path::new(save_folder).join(f);
            if p.exists() {
                return Some(p.to_string_lossy().to_string());
            }
        }
        None
    }
}
