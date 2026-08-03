//! D2I 协议层（shared stash .d2i 文件解析）。
//!
//! 模块结构：
//! - `page_header`：64 字节 page 顶层 header
//! - `page`：page 容器 + 多页切分
//! - `jm_reader`：顺序 JM 物品流解析器（Python d2r-zero 风格）
//! - `parser`：`parse_file` 入口 + `ParsedItem` / `D2IFile` 类型定义
//! - `summary`：轻量级 stash 摘要（用于快速概览命令）
//! - `legacy`：从旧 `stash/` 模块迁入的兼容层（含 `StashItem`、位流、TXT 数据加载等）
//!
//! 新代码应优先使用 `parser` / `summary` / `protocol::common::*`，
//! `legacy` 仅供 `commands/` 层平滑过渡。

pub mod legacy;
pub mod mod_stash_detect;
pub mod page;
pub mod page_header;
pub mod parser;
pub mod jm_reader;
pub mod summary;

#[cfg(feature = "mod-stash-experimental")]
pub use parser::parse_file_mod_shoudao;
pub use mod_stash_detect::{detect_mod_stash, ModStashKind};
pub use page::{find_stackable_page, split_pages, Page};
pub use page_header::{PageHeader, D2I_PAGE_HEADER_SIZE, D2I_PAGE_MAGIC};
pub use parser::{parse_file, D2IFile, ParsedItem};
pub use summary::{summarize, ItemSummary, StashSummary};