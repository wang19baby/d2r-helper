# Auto-Generated Data Tables · Regen 指南

> 适用范围:`src-tauri/src/data/python_stats.rs` (1220 行) 与 `src-tauri/src/data/affix_names.rs` (1385 行)。
> 这两个文件由 auto-gen 工具生成,git diff 默认折叠显示(`.gitattributes` 标记 `linguist-generated=true`)。

---

## python_stats.rs

### 数据来源
- **D2R `ItemStatCost.txt`** — 游戏 data/global/excel/ItemStatCost.txt
- 字段:`stat_id → save_bits / save_param_bits / save_add / signed / CSvBits`

### 当前 API
```rust
python_stats::stat_bits(sid: u16) -> usize         // bit 数
python_stats::stat_param_bits(sid: u16) -> usize  // param bit 数
python_stats::stat_save_add(sid: u16) -> i32      // save_add 偏移
```

### Regen 方法
1. **使用 `tools/regen_data_tables.py`**（推荐）:
   ```bash
   python tools/regen_data_tables.py
   ```
   - 从 `extracted_data/itemstatcost.json` 读取 stat 定义
   - 从 `extracted_data/d2core/magic_prefix.json` / `magic_suffix.json` 读取词缀名称
   - 输出到 `src-tauri/src/data/`
   - 需要先通过「设置 → 资源导入」导入一次以使 JSON 数据就绪

### 调用方
- `src-tauri/src/bin/cli_construct.rs` (cli dev 工具)
- `src-tauri/src/protocol/d2s/items_new.rs` (D2S items reader)

---

## affix_names.rs

### 数据来源
- `data/global/excel/magic_prefix.txt` (col 0 = name string)
- `data/global/excel/magic_suffix.txt`
- 字段:`prefix_id → string` (sorted by id)

### 当前 API
```rust
affix_names::MAGIC_PREFIX_NAMES: &[(u16, &str)]    // 全部 100+ 前缀
affix_names::MAGIC_SUFFIX_NAMES: &[(u16, &str)]    // 全部 100+ 后缀
```

### Regen 方法
同上：`python tools/regen_data_tables.py` 自动生成。

### 调用方
- `src-tauri/src/bin/cli_construct.rs` (cli dev 工具)

---

## ⚠️ 当前状态

|---|---|---|---|
| python_stats.rs | ✅ tools/regen_data_tables.py | ✅ linguist-generated | items_new |
| affix_names.rs | ✅ tools/regen_data_tables.py | ✅ linguist-generated | cli_construct |

## 历史

---

## 历史

- 2026-07-20: 添加本文件,记录 regen 需求(US-010)
- 2026-07-20: `.gitattributes` 标记两个 auto-gen 文件为 `linguist-generated=true`(US-007)