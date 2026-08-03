//! 框架无关的基础设施（位流、编码、统一 Result、版本枚举、零拷贝 I/O）。

pub mod bitio;
pub mod encoding;
pub mod iomap;
pub mod result;
pub mod version;

pub use bitio::{BitError, BitReader, BitWriter};
pub use encoding::{decode_huffman_string, encode_huffman_char, encode_huffman_string, HUFFMAN_LOOKUP};
pub use iomap::{D2IDataSource, IoError, IoResult, MmapFile};
pub use result::{ParseError, ParseResult};
pub use version::ProtocolVersion;
