//! 运行时 stat 表加载器：从 mod 的 ItemStatCost.txt 补充/覆盖内置 stat 位宽定义。
//!
//! ## 文件格式
//! D2R 的 `itemstatcost.txt` 是 TSV（tab-separated），列定义：
//! - Col 0: Stat name
//! - Col 1: *ID
//! - Col 20: Save Bits
//! - Col 21: Save Add
//! - Col 22: Save Param Bits
//! - Col 3: Signed (空白=0, 1=有符号)
//!
//! ## 策略
//! 启动时调用 `build_runtime_table()`，尝试加载 D2R 目录下的 mod 文件。
//! 如果找不到或解析失败，用内置 512-entry 表降级——不崩溃。

use crate::protocol::common::{StatProp, StatTable};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

#[derive(Debug, Clone)]
enum RuntimeExcelContext {
    Unset,
    Disabled,
    Path(PathBuf),
}

fn runtime_excel_dir() -> &'static RwLock<RuntimeExcelContext> {
    static RUNTIME_EXCEL_DIR: OnceLock<RwLock<RuntimeExcelContext>> = OnceLock::new();
    RUNTIME_EXCEL_DIR.get_or_init(|| RwLock::new(RuntimeExcelContext::Unset))
}

/// 设置当前解析上下文要使用的 excel 目录。
/// 传入 `None` 会显式禁用运行时路径回退，避免上一个 profile 的路径污染下一次解析。
pub fn set_runtime_excel_path(path: Option<&str>) {
    if let Ok(mut guard) = runtime_excel_dir().write() {
        *guard = match path.map(str::trim).filter(|s| !s.is_empty()) {
            Some(path) => RuntimeExcelContext::Path(PathBuf::from(path)),
            None => RuntimeExcelContext::Disabled,
        };
    }
}

#[cfg(test)]
fn clear_runtime_excel_path() {
    if let Ok(mut guard) = runtime_excel_dir().write() {
        *guard = RuntimeExcelContext::Unset;
    }
}

fn resolve_stat_file_from_excel_dir(excel_dir: &Path) -> Option<PathBuf> {
    let base = excel_dir.join("base").join("ItemStatCost.txt");
    if base.exists() {
        return Some(base);
    }
    let direct = excel_dir.join("ItemStatCost.txt");
    if direct.exists() {
        return Some(direct);
    }
    None
}

/// 尝试查找 mod 的 itemstatcost.txt。
/// 搜索路径优先级：
/// 0. 当前命令显式设置的 excel 目录
/// 1. `<D2R>/mods/<active_mod>/<active_mod>.mpq/data/global/excel/itemstatcost.txt`
/// 2. `<D2R>/mods/D2RMM/D2RMM.mpq/data/global/excel/itemstatcost.txt`
fn find_stat_file() -> Option<PathBuf> {
    match runtime_excel_dir()
        .read()
        .ok()
        .map(|guard| guard.clone())
        .unwrap_or(RuntimeExcelContext::Unset)
    {
        RuntimeExcelContext::Path(configured) => {
            return resolve_stat_file_from_excel_dir(&configured);
        }
        RuntimeExcelContext::Disabled => {
            return None;
        }
        RuntimeExcelContext::Unset => {}
    }
    let candidates: [PathBuf; 2] = [
        // 用户 mod 目录
        PathBuf::from(
            "D:/personal/games/Diablo II Resurrected/mods/D2RMM/D2RMM.mpq/data/global/excel/itemstatcost.txt"
        ),
        // 从 UserProfile 尝试 Steam 安装路径
        std::env::var("USERPROFILE").ok()
            .map(|s| {
                PathBuf::from(s)
                    .join("Saved Games/Diablo II Resurrected/mods/D2RMM/D2RMM.mpq/data/global/excel/itemstatcost.txt")
            })
            .unwrap_or_default(),
    ];

    for c in &candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }
    None
}

/// 解析一行 TSV，返回 (id, save_bits, save_param_bits, save_add, signed, name)。
/// 跳过 header 行和空行。
/// 解析一行 TSV，返回 (id, save_bits, save_param_bits, save_add, signed, cs_bits, name)。
/// 跳过 header 行和空行。
fn parse_stat_line(line: &str) -> Option<(usize, u8, u8, i32, u8, u8, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("Stat\t") || line.starts_with("Stat\x09") {
        return None;
    }

    let cols: Vec<&str> = line.split('\t').collect();
    if cols.len() < 23 {
        return None;
    }

    // Col 1: *ID
    let id: usize = cols[1].trim().parse().ok()?;
    if id > 511 {
        return None;
    }

    // Col 20: Save Bits — 用于物品 stat_list
    let save_bits: u8 = cols[20].trim().parse().unwrap_or(0);
    // Col 21: Save Add
    let save_add: i32 = cols[21].trim().parse().unwrap_or(0);
    // Col 22: Save Param Bits
    let save_param_bits: u8 = cols[22].trim().parse().unwrap_or(0);
    // Col 3: Signed
    let signed: u8 = match cols[3].trim() {
        "1" => 1,
        _ => 0,
    };
    // Col 9: CSvBits — 用于角色属性 (gf 段)
    let cs_bits: u8 = cols[9].trim().parse().unwrap_or(0);
    // Col 0: name
    let name = cols[0].trim().to_string();

    Some((id, save_bits, save_param_bits, save_add, signed, cs_bits, name))
}

/// 检查是否有可用的运行时 stat 文件。
/// 比 `build_runtime_table` 轻量，不解析文件内容。
pub fn has_runtime_table() -> bool {
    find_stat_file().is_some()
}

