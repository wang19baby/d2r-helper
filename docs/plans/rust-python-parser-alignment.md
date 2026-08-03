# Rust ↔ Python D2S Parser 输出对齐 — 开发计划

> 目标：Rust `d2r-marketplace-tauri` 的 D2S 解析输出与 `cli_construct` (d2r-zero) 完全一致——字段相同、结构相同、值相同。
>
> 范围：仅 D2S 解析（不含 D2I/写入）。验证样本：`开心邪帝.d2s`。

---

## 依赖图（总览）

```
Phase1（基础字段补齐）
  └─→ Phase2（物品细节展开）
  │     └─→ Phase3（镶嵌体系）
  └─→ Phase4（本地化名称表）
        └─→ Phase5（展示层渲染）
              └─→ Phase6（调试工具）
                    └─→ Phase7（结构化对齐验证）
```

Phase 1–4 无外部依赖，可并行推进。Phase 5 依赖 Phase 4 的本地化表。Phase 7 是最终验收。

---

## Phase 1 — 基础字段补齐（4 任务，无依赖）

**目标**：Rust `CharacterInfoResult` 补齐 Python 有 Rust 缺的 header/属性字段。

### 1.1 暴露耐力 (stamina/maxstamina)

| 位置 | 文件 |
|---|---|
| `CharacterAttributes` 新字段 | `protocol/d2s/attributes.rs` |
| 加入 `get_stamina()` / `get_max_stamina()` | 同上 |
| 暴露到 `CharacterInfoResult` (u32 × 2) | `commands/character.rs` |

Python 字段: `attr_val(10)` stamina, `attr_val(11)` max_stamina
Rust 已解析 stamina (attr id 10, 11) 但未暴露到 `CharacterInfoResult`。

### 1.2 暴露 save_timestamp → creation_time

`D2SHeader` 已有 `save_timestamp` 字段 (`header.rs:0x20`)。
- 在 `CharacterInfoResult` 添加 `creation_time: u32`（Unix timestamp）
- 不在此阶段做格式化——前端决定显示格式

### 1.3 添加 file_size

`CharacterInfoResult` 新增 `file_size: u32`
- 在 `read_character_info_inner` 中注入 `data.len()` → `binary_structure` 或顶层字段

### 1.4 添加 SHA-256 hash

`CharacterInfoResult` 新增 `file_hash: String`
- `use sha2::{Sha256, Digest}` 或现有 hash 工具
- Cargo.toml 可能需要加 `sha2` 依赖（当前项目可能已有）

---

## Phase 2 — 物品细节展开（4 任务，依赖 Phase 1）

**目标**：每个 `ParsedItem` / `StoredItemSummary` 包含 Python `_emit_item` 中展示的所有字段。

### 2.1 ParsedItem 添加防御/耐久/孔字段

Python 从 `_item_name_line` + `_emit_item` 输出:
- `ac`: 基础防御 (base defense)
- `maxac`: 最大防御 (defense range)
- `dur`: 当前耐久, `maxdur`: 最大耐久
- `sockets`: 孔数 (ItemFlags bit24)

**当前 Rust 状态**:
- `ItemFlags` 已有 `has_sockets` (bit 24) 但 `total_sockets` 字段在 `ItemHeader` 不存在
- 没有 `defense`/`durability`/`sockets` 作为结构化字段

**修改文件**:
| 文件 | 改动 |
|---|---|
| `protocol/common/item.rs` | `Item` struct 添加 `base_def`, `max_def`, `cur_durability`, `max_durability`, `total_sockets`, `item_level` |
| `protocol/d2i/legacy/item.rs` | 在 `skip_non_simple_complete_header` / `read_item_body` 中填充这些字段 |
| `protocol/d2s/items.rs` | `read_standard_items` / `try_read_equipped_item` 填充这些字段 |

**数据来源**: D2S bit-stream 中:
- socket count: 在 item body 中 4-bit 字段 (位置需确认, D2R 在 compact 段)
- defense/durability: 属于 item stat_lists 的 stat 流 (stat 31=防御, 72=耐久)
- item_level: 在品质段 ilvl 字段 (Python `_item_name_line` 用)

