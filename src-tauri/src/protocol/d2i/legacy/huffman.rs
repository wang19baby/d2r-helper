//! 兼容层：新代码请用 `crate::core::encoding::*`。

pub use crate::core::encoding::{
    decode_huffman_string, encode_huffman_char, encode_huffman_string, HUFFMAN_LOOKUP,
};

// 兼容旧模块内的子模块路径 `super::bit_reader::BitReader`
pub use crate::core::bitio::{BitReader as _BitReader, BitWriter as _BitWriter};