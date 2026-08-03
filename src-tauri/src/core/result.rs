//! 协议层统一错误与 Result 别名。

use crate::core::bitio::BitError;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("BitReader: {0}")]
    Bit(#[from] BitError),

    #[error("Unknown stat id: {0}")]
    UnknownStat(u16),

    #[error("Unknown item code: {0}")]
    UnknownItemCode(String),

    #[error("Invalid quality value: {0}")]
    InvalidQuality(u8),

    #[error("Page header magic mismatch: expected 0xAA55AA55, got {0:#x}")]
    PageMagic(u32),

    #[error("D2I header magic mismatch: expected 0xAA55AA55, got {0:#x}")]
    D2IMagic(u32),

    #[error("D2S magic mismatch: expected 'D2S', got {0:?}")]
    D2SMagic([u8; 3]),

    #[error("D2X magic mismatch: expected 'D2X', got {0:?}")]
    D2XMagic([u8; 3]),

    #[error("Item too large: {0} bits (max 65536)")]
    ItemTooLarge(usize),

    #[error("Invalid version: {0:#x}")]
    InvalidVersion(u32),

    #[error("Invalid section header: {0}")]
    InvalidSection(String),

    #[error("Truncated data at offset {0}")]
    Truncated(usize),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}