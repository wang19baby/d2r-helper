# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## D2R Marketplace - Tauri (Rust) Port

This is the Rust + Tauri port of the D2R Marketplace application — an offline auction house
for Diablo II: Resurrected single-player. The original Python/Flask version is at
`../d2r-marketplace/`.

## Architecture

```
src-tauri/
├── src/
│   ├── main.rs                      Binary entry point → calls lib::run()
│   ├── lib.rs                       Tauri app setup, command registration, AppState
│   ├── core/                        Framework-agnostic infrastructure
│   │   ├── bitio/                   BitReader (LSB-first) + BitWriter + BitError
│   │   ├── encoding/                Huffman 4-char encoding (D2SLib ItemCodeTree)
│   │   ├── result.rs                ParseError + ParseResult (统一协议层错误)
│   │   └── version.rs               ProtocolVersion V96/V97/V98/V105/V111
│   ├── data/                        Static data tables
│   │   ├── stat_cost.rs             420+ magical property definitions + build_stat_table()
│   │   └── items.rs                 ITEM_CODE_MAP / ITEM_NAME_TO_CODE (re-export from legacy)
│   ├── protocol/                    D2 file format protocol layer
│   │   ├── common/                  Cross-format shared fields
│   │   │   ├── item_flags.rs        32b flags header
│   │   │   ├── item_mode.rs         3b (Stored/Equipped/Belt/Buffer/Socket)
│   │   │   ├── item_location.rs     4b (Head/Neck/Torso/...)
│   │   │   ├── item_page.rs         ★ 3b Page 字段（★ 修复漏读 ★）
│   │   │   ├── item_quality.rs      4b (Low/Normal/Magic/Set/Rare/Unique/Crafted)
│   │   │   ├── item.rs              Item aggregate struct
│   │   │   ├── version_dispatch.rs  FieldSet: socket_count/version_bits/has_chronicle/...
│   │   │   ├── stat.rs / stat_list.rs / stat_table.rs
│   │   ├── d2i/                     Shared stash parser (.d2i files)
│   │   │   ├── page_header.rs       64-byte page header (magic 0xAA55AA55)
│   │   │   ├── page.rs              Page container + multi-page split
│   │   │   ├── parser.rs            ★ Top-level parse_file + read_item (含 Page 3b 修复)
│   │   │   ├── summary.rs           StashSummary (轻量级，用于 commands)
│   │   │   └── legacy/              ★ 兼容层：从原 stash/ 迁入的 21 文件
│   │   │                              含 1500 行 complete header（D2SLib 等价）
│   │   │                              新 parser 通过 skip_non_simple_complete_header 委托
│   │   └── d2s/                     Character save (.d2s files) — 基础完成
│   │       ├── header.rs            D2S_MAGIC + version + UTF-16 name + class/status
│   │       ├── attributes.rs        61-byte attributes (str/dex/vit/energy + level + ...)
│   │       └── parser.rs            Top-level parse_file
│   ├── commands/                    Tauri IPC command handlers
│   │   ├── stash.rs                 read_stash / read_stash_file / backup
│   │   ├── marketplace.rs           list/sell/buy/cancel/price_suggestion
│   │   ├── balance.rs               get_balance
│   │   ├── config.rs                get/update save_folder, game_root, active_mod, ...
│   │   ├── warehouse.rs             扩展收藏：deposit/withdraw/list
│   │   └── character.rs             ★ 新增 d2s 命令（read_character_info / list_characters）
│   ├── database/                    SQLite persistence layer
│   │   ├── db.rs                    Schema, CRUD for items/user/transactions/warehouse
│   │   └── models.rs                VirtualItem, ListedItem, SoldItem, WarehousedItem
│   ├── market/                      Economy/logic layer
│   │   ├── pricing.rs               Reference prices, price suggestions
│   │   ├── sell_time.rs             Auto-sell timer calculation
│   │   └── trade_rules.rs           Tradeable/purchasable rules
│   ├── io/                          File I/O layer (placeholder, 内容已并入 protocol::d2i::legacy)
│   └── stash/                       ❌ 已删除 — 内容迁入 protocol::d2i::legacy/
├── Cargo.toml
├── tauri.conf.json
├── build.rs
├── icons/
└── tests/
    ├── fixtures/                    真实 d2i fixture (ModernSharedStashSoftCoreV2.d2i 等)
    ├── d2i_real_integration.rs      ★ 真实 d2i 集成测试（验证 Page 3b 修复）
    ├── stash_integration.rs          Legacy 集成测试
    ├── stash_equipment_tests.rs      Equipment 解析测试
    ├── warehouse_tests.rs            收藏层测试
    └── poc_stash_chinese.rs          中文物品名 PoC
```

## Key Commands

```bash
# Run in development mode
cd src-tauri && cargo tauri dev

# Build for production
cd src-tauri && cargo tauri build

# Build Rust backend only (faster, no frontend build)
cd src-tauri && cargo build

# Run tests
cd src-tauri && cargo test

# Run specific module tests
cd src-tauri && cargo test --lib core::                     # bitio + encoding + version
cd src-tauri && cargo test --lib protocol::                # 75+ tests
cd src-tauri && cargo test --test d2i_real_integration    # 真实 d2i fixture
```