pub fn build_runtime_table() -> StatTable {
    // 先尝试加载 mod 文件
    let path = match find_stat_file() {
        Some(p) => p,
        None => {
            // 无文件: fallback 到内置表
            return crate::data::stat_cost::build_stat_table();
        }
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[stat_loader] Failed to read '{}': {}", path.display(), e);
            return crate::data::stat_cost::build_stat_table();
        }
    };

    // 从空表开始,仅加载 txt 里定义的 stat (匹配 Python jm_parser 行为)
    let mut table = StatTable::empty();
    let mut _updated = 0usize;
    let mut _total_lines = 0;
    for line in content.lines() {
        _total_lines += 1;
        if let Some((id, save_bits, save_param_bits, save_add, signed, cs_bits, _name)) = parse_stat_line(line)
            && (save_bits > 0 || save_param_bits > 0 || cs_bits > 0) {
                table.set(
                    id,
                    StatProp {
                        save_bits,
                        num_sub_props: 1,
                        save_add,
                        save_param_bits,
                        signed,
                        encoding: 0,
                        descfunc: 0,
                        cs_bits,
                    },
                );
                _updated += 1;
            }
    }

    // ★ D2R sub-property (NP) overrides: 某些 stat 有多个连续 sub-prop
    // Python jm_parser _STAT_NP 等价: {17:2, 48:2, 50:2, 52:2, 54:3, 57:3}
    for &(sid, np) in &[(17,2),(48,2),(50,2),(52,2),(54,3),(57,3)] {
        let sid_u16 = sid as u16;
        if (sid_u16 as usize) < table.len() {
            let p = table.get(sid_u16);
            if p.save_bits > 0 {
                let mut prop = p;
                prop.num_sub_props = np;
                table.set(sid as usize, prop);
            }
        }
    }
    // ★ Merge encoding/descfunc from built-in table
    //   运行时 txt 不解析 *Encode 列，所有 encoding 被设为 0，导致 skill 拆分失效。
    //   从内置表复制 encoding/descfunc（如有）。
    let builtin = crate::data::stat_cost::build_stat_table();
    for id in 0..table.len() {
        let bp = builtin.get(id as u16);
        if bp.encoding != 0 || bp.descfunc != 0 {
            let rp = table.get(id as u16);
            if rp.encoding == 0 && rp.descfunc == 0 {
                let mut merged = rp;
                merged.encoding = bp.encoding;
                merged.descfunc = bp.descfunc;
                table.set(id, merged);
            }
        }
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_strength_line() {
        // strength\t0\t1\t\t11\t\t\t1\t0\t10...
        let line = "strength\t0\t1\t\t11\t\t\t1\t0\t10\t\t\t1\t1\t\t125\t55\t\t7\t32\t8\t32\t\t\t\t\t\t...";
        let result = parse_stat_line(line);
        assert!(result.is_some());
        let (id, bits, pbits, add, signed, cs_bits, name) = result.unwrap();
        assert_eq!(id, 0);
        assert_eq!(bits, 8);
        assert_eq!(add, 32);
        assert_eq!(pbits, 0);
        assert_eq!(signed, 0);
        assert_eq!(cs_bits, 10, "strength should have CSvBits=10");
        assert_eq!(name, "strength");
    }

    #[test]
    fn test_parse_header_line_skipped() {
        let header = "Stat\t*ID\tSend Other\tSigned\tSend Bits\tSend Param Bits\t...";
        assert!(parse_stat_line(header).is_none());
    }

    #[test]
    fn test_runtime_table_fallback_when_no_file() {
        // 当前上下文显式禁用时，不应再回退到机器上的旧 D2RMM 固定路径。
        set_runtime_excel_path(None);
        let table = build_runtime_table();
        assert_eq!(table.len(), 512);
        let str_prop = table.get(0);
        assert!(str_prop.save_bits > 0);
        clear_runtime_excel_path();
    }

    #[test]
    fn test_runtime_table_uses_configured_excel_dir() {
        let temp_root = std::env::temp_dir().join(format!("d2r_stat_loader_{}", uuid::Uuid::new_v4()));
        let base_dir = temp_root.join("base");
        std::fs::create_dir_all(&base_dir).expect("create temp base dir");
        std::fs::write(
            base_dir.join("ItemStatCost.txt"),
            "Stat\t*ID\tSend Other\tSigned\tSend Bits\tSend Param Bits\tf1\tf2\tf3\tf4\tf5\tf6\tf7\tf8\tf9\tf10\tf11\tf12\tf13\tf14\tSave Bits\tSave Add\tSave Param Bits\n\
strength\t0\t1\t\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\t9\t40\t0\n",
        )
        .expect("write itemstatcost");

        set_runtime_excel_path(Some(temp_root.to_string_lossy().as_ref()));
        assert!(has_runtime_table(), "configured excel dir should expose runtime stat table");
        let table = build_runtime_table();
        let str_prop = table.get(0);
        assert_eq!(str_prop.save_bits, 9);
        assert_eq!(str_prop.save_add, 40);

        clear_runtime_excel_path();
        std::fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_configured_missing_excel_dir_does_not_fallback_to_legacy_path() {
        let temp_root = std::env::temp_dir().join(format!("d2r_stat_loader_missing_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_root).expect("create empty temp dir");

        set_runtime_excel_path(Some(temp_root.to_string_lossy().as_ref()));
        assert!(
            !has_runtime_table(),
            "configured but missing ItemStatCost.txt should not fallback to legacy machine-specific path"
        );
        let table = build_runtime_table();
        assert_eq!(table.len(), 512);

        clear_runtime_excel_path();
        std::fs::remove_dir_all(&temp_root).ok();
    }
}
