# D2R Marketplace 前后端接口与架构评审报告

日期: 2026-07-09

---

## 1. 前后端接口对应表

### 1.1 完全匹配 (32 个)

| 后端 Command | 前端调用文件 | 参数字段 | 返回类型 | 状态 |
|---|---|---|---|---|
| `get_balance` | App.tsx, Home.tsx, useToast.ts | 无参 | `i64` | ✅ |
| `get_app_config` | Characters.tsx, Config.tsx, StashManager.tsx | 无参 | `AppConfigResponse` | ✅ |
| `update_save_folder` | Config.tsx(×2) | `saveFolder` | `()` | ✅ |
| `set_game_root` | Config.tsx | `gameRoot` | `AppConfigResponse` | ✅ |
| `set_active_mod` | Config.tsx | `activeMod` | `AppConfigResponse` | ✅ |
| `set_game_version` | Config.tsx | `gameVersion` | `AppConfigResponse` | ✅ |
| `set_language` | Config.tsx | `language` | `AppConfigResponse` | ✅ |
| `set_stash_grid_size` | Config.tsx | `size` | `()` | ✅ |
| `set_grid_sizes` | Config.tsx | `backpackCols/Rows`, `cubeCols/Rows` | `AppConfigResponse` | ✅ |
| `switch_profile` | Config.tsx | `profileId` | `AppConfigResponse` | ✅ |
| `reimport_game_data` | Config.tsx | 无参 | `AppConfigResponse` | ✅ |
| `get_import_progress` | Config.tsx, App.tsx(×2) | 无参 | `ImportState` | ✅ |
| `delete_profile` | — | — | — | 注册未用 |
| `diagnose_zh_tw` | Config.tsx | 无参 | `LocaleDiagnosis` | ✅ |
| `read_stash` | Inventory.tsx, StashManager.tsx, Warehouse.tsx | 无参 | `StashReadResult` | ✅ |
| `list_backups` | Config.tsx | 无参 | `Vec<BackupEntry>` | ✅ |
| `create_stash_backup` | Config.tsx | 无参 | `BackupResult` | ✅ |
| `restore_backup` | Config.tsx | `timestamp` | `BackupResult` | ✅ |
| `get_listed_items` | Home.tsx, Inventory.tsx, Listings.tsx, Catalog.tsx | 无参 | `Vec<ListedItemResult>` | ✅ |
| `list_item` | SellModal.tsx | `itemName, itemCode, itemKind, quantity, unitPrice, stashFile` | `()` | ✅ |
| `sell_item` | Listings.tsx | `itemId` | `i64` (余额) | ✅ |
| `buy_item` | BuyModal.tsx | `itemName, itemType, tokenPrice, qty` | `BuyResult` | ✅ |
| `cancel_listing` | Listings.tsx | `listingId` | `()` | ✅ |
| `update_listing_price` | Listings.tsx | `itemId, newUnitPrice` | `bool` | ✅ |
| `get_price_suggestion` | SellModal.tsx | `itemName, itemKind` | `PriceSuggestionResult` | ✅ |
| `get_transactions` | Home.tsx, History.tsx | `limit, txType` | `Vec<HistoryEntry>` | ✅ |
| `list_characters` | Characters.tsx, StashManager.tsx | `dir` | `Vec<String>` | ✅ |
| `read_character_info` | Characters.tsx, StashManager.tsx | `path` | `CharacterInfoResult` | ✅ |
| `get_localized_skill_texts` | SkillTree.tsx | `language` | `HashMap<String,String>` | ✅ |
| `get_grail` | Grail.tsx | 无参 | `GrailProgress` | ✅ |
| `toggle_grail` | Grail.tsx | `itemKey, found` | `()` | ✅ |
| `find_runewords` | RunewordCalc.tsx | `ownedRunes` | `Vec<...>` | ✅ |

### 1.2 后端注册但前端未调用 (4)

| 后端 Command | 签名 | 状态 |
|---|---|---|
| `export_item` | `(item_name, item_code, item_type, character_file)` | ❌ 死代码 |
| `import_item` | `(item_id)` | ❌ 死代码 |
| `list_available_items` | `()` | ❌ 死代码 |
| `read_stash_file` | `(path)` | ❌ 死代码 |

### 1.3 类型精确性问题

| 位置 | 后端类型 | 前端类型 | 问题 |
|---|---|---|---|
| `warehouse_list` → `imported_at` | `String` (非 null) | `string` | ✅ 一致 |
| `warehouse_list` → `notes` | `String` (非 null) | `string` | ✅ 一致 |
| `buy_item` → `item_type` 参数名 | `String` | 传 `item.item_kind` | ⚠️ **命名误导**：参数叫 `item_type`，语义是 `item_kind` |
| `HistoryEntry` → `token_amount` | `i64` | `number` | ✅ Tauri 序列化自动转 |
| `StashItem` → `socketed_items` | `Vec<StashSocketedItemInfo>` | `StashSocketedItem[]` | ✅ 字段完全一致 |

### 1.4 结论

**前后端接口整体质量高**，47 处 IPC 调用中 43 处活跃调用完全匹配。4 个死命令建议清理。

---

## 2. 后端架构评审

### 2.1 模块依赖图（当前）

