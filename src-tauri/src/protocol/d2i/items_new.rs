//! D2I JM item parser — re-exports from `jm_reader`.
//!
//! This module exists as the canonical entry point for D2I item parsing.
//! The underlying implementation lives in `jm_reader` (sequential JM parser
//! following the Python d2r-zero approach).

pub use super::jm_reader::{parse_jm_page, parse_jm_page_with_table};