### 2.2 实现基础伤害计算器

Python `_base_damage(code, eth)`: 查询 `WEAPON_BASE` 字典 (来自 weapons.txt)。

Rust 需要:
- 从 weapons.txt 导入武器基础数据 (已有 `data::items.rs`?)
- 或从游戏数据 JSON 加载
- 新增 `fn base_damage(code: &str, ethereal: bool) -> Option<DamageRange>`

### 2.3 实现属性需求计算器

Python 从 armor.txt/weapons.txt 读 `reqstr`/`reqdex` 字段。
Rust 需要:
- 从 armors.txt/weapons.txt 导入需求字段
- 新增 `fn requirements(code: &str) -> Option<Requirements>` (str/dex/level)
- 注意无形 `-10` 需求修正 (Python 在 `_emit_item` 中: `无形-10 需求`)

### 2.4 扩展 StoredItemSummary 输出全部字段

`StoredItemSummary` (character.rs:36-64) 当前字段:
```rust
pub struct StoredItemSummary {
    pub code: String,
    pub name_zh: Option<String>,
    pub name_en: Option<String>,
    pub quality: Option<String>,
    pub page: u8, pub mode: u8, pub pos_x: u16, pub pos_y: u16,
    pub amount: u32,
    // item_level, defense, durability, sockets, runeword, unique/set id, magic affixes, base damage, requirements, …
}
```

需新增字段:
```rust
    pub item_level: Option<u8>,            // 物品等级
    pub base_defense: Option<u16>,          // 基础防御
    pub max_defense: Option<u16>,           // 最大防御
    pub cur_durability: Option<u16>,        // 当前耐久
    pub max_durability: Option<u16>,        // 最大耐久
    pub total_sockets: Option<u8>,          // 插槽数
    pub base_damage_1h_min: Option<u16>,    // 单手最小伤害
    pub base_damage_1h_max: Option<u16>,    // 单手最大伤害
    pub base_damage_2h_min: Option<u16>,    // 双手最小伤害
    pub base_damage_2h_max: Option<u16>,    // 双手最大伤害
    pub req_strength: Option<u16>,          // 力量需求
    pub req_dexterity: Option<u16>,         // 敏捷需求
    pub req_level: Option<u16>,             // 等级需求
    pub is_ethereal: bool,                  // 无形
    pub is_runeword: bool,                  // 符文之语
    pub unique_id: Option<u16>,             // 暗金ID
    pub set_id: Option<u16>,                // 套装ID
    pub runeword_id: Option<u16>,           // 符文之语ID
    pub magic_prefix_ids: Vec<u16>,         // 魔法前缀ID列表
    pub magic_suffix_ids: Vec<u16>,         // 魔法后缀ID列表
    pub socketed_items: Vec<StoredItemSummary>, // 镶嵌物品
    pub stat_lines: Vec<String>,            // 属性摘要文本
```

**⚠️ 关键依赖**: 此步骤依赖 Phase 2.1–2.3 的字段定义和数据源。

---

## Phase 3 — 镶嵌体系（3 任务，依赖 Phase 2）

### 3.1 ParsedItem 跟踪 socketed_items 列表

Python 通过 bit-stream 中 mode=6 (socket) 后续 items 识别镶嵌物。
Rust `read_standard_items` 的 items 列表已有 mode 信息，但未关联 parent item。

方案:
- 在 `read_standard_items` 中识别 mode=6 item，向前匹配最近的 socketed item
- 或在 `D2SCharacter::parse` 中建立 `item.socketed_by` 关系

### 3.2 实现符文之语组合推导

Python `_item_name_line`: 扫描 socketed rune codes → 按组合匹配 runeword。

Rust:
- 从 runes.txt 导入 rune combo → runeword_name 映射
- 新增 `fn match_runeword(socketed_codes: &[&str]) -> Option<String>`
- 对 runeword_id=0 但实际符合组合的 item 做推导

