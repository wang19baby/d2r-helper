# D2S 角色存档解析规格（当前代码状态）

> 本文档记录 `src-tauri/src/protocol/d2s/` 当前解析器的**已确认结论**与**未确定字段**。
> 证据标注：[字节] = 真实 .d2s 字节验证；[D2SLib] = dschu012/D2SLib 源码；[webpack] = D2R webpack chunk 3948；[py] = Python d2r-zero / construct_adapter。
> 主验证样本：`开心邪帝.d2s`（4126B，与 `src-tauri/tests/fixtures/xieedi.d2s` 字节一致）。

## 1. 文件顶层结构

已从真实字节验证的分段（开心邪帝.d2s）：

| 偏移 | 结束 | 段 | 说明 |
|---|---|---|---|
| `0x0000` | `0x000F` | Header | magic/version/filesize/checksum |
| `0x0010` | `0x0192` | Character | 角色基础数据（含名字） |
| `0x0193` | `0x02BC` | Quests `Woo!` | 任务进度 + 技能分配 |
| `0x02BD` | `0x030C` | Waypoints `WS` | 小站位域 |
| `0x030D` | `0x0340` | NPCs `w4` | NPC 对话/奖励消费 |
| `0x0341` | var | Attributes `gf` | 角色属性位流 |
| `0x0374` | `0x0393` | Mercenary `if` | 佣兵技能数据 |
| `0x0394` | var | Items `JM` | 角色全部物品（第一 JM 块） |
| `0x0DD3` | `0x0DD6` | 空 `JM` | `JM 00 00` 结束标记 |
| `0x0DD7` | var | Journey `jf` | 历程段 |
| `0x0FA1` | var | Corpse `kf` | 死尸段 |
| `0x1006` | EOF | 另一 `gf` | Golem/其余段 |

注意：这些偏移是**单样本验证**（开心邪帝.d2s），非全量协议布局；标准 v105 文件的段偏移可能不同，解析器实际按 marker 搜索而非固定偏移（见 §7）。

## 2. D2SHeader（105B header）

源码：`d2s/header.rs`。标准 D2R v105 layout：

```
0x00 magic        4B  0xAA55AA55 (LE 55 AA 55 AA)   ← 与 d2i 共用
0x04 version      4B  = 0x69 (105)
0x08 filesize     4B
0x0C checksum     4B  D2 滚动校验和
0x10 active_weapon 4B  0=主, 1=切换
0x14 menu_layout  4B  ← 语义未定 (见 §8)
0x18 class        1B  0=Amazon..7=Warlock
0x19 status_flags 1B  bit2=HC, bit3=Dead, bit4=Expansion
0x1A num_skills   1B  已分配技能点数
0x1B level        1B  ← 当前 D2SHeader 未读 (由 attributes stat 12 提供)
0x1C reserved     4B  全零
0x20 save_timestamp 4B  Unix 时间戳
0x24 unused       4B  0xFFFFFFFF
0x28 hotkeys      40B  10 × u32
0x50 left_mouse   4B  左键技能 ID
0x54 right_mouse  20B  5 × u32 (含换手)
0x68 end_marker   1B  = 0x00
─── 共 105B (0x69)
```

对应结构体字段：

```rust
pub struct D2SHeader {
    pub version_raw: u32,        // 0x04, 兼作 ProtocolVersion 映射
    pub filesize: u32,           // 0x08
    pub checksum: u32,           // 0x0C
    pub active_weapon: u32,      // 0x10
    pub name: String,            // compat 段启发式, 见 §3
    pub status_flags: u8,        // 0x19
    pub class: u8,               // 0x18
    pub num_skills: u8,          // 0x1A
    pub menu_layout: u32,        // 0x14  ← 语义未定
    pub save_timestamp: u32,     // 0x20
    pub location: [u8; 3],       // 0xA8..0xAB  ← 未验证
    pub hotkeys: [u32; 10],      // 0x28
    pub left_mouse_skill: u32,   // 0x50
    pub right_mouse_skills: [u32; 5], // 0x54
    pub end_marker: u8,          // 0x68
}
```

### 3. 角色名（启发式，未字节级验证）

