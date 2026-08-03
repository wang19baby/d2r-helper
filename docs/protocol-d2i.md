# D2I 共享仓库解析规格（当前代码状态）

> 本文档记录 `src-tauri/src/protocol/d2i/` 当前解析器的**已确认结论**与**未确定字段**。
> 证据标注：[D2SLib] = dschu012/D2SLib；[webpack] = D2R webpack chunk 3948（Emscripten 编译，与 D2R.exe 同源）；[py] = Python d2r-zero。
> 主验证样本：`ModernSharedStashSoftCoreV2.d2i`（vanilla）+ 仙道轮回 mod stash（魔改路径）。

## 1. 文件顶层布局（best-effort）

`parse_file`（`d2i/parser.rs`）→ `split_pages`（`d2i/page.rs`）：

```
.d2i buffer
  │
  ▼ split_pages: 从 offset 0 起循环
  ├─ Page[0]: 64B PageHeader + item 数据
  ├─ Page[1]: ...（连续, 每页 size = header.page_size）
  ├─ ...
  └─ tail: 剩余字节（保留用于回写）
```

约束：`MAX_PAGES = 50`；每页要求 64B header 且 `magic == 0xAA55AA55`，否则停止分页。

```rust
pub struct Page {
    pub index: usize,
    pub offset: usize,
    pub size: usize,        // = header.page_size
    pub is_stackable: bool, // = header.is_stackable == 1
    pub data: Vec<u8>,      // 含 64B header
}
pub struct D2IFile {
    pub pages: Vec<Page>,
    pub items: Vec<ParsedItem>, // 主物品 + 镶嵌子物品
    pub tail: Vec<u8>,
}
```

## 2. PageHeader（64B）

源码：`d2i/page_header.rs`。

| 偏移 | 字段 | 大小 | 状态 |
|---|---|---|---|
| 0x00 | magic `0xAA55AA55` | u32 | ✅ 验证 |
| 0x04 | unknown1 | u32 | ❌ 未知 |
| 0x08 | unknown2 | u32 | ❌ 未知 |
| 0x0C | unknown3 | u32 | ❌ 未知 |
| 0x10 | page_size | u32 | ✅ 验证 |
| 0x14 | is_stackable | u8 | ✅ 0/1；魔改样本出现 2（见 §7） |
| 0x15 | unk0 | u8 | ❌ 未知 |
| 0x16 | unk1 | u8 | ❌ 未知 |
| 0x17 | unk2 | u8 | ❌ 未知 |
| 0x18 | reserved | 40B | ❌ 未知 |

**未知字段占 64B 的 73%**（4+4+4+1+1+1+40 = 55B）。

## 3. JM 段头（每页）

item 数据区以 `JM(2B) + u16 count(LE)` 起始（count 为 item 数提示，实际解析按位流顺序走，count 仅作参考）。

## 4. Item compact header（共享 d2s/d2i）

源码：`d2i/jm_reader.rs::try_parse_one`。`KnownItemBitLayout`（d2s）与本处一致。

```
bit  0-31  flags (u32)          ← ItemFlags, 见 §5
bit 32-34  version (3b)         5 = v105
bit 35-37  mode (3b)            ← jm_reader 映射见 §8
bit 38-41  location (4b)        bodyloc 1..12
bit 42-45  x (4b)               格子列
bit 46-49  y (4b)               格子行（4b 已验证: webpack grid 4x4 + amount=(py<<4)|px 逐字节对拍）
bit 50-52  page (3b)            0=Equipped 1=Backpack 5=MyStash 6=SharedStash
bit 53+    code (Huffman 4 字符)  如 "r01"/"gcv"/"7ar"
之后      socket_hint (3b)      实际未用
```

Ear 特例：`flags.ear && version != 5` 额外跳 8 bit；ear item 直接对齐返回（无 body）。

## 5. ItemFlags（32b，100% 确认）

源码：`common/item_flags.rs`。位序经 [webpack] 逐位验证，与 D2R 引擎一致：

