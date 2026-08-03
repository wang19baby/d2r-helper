# Phase 2 — 仓库高级搜索设计方案

## 需求

1. 按来源角色筛选（已有 `source_character`）
2. 按装备部位筛选（`slot_equipped`: 头盔/胸甲/武器/戒指...）
3. 按物品类型筛选（`item_kind`: 符文/宝石/护甲/武器...）
4. 按品质筛选（`quality`: 暗金/套装/稀有/魔法/普通）
5. 按名称/文本搜索（LIKE `item_name`）
6. 按需求等级搜索（需新增字段）
7. 按词缀属性搜索（需存储 stat 摘要）

## 实现方案

### P0 — 基础搜索（当前可做）

利用已有字段：`source_character`, `slot_equipped`, `item_kind`, `quality`, `item_name`

新增一个通用搜索命令替代多个 `warehouse_list_by_*`：

```rust
#[tauri::command]
fn warehouse_search(
    state: State<AppState>,
    source_character: Option<String>,
    item_kind: Option<String>,
    equipment_slot: Option<String>,  // slot_equipped
    quality: Option<String>,
    search_text: Option<String>,      // LIKE on item_name
    page_name: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<WarehousedItem>, String>;
```

SQL 生成：动态 WHERE 子句拼装，只加有值的条件。

### P1 — 需求等级搜索（需 migration）

`WarehousedItem` 加 `required_level: Option<i32>` 字段 + ALTER TABLE。
在 `extract_character_equipment` 填充（从 item stat 中解析 `item_levelreq` / `levelreq`）。

### P2 — 词缀属性搜索（大工程）

需在提取时解析 stat 列表 → 摘要为 JSON → 存入 `item_json`。
搜索时用 SQLite JSON 函数或 LIKE。

**格式示例：**
```json
{
  "level_req": 30,
  "stats": [
    {"id": 19, "name": "tohit", "value": 20},
    {"id": 93, "name": "ias", "value": 20}
  ],
  "resist": { "fire": 25, "cold": 25 },
  "skills": ["+1 所有技能"]
}
```

## 命令设计

合并现有 `warehouse_list`, `warehouse_list_by_page`, `warehouse_list_by_page_in_profile` 为一个统一搜索：

### 新增命令

```rust
#[tauri::command]
fn warehouse_search(...) -> Result<Vec<WarehouseSearchItem>, String>;
```

### 响应类型

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct WarehouseSearchItem {
    // ...所有 WarehousedItem 字段 +
    pub stat_summary: Option<StatSummary>,  // 解析后的 stat 摘要
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatSummary {
    pub level_req: Option<i32>,
    pub resist: HashMap<String, i32>,   // {"fire": 25, "cold": 10}
    pub skills: Vec<String>,             // ["+1 所有技能"]
    pub stats: Vec<StatEntry>,
}
```

### 前端 UI

仓库页面新增筛选栏：
- 角色下拉（`source_character` 去重列表）
- 部位下拉（`slot_equipped` 去重列表）
- 类型下拉（`item_kind` 去重列表）
- 品质下拉（固定：全部/暗金/套装/稀有/魔法/普通）
- 文本搜索框（搜索 `item_name`）
- 等级区间滑块（P1）

## 工作量估算

| 子任务 | 工作量 |
|--------|--------|
| P0: 通用搜索命令 + 前端筛选栏 | 2d |
| P1: 需求等级字段 + migration | 0.5d |
| P2: Stat 摘要提取 + JSON 存储 | 2d |
| P2: 词缀搜索 UI | 1d |
| **总计** | **5.5d** |