compat 段（`0x69..0x193`）内按优先级尝试 4 种来源：
1. `file+0x12B`（d2emu mod 扩展名偏移）
2. `compat+0xC8`（标准 v105 布局）
3. `file+0x12C`（d2emu 兼容）
4. compat 段宽扫描：取最长的合理 ASCII/CJK 字符串

## 4. Attributes 段（gf，9-bit 位流）

源码：`d2s/attributes.rs`。起点常量 `ATTRIBUTES_OFFSET = 0x341`，marker `67 66`（"gf"），以 `0x1FF`（9-bit 全 1）终止。

每 stat：`9-bit stat_id + N-bit value`，位宽来自 `StatTable` 的 `CSvBits`（角色面板用 CSvBits，非物品 Save Bits）。

| id | 字段 | 位宽 | 备注 |
|---|---|---|---|
| 0 | Strength | 10 | |
| 1 | Energy | 10 | |
| 2 | Dexterity | 10 | |
| 3 | Vitality | 10 | |
| 4 | StatPoints | 10 | |
| 5 | NewSkills | 8 | |
| 6-11 | HP/MaxHP/Mana/MaxMana/Stamina/MaxStamina | 21 | Q8 值, 显示值 = raw/256 |
| 12 | Level | 7 | 角色等级来源 |
| 13 | Experience | 32 | |
| 14 | Gold | 25 | |
| 15 | GoldBank | 25 | |

已验证：开心邪帝.d2s 在真实字节上复算 strength=143/dex=463/vit=552/level=116 等全部 14 项。[字节]

## 5. 技能 / 小站 / 任务段

| 段 | 解析函数 | 结构 |
|---|---|---|
| Skills | `items::read_skills` + `parse_skills` | `if` 段后 30 字节，每字节 = per-class skill 等级 |
| Waypoints | `parse_ws_waypoints`（首选）/ `parse_waypoints_from_jf`（回退） | `WS(2B)+version(4B)+length(2B)+3×24B 难度块`；每块 ActI(9b)+ActII(9b)+ActIII(9b)+ActIV(3b)+ActV(9b) |
| Quests | `parse_woo` | `Woo!(4B)+payload_size(4B)+data_size(1B)+progression(1B)+288B`；3 难度 × 96B，每难度 5 act × quest uint16 位掩码 |
| NPC | `parse_w4` | `w4(2B)+block_type(1B)+6×8B`（3 难度 × 2 类 dialog/reward_consumed） |

## 6. 物品容器（D2SCharacter）

```rust
pub struct D2SCharacter {
    pub header: D2SHeader,
    pub attributes: CharacterAttributes,
    pub skills: Vec<u8>,              // 原始 30B
    pub skills_decoded: Vec<SkillEntry>,
    pub waypoints: WaypointSet,
    pub woo: WooQuestData,
    pub w4: W4DialogData,
    pub equipped: Vec<ParsedItem>,    // ItemMode::Equipped, bodyloc 1..=12
    pub belt: Vec<ParsedItem>,        // ItemMode::Belt
    pub backpack: Vec<ParsedItem>,    // Stored + page=Backpack
    pub cube: Vec<ParsedItem>,        // Stored + page=Mod(4)
    pub personal_stash: Vec<ParsedItem>, // Stored + page=MyStash
    pub merc: Vec<ParsedItem>,        // 第二个 JM 块
}
```

解析策略（`d2s/parser.rs`）：**best-effort**——header + attributes 必须成功，其余段找不到就空容器不报错。物品容器按语义过滤而非按位置。

### 两条 items 解析路径

| 路径 | 触发 | 格式 |
|---|---|---|
| 标准 `items::read_standard_items` | 默认（item 格式同 d2i，见 `docs/protocol-d2i.md` §6） | 顺序位流扫描，`try_parse_one` 逐 item |
| 魔改 `items_modified::read_items_with_quality` | `detect_modified_layout` 命中 | **12B 固定 stride**：`code[3] + pad + i_lvl + quality + 6B raw`，起于 `0xFB`，止于 `0x12B`（name 区），装备数 = `(0x12B-0xFB)/12` |

魔改 layout 关键偏移（xieedi.d2s / happy_manman.d2s 实测）：`MOD_FIRST_ITEM_OFFSET=0xFB`，`MOD_ITEM_LEN=12`，`MOD_NAME_OFFSET=0x12B`。该 layout 的 Status/ClassId/Level 等 header 字段不可信，attributes 段不存在。