| bit | 字段 | 说明 |
|---|---|---|
| 0-3 | unknown | 固定 0x0 |
| 4 | identified | |
| 5-10 | unknown | |
| 11 | socketed | |
| 12 | unknown | |
| 13 | new | |
| 14-15 | unknown | |
| 16 | is_ear | PvP 耳朵 |
| 17 | starter_item | |
| 18-20 | unknown | |
| 21 | simple_item | 1=堆叠物（无 body） |
| 22 | ethereal | |
| 23 | unknown | |
| 24 | personalized | |
| 25 | unknown | |
| 26 | given_runeword | |
| 27-31 | unknown | |

## 6. Non-compact body（装备物品）

源码：`d2i/jm_reader.rs::parse_noncompact_body`（顺序读，匹配 [py] `_scan_complete_item`）：

| 步骤 | 字段 | 位宽 | 条件 |
|---|---|---|---|
| 1 | item_id | 32 | 总是 |
| 2 | ilvl | 7 | 总是 |
| 3 | quality | 4 | 总是 |
| 4 | jump bits | 1+3 或 1+11 | multi_pic / class_specific 标志 |
| 5 | quality 专属 | 见下 | |
| 6 | runeword_id + prop_lists | 12+4 | flags.bit26 |
| 7 | ear / personalized | 7-bit 字符串 | flags.bit16 / bit24 |
| 8 | realm data | 1 + 128 | body 起始 |
| 9 | type-specific | 见下 | armor/weapon/other |
| 10 | socket_count | 4 | flags.bit11（4b 已验证, TC59 4 孔物品） |
| 11 | set mask | 5 | quality=5 |
| 12 | 主 stat list + bonus streams | var | 0x1FF 终止 |
| 13 | align + forward-scan resync | — | stat 流非干净终止时 |
| 14 | post-body realm data | 32b peek, bit16 → 128b | |
| 15 | align | — | |
| 16 | amount | 8 | stackable 页：`(py<<4)\|px`，否则 1 |

### quality 专属分支

| quality | 读取 |
|---|---|
| 7 Unique | `unique_id` 12b |
| 5 Set | `set_id` 12b |
| 4 Magic | prefix 11b + suffix 11b |
| 6 Rare | 16b + 6×(flag 1b + 11b) |
| 8 Crafted | 16b + 6×(flag 1b + 11b) |
| 1 Low / 3 Superior | 3b |

### type-specific

- armor（含 shield）：defense = `table[31].save_bits`，然后 max_durability，>0 时再 current + 1b
- weapon：max_durability，>0 时 current + 1b
- other：1 flag + 9b

## 7. Stat list（9-bit 编码）

源码：`common/stat_list.rs` + `common/stat.rs::ItemStat::read`。

```
循环:
  stat_id = read(9)
  stat_id == 0x1FF → 终止
  prop = StatTable[stat_id]
  param = read(prop.save_param_bits)
  value = read(prop.save_bits)  − prop.save_add   // save_bits==0 时默认读 9b [py]
```

特殊编码（[D2SLib] `ItemStat.Read`，EchoingStrike.d2s 验证）：

| 编码 | 字段 | 拆分 |
|---|---|---|
| descfunc=14（stat 188） | `+1 暴风雪` | SkillTab = param & 0x7；SkillLevel = value |
| encode=1 | `+N to Skill X` | SkillId = param；SkillLevel = value |
| encode=2 | `+X% 概率施放` | SkillLevel = param & 0x3f；SkillId = (param>>6) & 0x3ff |
| encode=3 | 充能技能 | 同上 + MaxCharges = (value>>8)&0xff；Value = value & 0xff |

结构体：

```rust
pub struct ItemStat {
    pub id: u16,
    pub param: u32,
    pub value: i64,
    pub skill_tab: Option<u8>,
    pub skill_level: Option<u16>,
    pub skill_id: Option<u16>,
    pub max_charges: Option<u8>,
}
pub struct StatList { pub stats: Vec<ItemStat> }  // 一条 0x1FF 终止的流
```

## 8. 未确定字段清单

