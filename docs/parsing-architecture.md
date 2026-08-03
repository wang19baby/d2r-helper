# D2I 装备解析引擎 — 架构与数据流

## 整体架构

```
.d2i file
  │
  ▼
Page split ───→ D2IPages { pages[], tail }
  │
  ▼
read_stash_items_from_page()
  │
  ▼
for each item in stream:
  read_single_item()
    │
    ├─► simple_item=true  ──→ 堆叠物品（符文/宝石）→ 快速路径
    │
    └─► simple_item=false ──→ 装备物品 → skip_non_simple_item_body()
                                       │
                                       └─→ read_magic_properties()
                                       └─→ read_magic_properties_limited()
```

## 调用链

```
read_all_stash_items(pages)          lib 公开接口
  └─ read_stash_items_from_page(page)  解析单页
       └─ for _ in 0..JM_count:
            read_single_item(reader, on_stackable_page)
              ├─ 32-bit header
              ├─ version + location + item_type (Huffman)
              ├─ [if !simple] skip_non_simple_item_body()
              │    ├─ id(32) + level(7) + quality(4)
              │    ├─ multi_pic(1+3) + class_specific(1+11)
              │    ├─ quality-specific (set_id 12 / unique_id 12 / magic 11+11 / rare 8+8+...)
              │    ├─ given_runeword(12+4)
              │    ├─ personalized (0~128 bits)
              │    ├─ defense(11) [armor/shield]
              │    ├─ durability(8+9) [armor/weapon/shield]
              │    ├─ v105 stackable(1 + optional 9)
              │    ├─ sockets(4)
              │    ├─ plist_flag(5) [set]
              │    ├─ read_magic_properties()  ← ★ 核心：stat列表
              │    ├─ [if set] for plist bits ≥ 0..5:
              │    │    read_magic_properties_limited(25)
              │    └─ [if runeword] read_magic_properties_limited(25)
              ├─ [if v105+unidentified+unique/set] chronicle(52)
              ├─ [if has_trailer] chest_stackable(1 + 8)
              └─ align()
```

## 逐段位宽明细

### SECTION 1 — Item Header (32 bit)

| 字段 | 位宽 | 说明 |
|------|------|------|
| b0~b3 | 4 | 固定 0000 |
| identified | 1 | 0=未辨识 |
| b5~b10 | 6 | 固定 000100 |
| socketed | 1 | 是否有孔 |
| b12 | 1 | |
| new | 1 | |
| b14~b15 | 2 | |
| is_ear | 1 | |
| starter_item | 1 | |
| b18~b20 | 3 | |
| **simple_item** | **1** | **★ 关键：1=堆叠物，0=装备** |
| ethereal | 1 | 是否无形 |
| b23 | 1 | |
| personalized | 1 | 是否带角色名 |
| b25 | 1 | |
| given_runeword | 1 | 是否符文之语 |
| b27~b31 | 5 | |

### SECTION 2 — Version + Location + Type (fixed)

| 字段 | 位宽 | 说明 |
|------|------|------|
| version | 3 | 2 = v105 |
| location_id | 3 | |
| equipped_id | 4 | |
| **position_x** | **4** | 格子列 |
| **position_y** | **4** | 格子行 |
| alt_position_id | 3 | |
| item_type | ~40 | Huffman编码 4字符 |
| nr_of_sockets_hint | 1/3 | simple=1bit, 装备=3bit |

### SECTION 3 — skip_non_simple_item_body (仅装备)

| 字段 | 位宽 | 条件 |
|------|:----:|------|
| item_id | 32 | 总是 |
| level | 7 | 总是 |
| quality | 4 | 总是 |
| multi_pic | 1+3 | 1+3bit 如果 multi_pic==1 |
| class_specific | 1+11 | 1+11bit 如果 class_specific==1 |
| **quality-specific** | 变量 | Low=3, Normal=0, Super=3, Magic=22, Set=12, Unique=12, Rare/Crafted=54 |
| given_runeword | 12+4 | 如果 given_runeword |
| personalized | 0~128 | 如果 personalized |
| defense | 11 | 装甲/盾牌 |
| durability | 8+9 | 装甲/武器/盾牌 |
| v105 stackable | 1+9 | 第1位总是，+9仅堆叠物代码 |
| sockets | 4 | 如果 socketed |
| **plist_flag** | **5** | **套装专用，标识部分套装奖励** |

