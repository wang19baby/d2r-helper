//! Resource layer — profile-aware structured data loading and querying.
//!
//! 职责分工：
//! - `importer` — TXT/JSON → SQLite 定义层导入（Phase 2）
//! - `queries`  — 对 SQLite 定义层的统⼀查询封装
//! - `resolver` — 名称解析（Phase 3）
//! - `tooltip`  — tooltip 格式化（Phase 4 以后）
//!
//! 本层核⼼设计原则：所有查询必须带上 `profile_id`，
//! 确保多 mod / 多版本资源隔离。

pub mod import_task;
pub mod importer;
pub mod queries;
pub mod resolver;
pub mod tooltip;

pub use import_task::*;
pub use importer::*;
pub use queries::*;
pub use resolver::*;
pub use tooltip::*;