## Stash Binary Format (D2I)

The `.d2i` file format uses the game's own bit-level encoding:
- **Page structure**: 64-byte headers with magic `0xAA55AA55`, max 50 pages (mod-extended)
- **Stackable page**: Identified by `is_stackable=1` flag (runes/gems/keys)
- **BitReader/BitWriter**: LSB-first per byte, peek/seek/align/slice API
- **Item type encoding**: Huffman-encoded 4-char codes (e.g. "r01", "gcv")
- **Chest-stackable trailer**: 1 bit flag + 8 bits amount (actual stash count)
- **★ Page 3b 字段**: v97+ ItemPage field (Equipped/Backpack/MyStash/SharedStash/Mod)
- **Version**: v96/v97/v98/v105/v111, default v105

See `docs/d2r-stash-reverse-engineering.md` in the original project for full format details.

## Important Constraints

- **Never open D2R Marketplace and the game at the same time** — can corrupt save files
- Only **stackable items** (runes, gems, potions, keys, essences, tokens, shards) supported
- Marketplace currently only allows **rune purchases** directly into the stash
- MAX_STACK = 99 (hardcoded in legacy/item.rs)
- Database lives in `%LOCALAPPDATA%\D2RMarketplace\database\`

## ★ 循环开发协议 (Development Loop)

让 AI 自己完成「写 → 跑测试 → 改 → 再跑」直到全绿，人只发目标、核对 diff、收结果。

### 验证（合格线）
- **开工前先跑一次 `cd src-tauri && cargo test` 验证基线是绿的**；基线红先修基线，再动工（ECC 安全门）
- 验收停止条件 = **编译通过 + 测试通过**（cargo test 含编译；编译失败也算红）
- 改完代码必须自己跑：`cd src-tauri && cargo test`
- 全绿（exit 0）才算完成；非 0 就继续修，不许声称 done
- **失败先看代码，禁止改测试 / 删测试 / 改 fixtures 来凑绿**
- 全绿后明确输出「DONE」

### 开工前（TDD 对齐）
- 新功能 / 修 bug：先写 1 个失败测试（把「做完长啥样」翻译成可执行断言），再实现
- 示例：`tests/warehouse_tests.rs` 的 T18（先复现 withdraw merge bug，再修）

### 报告与信任
- **git diff 是真相，口头汇报仅供参考** — 交工前核对实际改动
- 每轮结束报告：改了哪些文件 + 测试结果

### 禁止区
- 不改 `tests/fixtures/*.d2i`（真实存档夹具 = 解析正确性基准）
- 不改 `protocol::d2i::legacy::*` 的公开签名（新 parser 依赖其委托）
- 不加新 Cargo 依赖（要加先在回复里说明理由）

### 循环控制
- 粒度：控制在人半小时能写完的量级；大活先拆
- 停止条件：cargo test 全绿 + DONE 声明；**未达 DONE 不许交工**
- 无进展检测：**连续 3 轮测试结果无变化（仍红且失败项相同）就停下来让用户介入**，不要无限撞

### Loop 还是 Graph：分层决策

**默认走 Loop（单 Agent 迭代），只有任务结构本身要求时才上 Graph（多 Agent 协作）**——概念要跟上，但别为了追概念上 Graph。

| 任务形态 | 用什么 | 说明 |
|----------|--------|------|
| 小活：改 1-2 个文件，目标明确 | 直接干 / `/fix` 循环 | 不需要建模 |
| 中活：单 Agent 迭代，半小时量级 | `/feat` `/fix` `/dev` + 本协议 | **默认档** |
| 大活：多文件、有依赖、可并行子任务 | **Graph：pi-crew team run** | planner 拆任务 → parallel 并行 → reviewer/verifier 验收 → worktree 隔离 |

**升级判断标准（问自己一个问题）**：
> 这个任务能不能拆成「有明确依赖关系」的多个子任务？能 → 上 Graph；不能 → 保持 Loop。

**分工映射**（Graph 概念 ↔ 本项目能力）：Org Graph（谁管哪块）= pi-crew 的 team/roles（explorer/planner/executor/reviewer/verifier）；Work Graph（当前任务编排）= pi-crew workflow（chain / parallel / 依赖传递）。
## ★ CascView 参考数据源

**路径**：`D:\dev\d2r\cascview_cn\x64\Work\data\data\`

这是**原版 D2R 3.2 (build 92777) 解压**的数据,包含所有 .txt 和 .json 资源。

| 类别 | 路径 | 用途 |
|------|------|------|
| strings (i18n) | `local/lng/strings/*.json` | 物品名、技能名、任务名等多语言翻译 |
| excel (data tables) | `global/excel/*.txt` | 游戏数据表 (armor.txt / weapons.txt / misc.txt / skills.txt / etc.) |
| assets | `global/ui/`, `local/` | UI 资源、字体等 |

**重要数据举例**:
- `local/lng/strings/item-runes.json` — 33 个 runes (r01-r33) 的 enUS/zhCN/zhTW
- `local/lng/strings/item-names.json` — 基础物品名 + unique/set 物品
- `local/lng/strings/item-nameaffixes.json` — affix (前缀后缀) 翻译
- `global/excel/misc.txt` — rune code → 物品 type 映射
- `global/excel/itemtypes.txt` / `armor.txt` / `weapons.txt` — 基础物品数据

**遇到 SQLite database 缺数据时的修复流程**:
1. 找到 `D:\dev\d2r\cascview_cn\x64\Work\data\data\` 下对应 json/txt
2. 解析后批量 INSERT 到 `%LOCALAPPDATA%\D2RMarketplace\database\d2r_marketplace.db`
3. 参考 `src-tauri/examples/import_runes_from_cascview.rs` 写 importer
4. 重启 tauri dev 让 warmup cache 加载新数据

**importer 模板** (可复制):
- 读 cascview json: `serde_json::from_str(&std::fs::read_to_string(path)?)`
- 写 item_base: `INSERT OR REPLACE INTO item_base (code, profile_id, name_en, item_type, item_category) VALUES (?, ?, ?, ?, ?)`
- 写 localized_string: `INSERT OR REPLACE INTO localized_string (profile_id, namespace, string_key, language, text_value, source_path) VALUES (?, 'item_names', lower(?), ?, ?, 'cascview_cn/...')`

## ★ Phase 1-3 Protocol Refactor


**关键修复**：原 `stash/item.rs::read_single_item` 漏读 Page 3b 字段（D2SLib `Items.cs:283`），
导致 Page[0] 80 装备只能解析 47 个。新 `protocol::d2i::parser` 在 compact header 中
显式读 `ItemPage::read(reader)`，配合 `FieldSet` 版本分发表统一处理 v96/v105 差异。

**真实 d2i 集成测试结果**（`ModernSharedStashSoftCoreV2.d2i`，18 KB）：

| Phase | items 解析 | 3-char 干净码 |
|-------|-------|--------|
| Phase 1（贪婪，无 stat_list） | 1270（错误） | 22.8% |
| Phase 2.3（+stat_list） | 278 | 45.7% |
| Phase 3.1（+complete header） | 217（精准） | 45.2% |

## Migration Status

### Protocol Layer (Phase 1-3 ✅)

| Module | Status | Notes |
|--------|--------|-------|
| `core/bitio/` | ✅ Done | peek/align/slice |
| `core/encoding/` (Huffman) | ✅ Done | 20 tests |
| `core/result.rs` / `version.rs` | ✅ Done | ParseError + V96/97/98/105/111 |
| `protocol/common/item_*` | ✅ Done | ItemFlags/Mode/Location/Page/Quality + FieldSet |
| **`protocol/common/item_page.rs`** | ★ Done | **Page 3b 字段修复** |
| `protocol/common/stat.rs` + `stat_list.rs` + `stat_table.rs` | ✅ Done | 0x1FF terminator + sub-property expansion |
| `protocol/d2i/parser.rs` | ✅ Done | compact + complete header + chest-stackable trailer |
| `protocol/d2i/page*.rs` + `summary.rs` | ✅ Done | page header + multi-page + summary |
| `protocol/d2i/legacy/` | ⚠️ Compat | 21 files migrated from `stash/`, contains complete 1500-line header |
| `protocol/d2s/` | ✅ Done | header + attributes (61B) + skills |
| `data/stat_cost.rs` | ✅ Done | 420+ stat definitions + `build_stat_table()` |

### Business Layer

| Component | Status | Notes |
|-----------|--------|-------|
| Database Schema + CRUD | ✅ Done | SQLite via rusqlite |
| D2I page split/merge | ✅ Done | Real d2i fixture verified |
| Item read/write (stackable + equipment) | ✅ Done | Via `protocol::d2i::legacy` delegation |
| Item constants + lookup | ✅ Done | `data::items` |
| Magical properties (stat_list) | ✅ Done | Full parsing |
| Market pricing | ✅ Done | Ported from Python |
| Sell time calculator | ✅ Done | Unit tested |
| Trade rules | ✅ Done | Unit tested |
| Tauri commands | ✅ Done | 27 IPC commands registered |
| Character info (d2s) | ✅ Done | `read_character_info` / `list_characters` |
| Node.js fallback parser | ✅ Done | Auto-detects original project path |
| Frontend (web/) | 🔄 Basic | Needs full UI port |

## Where to Find What

- **New code should use**: `protocol::d2i::*` (parser, summary, page, common::*)
- **Legacy 1500-line complete header**: `protocol::d2i::legacy::item::skip_non_simple_complete_header`
  (new parser delegates to this for quality-specific fields)
- **Item constants**: `data::items::ITEM_CODE_MAP` (re-export from `protocol::d2i::legacy::constants`)
- **Stat table**: `data::stat_cost::build_stat_table()` → `protocol::common::StatTable`
- **Old `stash/` module**: ❌ Deleted. Use `protocol::d2i::legacy` instead.