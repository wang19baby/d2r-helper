//! Item 元数据表（re-export from legacy）。
//!
//! 数据源：`protocol::d2i::legacy::constants::ITEM_CODE_MAP`
//! 等静态常量在新架构下继续通过 re-export 暴露。
//!
//! 后续 Phase：把常量数组本体搬到 `data/` 独立文件。

pub use crate::protocol::d2i::legacy::constants::{
    ITEM_CODE_MAP, ITEM_NAME_TO_CODE, STACKABLE_ITEM_CODES,
};