### 🔴 P0 — 解析错位风险
| 项 | 现状 | 状态 |
|---|---|---|
| **unknown stat 默认 9 bits** | `save_bits==0` 时按 9b 读（[py] 行为），对 mod stash 对齐是关键，无官方依据 | ❌ 未定 |

> ✅ 2026-08-02 已解决（原 P0 三项，见代码修正）：
> - **y 行坐标 3 vs 4** → **4b 确认**：webpack grid 4x4；`amount=(py<<4)|px` 依赖 4b，`d2i_compare_*` 逐字节对比测试通过反证。3b 是 d2r-horadric-tools 单方错误说法。
> - **socket_count 4b vs 3b** → **4b 确认**：`jm_reader` 4b 读取 + TC59 `uhm ns=4` 4 孔全吸收；`KnownItemBitLayout.socket_count_bits` 已 3→4（纯展示常量）。
> - **mode 6 vs 4** → **6=Socket 确认**：TC59 4 个 jew 按 `mode==Socket` 全部吸收成功，而该枚举值只由 `u8_to_mode(6)` 产生，反证真实位流 mode=6；`ItemMode` 枚举已修正 `Socket=4→6`（web 端无数值依赖，`as u8` 仅 CLI 展示比较 Equipped/Belt，不受影响）。

### 🟠 P1 — 语义未验证
| 项 | 现状 |
|---|---|
| **PageHeader unknown1/2/3 + unk0/1/2 + reserved[40]** | 55B 完全未知 |
| **文件级 magic** | 分页按 page magic `0xAA55AA55` 判定；d2i 文件是否另有文件级头未确认（tail 语义未验证） |
| **is_stackable 判定** | 仅依赖 header 0x14 == 1；魔改样本出现 ==2（`detect_mod_stash` 探针） |
| **realm data（1+128b / peek bit16）** | [py] 行为移植，D2R 侧未独立确证 |
| **prop_lists 位宽** | runeword 4b shift + set mask 5b 的映射（`prop_lists |= 1 << (shift+1)`）为 [py] 移植，未与 D2SLib 核对 |
| **forward-scan resync** | stat 流非干净终止时 0-64B 前向扫描重对齐，是 [py] 的容忍策略，可能吞掉相邻 item |

### 🟡 P2 — 表级 / 编码
| 项 | 现状 |
|---|---|
| **num_sub_props 两套逻辑** | `stat_list.rs`/`d2s/items.rs` 用 `FieldSet::sub_property_count()` 硬编码（firemindam+1, coldmindam+2...），`d2i` 走表内 `num_sub_props`——6 处 stat（16/48/50/52/54/57）不一致 |
| **stackable amount 编码** | 读取 `(py<<4)\|px`；**回写**走 `px + realm` 2 字节原位修改 + `try_div_encoding`（DIV 编码只是备选猜测），`MAX_STACK=99` 是假设 |
| **is_pseudo_unverified** | `ParsedItem` 保留该标志标注"保守识别可能假阳性"，但 jm_reader 目前恒置 false——未实际使用 |
| **魔改 stash 检测** | `detect_mod_stash` 探针（Page[31].is_stackable==2 + count 失败率>50%）是启发式；魔改 item 走 `mod-stash-experimental` feature 降级路径，完整 mod 协议未逆向 |
| **chronicle 块** | `parsing-architecture.md` 提及 v105 unidentified unique/set 的 52b chronicle——当前 `parse_noncompact_body` 未见对应读取，是否已在 [py] 对齐流程中覆盖未确认 |

## 9. 相关数据表

| 表 | 位置 | 来源 |
|---|---|---|
| `StatTable`（512 项） | `common/stat_table.rs` | itemstatcost.txt + webpack MAGICAL_PROPS（save_add/encoding/descfunc 已补齐） |
| `ALL_ITEMS`（600+ code） | `d2i/legacy/game_items.rs` | armor.txt + weapons.txt + misc.txt |
| `MagicPrefix/Suffix` | `d2s/magic_affix.rs` | MagicPrefix.txt 等（运行时加载，失败回退硬编码） |
