//! 兼容层：新代码请用 `crate::core::BitReader`。
//!
//! 重构计划 Step 1：将旧的 `stash/bit_reader.rs` 内容迁入 `core/bitio/reader.rs`。
//! 本文件保留作为 re-export，使现有调用方（item.rs / huffman.rs 等）无需修改即可继续工作。

pub use crate::core::BitReader;