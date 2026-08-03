//! 通用位流（LSB-first per byte）。

pub mod reader;
pub mod writer;

pub mod error;

pub use reader::BitReader;
pub use writer::BitWriter;
pub use error::BitError;