```
main.rs / lib.rs (Tauri 入口)
├── commands/     (Controller 层)     ← 依赖所有下层
│   ├── config.rs, character.rs
│   ├── stash.rs, warehouse.rs
│   ├── marketplace.rs, history.rs
│   ├── grail.rs, runeword_calc.rs
│   └── balance.rs, character_equip.rs
├── market/       (领域服务层)
│   ├── pricing.rs    — 定价规则
│   ├── sell_time.rs  — 售卖时长规则
│   └── trade_rules.rs — 交易规则
├── database/     (数据访问层)
│   ├── db.rs     — SQLite 操作 (~100+ 方法)
│   └── models.rs — 数据模型
├── resource/     (资源管理层)
│   ├── importer.rs   — TXT/JSON → SQLite
│   ├── queries.rs    — 查询封装
│   ├── resolver.rs   — 名称解析
│   ├── tooltip.rs    — Tooltip 格式化
│   └── import_task.rs — 后台导入任务
├── protocol/     (协议解析层)   ← 无外部依赖
│   ├── d2i/          — 物品容器解析
│   ├── d2s/          — 角色存档解析
│   └── common/       — 共享类型
├── data/         (数据定义层)
│   ├── stat_cost.rs  — Stat 位宽表
│   └── stat_loader.rs — 运行时加载器
└── core/         (基础设施层)   ← 无外部依赖
    ├── bitio/        — 位流读写
    ├── encoding/     — Huffman 编码
    ├── version.rs    — 协议版本
    └── result.rs     — 统一错误类型
```

### 2.2 DDD 评估

#### ✅ 做得好的

1. **`core/` 层无外部依赖** — 纯基础设施（位流、编码、枚举），复用度高
2. **`protocol/` 层独立** — d2i/d2s 解析器不依赖任何业务模块，可独立测试
3. **`resource/` 的 profile 隔离设计** — 所有查询强制带 `profile_id`，多 mod 多版本资源天然隔离
4. **`market/` 逻辑与 `database/` 分离** — 定价规则是纯函数 (`pricing.rs`)，不依赖数据库
5. **`data/stat_cost.rs` → `stat_table.rs` 模式** — 配置与读取分离清晰

#### ⚠️ 待改进的

1. **`commands/` 层过厚** — Controller 混入了大量业务逻辑（特别是 `marketplace.rs` 的 `buy_item`/`list_item` 中直接嵌入 stash 文件读写和位流操作）。应为：`commands` → `market::service` → `database` → `protocol`

2. **`database/db.rs` 上帝对象** — 单个文件包含 100+ 方法，混合了 `get_transactions`、`add_virtual_item`、`get_grail_progress`、`get_config` 等完全不相关功能。**违反单一职责原则**。

3. **领域模型贫血** — `database/models.rs` 中的 `VirtualItem`、`Transaction`、`WarehousedItem` 都是纯粹的 getter/setter 结构体，没有领域行为。`market/pricing.rs` 的纯函数游离在模型之外。

4. **`marketplace.rs` 引用 `protocol::d2i::legacy::*`** — Command 层直接依赖协议解析细节。`buy_item` 中直接调用了 `split_legacy_d2i_pages`、`update_stash_items`、`reassemble_d2i`。这应该是 StashService 的职责。

5. **跨模块耦合** — `commands/config.rs` 引用了 `protocol::d2i::legacy::resource_manifest`，说明资源扫描逻辑和协议解析未分离。

6. **无 Repository 抽象** — `commands/` 直接持有 `Database` 的 Mutex 锁，业务层无法脱离 SQLite 测试。应引入 `trait StashRepository { fn read(...) }` / `trait MarketRepository { fn get_balance(...) }`。

7. **`State<AppState>` 模式耦合** — 所有 Command 都通过 `state.db.lock()` 获取数据库引用，导致：
   - 无法对单个 command 做单元测试（需要真实 SQLite 或 mock）
   - 锁的粒度是整个 Database，而不是单表

### 2.3 具体改善建议

#### P1 — 拆分 Database 上帝对象

```
database/
├── mod.rs
├── db.rs               ← 只保留连接管理和 Schema 初始化
├── models.rs            ← 数据模型
├── repository/
│   ├── mod.rs
│   ├── stash_repo.rs    ← 仓库物品 CRUD
│   ├── market_repo.rs   ← 市场交易/余额
│   ├── character_repo.rs ← 角色/技能查询
│   └── config_repo.rs   ← 配置 KV
```

#### P2 — 抽离 StashService

```
commands/stash.rs        ← 只做参数校验 + IPC 响应格式化
↓
services/
├── mod.rs
├── stash_service.rs     ← 调用 protocol + database::repository
├── market_service.rs    ← 调用 pricing + inventory
└── warehouse_service.rs ← 调用 database::repository
```

#### P3 — 清理死代码

删除 `export_item`、`import_item`、`list_available_items`、`read_stash_file` 的注册和实现（除非有计划在后续使用）。

#### P3 — 统一 buy_item 参数命名

`item_type` → `item_kind`，避免与 D2 的 `item type` 概念混淆。

### 2.4 架构评分总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 分层清晰度 | ⭐⭐⭐⭐ | core→protocol→data→resource→market→commands 方向明确 |
| 依赖方向 | ⭐⭐⭐⭐ | 无反向依赖（没有下层引用上层） |
| 模块内聚 | ⭐⭐⭐ | database 上帝对象降低评分 |
| 领域模型 | ⭐⭐ | 模型贫血，缺少行为封装 |
| 可测试性 | ⭐⭐ | 数据库锁模式导致单元测试困难 |
| 前后端接口 | ⭐⭐⭐⭐⭐ | 47 处 IPC 映射准确，类型一致 |

---

## 3. 总结

**前后端通信** 质量很高，snake_case ↔ camelCase 转换正确，返回类型一一对应。建议删除 4 个死命令并重命名 `buy_item.item_type` 参数。

**后端架构** 选择了务实的层状架构而非纯 DDD，这在规模和场景下是合理取舍。主要问题在 `database/db.rs` 上帝对象和 `commands/` 过厚的业务逻辑。建议逐步抽离 Service 层和 Repository 接口，提升可测试性和维护性。