### 3.3 StoredItemSummary 增加 sockets 子项

`StoredItemSummary` 新增 `socketed_items: Vec<StoredItemSummary>`。

---

## Phase 4 — 本地化名称表（4 任务，无依赖，可并⾏ Phase 1–3）

### 4.1 构建中文小站名称表

Python: `WP_BIT_NAMES_ZH` (models.py:150-160) — 6+5+6+3+6 = 26 个小站中英文名。

Rust: `protocol/d2s/parser.rs` 或 `commands/character.rs` 添加常量数组:
```rust
const WP_NAMES_ZH: [[&str; 6]; 5] = [
    ["罗格营地", "冰冷之原", "石块旷野", "黑暗森林", "黑色荒地", "泰摩高地"],
    // ...
];
```

### 4.2 导入中文任务名

Python: `ACT_QUEST_NAMES_ZH` (models.py:218-250) — 5幕 × ~6 quest。

Rust:
- 从 `quests.json` 导入（该项目已有此文件在 extracted_data/）
- 或硬编码 fallback 表

### 4.3 构建技能树位置 + 中文名表

Python: `_SKILL_TREE` dict (skill_id → (tree, row, col)) + `_SKILL_NAMES_ZH`。

Rust: 从现有 `skillcalc.json` (+ skill_data) 构建 `HashMap<u16, (u8,u8,u8)>` + 中文名表。

### 4.4 导入中文魔法词缀名

Python: `_AFFIX_ZH` / `_AFFIX_EN` (magicprefix.txt / magicsuffix.txt)。

Rust: 从 `magicprefix.txt` / `magicsuffix.txt` 导入前缀/后缀中文名。共 ~1000 条。可先用 JSON 固化。

---

## Phase 5 — 展示层渲染（6 任务，依赖 Phase 4）

### 5.1 物品名行完整格式

实现 Rust 版的 `_item_name_line`:
```
[code] 中文名 (英文名)  [品质]  ilvl=xx
```
含:
- 代码 + 中文名 + 英文名回退
- 品质标识 (暗金≥套装≥亮金≥魔法≥超强≥普通)
- 装备类型 (头盔/铠甲/武器/…)
- 孔数 (孔:N)
- 无形标识
- 符文之语名
- 暗金/套装名
- 物品等级 (ilvl)
- 位置坐标

### 5.2 品质详情格式化