### read_magic_properties (stat 序列)

```
循环结构:
  ① 读9-bit stat_id
  ② stat_id == 0x1FF → 终止
  ③ stat_id >= MAGICAL_PROPS.len() → 扫描寻找0x1FF（最多256位）
  ④ sb==0 && sp==0 → 跳过1bit，继续
  ⑤ 正常读取: for np in num_sub_props:
       [if sp>0] skip_param_bits
       read_value_bits = sb (或 8 或 1)
```

| 字段 | 位宽 | 说明 |
|------|:----:|------|
| stat_id | 9 | 来自 MAGICAL_PROPS 表索引 |
| param | sp | save_param_bits |
| value | sb | save_bits（含 save_add 偏移） |
| num_sub_props | 1~3 | 重复读取 value 的次数 |

### Section 3b — Chronicle (v105)

| 字段 | 位宽 | 条件 |
|------|:----:|------|
| monster_id | 16 | identified=false 且 quality=5/7 |
| timestamp | 32 | |
| padding | 4 | |

### Section 4 — 结尾

| 字段 | 位宽 | 条件 |
|------|:----:|------|
| chest_stackable | 1+8 | on_stackable_page=true 或 simple_item=true |
| byte_align | 0~7 | |

## 关键表

| 表名 | Rust 位置 | 来源 TXT |
|------|-----------|----------|
| `MAGICAL_PROPS` | `magical_props.rs` | `itemstatcost.txt` |
| `ALL_ITEMS` | `game_items.rs` | `armor.txt` + `weapons.txt` + `misc.txt` |
| `ITEM_CODE_MAP` | `constants.rs` | `armor.txt` + `weapons.txt` + `misc.txt` |
| `ITEM_INVENTORY_SIZES` | `item_sizes.rs` | `inventory.txt` / `armor/weapon/misc` |
| `SET_BONUSES` | `set_items.rs` | `setitems.txt` + `sets.txt` |
| `MAGICAL_PROPS` 的 套装奖励 | 内嵌在代码中 | `properties.txt` |

## 数据依赖链

```
itemstatcost.txt ──► MAGICAL_PROPS[stat_id] = (sb, np, sa, sp, signed, encoding)
                  用于 stat 序列解析的位宽

armor.txt
weapons.txt  ──► ALL_ITEMS[code] = (code, name, is_armor, is_weapon, is_shield)
misc.txt          用于 item_type?代码查名称 + 类别判断

properties.txt ──► 影响 MAGICAL_PROPS 的 num_sub_props
                  多stat合一的属性（如 dmg-fire = firemindam + firemaxdam）

setitems.txt  ──► SET_BONUSES 用于套装奖励 stat 列表
sets.txt
```

## 当前已发现的问题（2026-07-02）

### ✅ 已修复
1. **chest-stackable trailer** — 装备页上不再读取尾部1+8b，amount=1
2. **抗性位宽 8→9** — `magical_props.rs` 的 6 条抗性 stat 匹配 mod 的 `itemstatcost.txt`

### ❌ 待修复
**Item[0] 体解析偏移** — 套装物品 `skip_non_simple_item_body`+ 后续对齐消耗 312bit，但实际应为更长。导致 3 个伪物品（~152 字节伪像）后才重新对齐到真实物品。

**最可能的原因（按概率排序）：**
1. **套装奖励块** — `read_magic_properties_limited` 对 Mod 自定义套装属性的 0x1FF 结束标记找到了错误位置 → 偏移累积
2. **chronicle 块** — 52bit 被跳过/不应跳过
3. **plist_flag 位置** — 先于 read_magic_properties 读取 → 但回退后验证无变化
