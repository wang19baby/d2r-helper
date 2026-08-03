//! 位流错误类型。

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitError {
    #[error("Unexpected end of bit stream at offset {0}")]
    UnexpectedEnd(usize),
}