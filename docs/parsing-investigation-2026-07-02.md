# D2I 装备解析 — 排查记录与关键发现

## 一、已修复

### 1. 抗性 stat 位宽 + save_add (2026-07-02)
**文件**: `src/stash/magical_props.rs`

| Stat | 修前 | 修后 | 原因 |
|------|:----:|:----:|------|
| damageresist [36] | sb=8 sa=0 | sb=9 sa=200 | Mod 的 itemstatcost.txt 使用 9bit |
| magicresist [37] | sb=8 sa=0 | sb=9 sa=200 | 同上 |
| fireresist [39] | sb=8 sa=50 | sb=9 sa=200 | 同上 |
| lightresist [41] | sb=8 sa=50 | sb=9 sa=200 | 同上 |
| coldresist [43] | sb=8 sa=50 | sb=9 sa=200 | 同上 |
| poisonresist [45] | sb=8 sa=50 | sb=9 sa=200 | 同上 |

### 2. Chest-stackable trailer (2026-07-02)
**文件**: `src/stash/item.rs`

**问题**: `read_single_item` 对所有物品末尾都读了 chest-stackable 标记(1+8bit)，导致装备页(非堆叠页)的物品数量显示为 ×255/×325 等错误值。

**修复**: 仅在 `on_stackable_page || simple_item` 时读取尾部，非堆叠页的装备 amount=1。

## 二、待修复 — Body 偏移 3bit (2026-07-02)

### 问题
规范解析器从偏移 **198** 开始读取 magic properties，但正确的起点是 **201**，差 3bit。

### 根因
`multi_pic=1` 触发读取 3bit `picture_id`，但该字段之后的 body 消费位置有误，导致 magic properties 起点偏移 3bit。

### 证据
通过 Page[3] 一个稀有护身符验证：

| 偏移 | 读取到的 stat | 对比用户描述 |
|:----:|---------------|-------------|
| 198(规范) | sid=232,73,304,2,120,166,79,260,45,139,220,468 | ← 完全不对 |
| 201(正确) | sid=0,1,60,83,105,368,369,371,373 | ← 全部命中 |

### 偏移 201 验证的完整 stat 值 (Rare Amulet)

```
sid=  0  力量   sb=7 sa=32    val=38  adj=+6    → +6力量  ✓
sid=  1  能量   sb=7 sa=32    val=35  adj=+3    → +3能量  ✓
sid= 60  生命偷取 sb=7 sa=0   val=6   adj=+6    → 6%偷命  ✓
sid= 83  技能   sb=3 sp=3     val=1   p=7        → +1术士技能(Sorceress class=7) ✓
sid=105  施法速度 sb=7 sa=20  val=38  adj=+18   → 18%施法  ✓
sid=277  活力(按时间) sb=22  val=150318         ← 无关属性
sid=368  coi_inf_t1_count    sb=10   val=5       ★ 注灵
sid=369  coi_inf_t1_gate     sb=10   val=8       ★ 注灵
sid=371  coi_inf_t2_gate     sb=10   val=13      ★ 注灵
sid=373  coi_inf_t3_gate     sb=10   val=3       ★ 注灵
```

## 三、关键数据源

| 文件 | 对应 Rust 表 | 说明 |
|------|-------------|------|
| `itemstatcost.txt` | `magical_props.rs` | stat ID → 位宽(sb/sp/sa) |
| `magicprefix.txt` | (无表,行号=ID) | 728条前缀, 项链可用26种mod |
| `magicsuffix.txt` | (无表,行号=ID) | 786条后缀, 项链可用43种mod |
| `armor/weapon/misc.txt` | `game_items.rs` | 物品代码 → 名称/类别 |
| `properties.txt` | `magical_props::num_sub_props` | 多stat复合属性 |

### 重要结论
- 物品头部(32bit flags) 和游戏 webpack 代码完全一致 ✅
- `itemstatcost.txt` 的 **47 个 Saved=1 stat** 位宽全部正确 ✅
- Mod 自定义 stat 为 **ID 368-419** (coi_inf_*, coi_jzb_*, coi_root_*)
- 游戏 webpack 的 `magical_properties` 数组只有 **359 条**(0-358)，不包含 mod 自定义 stat
- 稀有词缀 ID 编码: `< 728=前缀ID`，`>=728=后缀ID-728`
- 玩家存档文件(d2i)的 magic properties 偏移比规范解析器多 3bit（multi_pic=1 的 picture_id 未计入 body 前缀消耗）

## 四、后续待做

1. **修正 body 偏移 3bit** — 找到 `multi_pic=1` 的 picture_id 在 body 流程中的累积点
2. **更新 magical_props.rs** — 补充 sid=468(或其他被引用但表里没有的 stat)的定义
3. **Page[0] 重新验证** — 偏移修正后，检查所有装备解析结果