Python `_emit_item`:
- 防御: `防御: 基础-N / 实际-N
- 耐久: `耐久: cur/max`
- 基础伤害: `伤害: 1h min-max / 2h min-max`
- 需求: `需求: 力量N / 敏捷N / 等级N`
- stat_lines: 逐行属性输出

### 5.3 腰带网格渲染

Python `_render_belt`: 4×4 grid, 行4(底部)→1(顶部),  potion 简码。

Rust 需实现:
- 按 px 排序 belt items
- 4列布局渲染

### 5.4 技能树网格渲染

Python `_show_skills`: 3树×行列 + 等级条 █。

Rust:
- 用 `_SKILL_TREE` (Phase 4.3) 定位技能到 grid
- 渲染文本输出

### 5.5 W4 对话状态渲染

Python `_show_w4`: per-difficulty NPC dialog + reward consumed。

### 5.6 任务完成度渲染

Python `_show_woo`: per-difficulty act + quest ✓/✗。

---

## Phase 6 — 调试工具（3 任务，低优先）

### 6.1 逐位转储 (--bits)

Python `_dump_item_bits` 输出每个 bit 的字段解释。
Rust 版本可以用 bitio reader 逐字段 dump。

### 6.2 物品列表表格

Python bits 模式的表格输出:
```
[idx]  off  len  ver  x y  pos    code       name      flags  stats
[ 0]   off=...  len=...  ver=...  x=...  y=...  page  code  name  flags
```

### 6.3 监控模式 (--watch)

Python `_watch_loop`: 轮询文件 SHA-256，变化时自动重解析。

---

## Phase 7 — 结构化对齐验证（4 任务，Phase 7 验收）

### 7.1 Python 导出规范 JSON schema

在 `cli_construct.py` 的 `--json` 模式增强输出:
- 包含所有字段 (目前的 `_header_to_dict` 不够全)
- 输出 `_show_jm` 的结构化版本 (items 完整对象)
- 输出 `_show_woo/ws/w4/skills` 的结构化版本

产出: `python_cli_construct_schema.json` + 开心邪帝.d2s 的真实输出快照。

### 7.2 Rust 实现 JSON 模式输出对齐

- 调整 `CharacterInfoResult` 字段名/类型匹配 Python 的 JSON 输出
- 或新增 `CliConstructOutput` struct 专门用于对齐输出

### 7.3 编写自动化对比测试

Python 脚本: 分别调 Rust binary (或模拟 HTTP) 和 Python cli_construct，JSON diff。

`tools/diff_parser_output.py`:
```python
# 1. Run: cargo run -- read "开心邪帝.d2s" --json
# 2. Run: python -m d2r_zero.cli_construct "开心邪帝.d2s" --json
# 3. deep-diff JSON objects → report mismatch paths
```

### 7.4 验证开心邪帝.d2s 完全一致

最终验收: 零差异。所有字段类型、值、嵌套结构一致。

---

## 执行顺序建议

### 推荐路线（最短路径到 Python 等价输出）

```
Phase 1.1–1.4  (基础字段, 2天)   → 低风险直接做
Phase 4.1–4.4  (本地化表, 3天)   → 纯数据导入, 可独立于物品逻辑
Phase 2.1–2.2  (物品字段展开, 3天) → 核心, 需 bitstream 深入
Phase 3.1–3.3  (镶嵌体系, 2天)   → 依赖 2.1
Phase 2.3–2.4  (需求+输出, 2天)  → 依赖 2.1
Phase 5.1–5.6  (展示渲染, 4天)   → 依赖 Phase 4
Phase 7.1–7.4  (对齐验证, 2天)   → 验收
Phase 6.1–6.3  (调试工具, 3天)   → 最低优先, 可选
```

**合计**: ~18 天 (Phase 1-5+7), 含调试 ~21 天。

### 快速见效路线（按周切分）

| 周次 | 交付 |
|---|---|
| Week 1 | Phase 1 (基础补齐) + Phase 4 (本地化表) |
| Week 2 | Phase 2 (物品展开) + Phase 3 (镶嵌) |
| Week 3 | Phase 5 (展示渲染) + Phase 7 (对齐验证) |
| Week 4 | Phase 6 (调试工具, 可选) |

---

## 风险点

| 风险 | 影响 | 缓解措施 |
|---|---|---|
| bitstream 中 socket/defense/durability 字段定位不准确 | Phase 2.1 卡住 | 先用 Python jm_parser 跑开心邪帝.d2s 定位具体 bit 偏移; Rust 端加 gstack/diag 验证 |
| 数据表来源不一致 (weapons.txt/armors.txt vs Python 提取) | Phase 2.2–2.3 数据不一致 | 统一使用同一份 txt-json 数据源 (Python 的 data/test/txt-json/) |
| skills 树位置数据 (SKILL_TREE) 在 Rust 端无现成来源 | Phase 4.3/5.4 卡住 | 将 Python 的 skillcalc.json 导入 Rust；或提取 Python 的 dict hardcode |
| D2S items 的 mode=6 socketed item 关联逻辑不明确 | Phase 3.1 卡住 | 解析完后遍历 items, 从当前 item 的 payload 偏移 + 下一个 item 偏移推导; 先只在 parsed items 列表标注 parent |
| D2S items 的 stat_lists 解析漏项 (magic affix ID 丢失) | Phase 2.4 词缀ID缺失 | stat_lists 中 stat id 75/76/77/78/79 分别对应 prefix/suffix; 检查 Rust stat_list reader 是否捕获 |