### 佣兵物品（第二个 JM 块）

`read_merc_items`：定位 `merc_jm` marker，顺序扫描 + byte 对齐补充扫描，再 `associate_socketed_items` 关联镶嵌。

### 已确认的 item 位布局（KnownItemBitLayout）

```rust
pub fn known_item_bit_layout() -> KnownItemBitLayout {
    location_id_bit_offset: 35,   // mode 3b 之后
    equipped_slot_bit_offset: 38, // location 4b
    huffman_code_bit_offset: 53,  // flags(32)+version(3)+mode(3)+loc(4)+x(4)+y(4)+pg(3)
    socket_count_bits: 4,
    uid_bits: 32,
    ilvl_bits: 7,
    quality_bits: 4,
    stat_terminator: 0x1FF,
}
```

## 7. Marker 定位

`marker_offsets` 顺序扫描 `gf / if / JM / jf / kf`。⚠️ `"JM"` 字节对也会出现在 item flags 内部，必须从 `if` 区之后开始搜，否则误中伪 marker。

## 8. 未确定字段清单

### 🔴 P0 — 会导致解析错位
| 项 | 现状 | 证据 |
|---|---|---|
| **code index 位窗（10-bit）** | `76..85` 是强候选但未坐实：该位窗下项链/戒指/腰带/鞋/手套槽位 0 命中，与真实装备矛盾；替代位窗 70/79/81 能局部命中 `vip/rin/tgl/wae` 但 body 不自洽 | 开心邪帝.d2s 全局扫描 |
| **code→index 映射** | 依赖 `code.localeCompare()` 排序假设（`code_table_tmp.txt`），未与游戏内排序核对 | 工具 `generate_tables.cjs` |
| **item 精确起点/边界** | 104 items 仅 4 个高可信样本点（bit 9020/9588/17336/22235）；最可信 `7s8 @ bit 22235` 是**非 byte 对齐**（byte 2779 + 3bit），而 d2i `parser_v3` 扫描器只接受 byte 对齐候选，无法直接复用 | 验证文档候选 A-E |

### 🟠 P1 — body 分支语义
| 项 | 现状 |
|---|---|
| **advanced body 扩展区** | `multi_pic(4b)/class_specific(12b)/auto_affix/superior_bits/runeword_id(16b)/unique_id(16b)/v105_2` 完整分支未验证；`v105_2` 两样本不一致（7s8=0, 6bs=3） |
| **unique/set 分支还原** | `unique_id=1384 / runeword_id=3334` 未与 uniqueitems.txt 核对 |
| **property list 终止位** | `7s8` 在 512-bit 窗口扫到两个候选 0x1FF（bit 22604 / 22752），结束位在 22624/22776 间未唯一确定 |
| **y 行坐标位宽** | ~~3 vs 4 两文档矛盾~~ ✅ 2026-08-02 已确认 4b（webpack grid 4x4 + stackable amount 逐字节对拍） |

### 🟡 P2 — 结构内占位
| 项 | 现状 |
|---|---|
| **header 0x14** | 代码按 u32 `menu_layout` 读，但验证文档写 1 字节 `status_flags`——两处矛盾，语义未定 |
| **header 0x1B level** | `D2SHeader` 未读该字段（等级走 attributes stat 12） |
| **header 0xA8..0xAB location** | 注释称"三难度 Act 位置，bit7=active"，超 header 范围，未验证 |
| **0x1C reserved / 0x24 0xFFFFFFFF** | 用途未知 |
| **realm data 消费** | `bit16 → 消费 128 bits` 是简化验证，非确证 |
| **d2emu 魔改 layout** | 12B stride 探测是启发式（多层自检画像），仅 2 样本验证 |

### 🟢 P3 — 表级
| 项 | 现状 |
|---|---|
| **w4 段解析** | `parser.rs` 存在重复代码路径（`_w4` 误调 `parse_woo`），实际 `w4` 用 `find_marker` 重新定位——语义未完整确认 |
| **装备栏过滤** | `parse_equipped` 依赖 mode/location 位段正确性，随 P0 的位窗问题受牵连 |
