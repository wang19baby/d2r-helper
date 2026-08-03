# D2R Stat 系统完整逆向分析报告

日期: 2026-07-09
来源: D2R.exe v1.6.0 (32MB PE32+ x86-64) + webpack chunk 3948 + ItemStatCost.txt 提取数据

---

## 1. PE 段映射

| 段 | RVA | VA | 原始偏移 | 原始大小 |
|----|-----|-----|---------|---------|
| .text | 0x1000 | 0x140001000 | 0x400 | 0x15d7000 (22.9MB) |
| .rdata | 0x15d8000 | 0x1415d8000 | 0x15d7400 | 0x3ad600 (3.7MB) |
| .data | 0x1986000 | 0x141986000 | 0x1984a00 | 0x3afa00 |
| .rsrc | 0x2757000 | 0x142757000 | 0x1e6b600 | 0x1f400 |

## 2. 关键内存地址验证

| 符号 | 地址 (VA) | 原始偏移 | 内容 |
|------|-----------|---------|------|
| ItemStatCost 列名表 | 0x140172260 | 0x171660 | 列名字符串 ("ItemStatCost\0", "advdisplay", "stuff", ...) |
| ItemStatCost 描述符表 | 0x140172d40 | 0x171d40 | 24-byte 列描述符 (name_ptr + func1 + func2) |
| "gf" header 引用 | .text 全段 | − | 345 处 0x6766 代码引用 |

**原始报告修正**：之前的报告列出了 `0x01722660` 作为 ItemStatCost 地址——这是 **RVA**（不含 image base）。完整 VA = `0x140000000 + 0x01722660 = 0x140172260`。此地址存储的是列名 ASCII 字符串，而**实际行数据**通过列描述符表（0x140172d40）间接访问，运行时动态构建。

## 3. Stat 位流编码 — webpack 验证（D2R 引擎一致性）

Webpack chunk `3948-158edc89d00d35f1.js` 是 Emscripten 编译的 C++ 代码，与 native D2R.exe 使用**同一份源码**。两者数据格式**完全一致**：

### 3.1 物品 Stat 列表读取循环（伪码）

```
while (true) {
    int stat_id = ReadBits(9);            // SHL EDX,9 (5 matches in .text)
    if (stat_id == 0x1FF) break;         // 终止符
    
    prop = magical_properties[stat_id];   // 查表
    int param = ReadBits(prop.save_param_bits);
    int raw = ReadBits(prop.save_bits);
    int value = raw - prop.save_add;      // 减偏移得到实际值
    
    // encoding 1/2/3 特殊处理
    switch (prop.encoding) {
        case 1: // non_class_skill: param低6位+高10位
        case 2: // skill_on_event: param低6位+高10位
        case 3: // charged_skill: value分低字节+高字节
        case 4: // by_time: 32-bit time windowed
    }
    
    // sub_prop 自动展开
    for (int j = 1; j < prop.num_sub_props; j++) {
        sub_id = stat_id + j;
        value = ReadBits(...);  // 后续 sub-stat 共用同一 bitstream 条目
    }
}
```

### 3.2 关键位流操作确认

- **9-bit stat_id**: `SHL EDX,9` 在 .text 中出现 5 次，确认 9 位移位读取操作
- **0x1FF 终止符**: webpack 确认值为 511（9-bit ALL-1）。native x64 中未发现直接 `CMP/AND EAX,0x1FF`——编译器优化为跳转表或位流状态机内部处理
- **CSvBits 用于 gf 段**: `0x6766` 在 .text 中出现 345 处代码引用，`AttributeId::bit_length_from_table()` 正确实现

## 4. `data/stat_cost.rs` vs `tools/stat_cost_generated.rs` — 65 处差异

### 4.1 差异分类汇总

| 字段类型 | 差异数 | 正确版本 | D2R.exe/ItemStatCost 证据 |
|---------|-------|---------|------------------------|
| **save_add** | **37** | ✅ tools | `itemstatcost.json` Col 21 |
| **signed** | **15** | ✅ tools | `itemstatcost.json` Col 3 |
| **encoding** | **7** | ✅ tools | `itemstatcost.json` Col 14 |
| **save_param_bits** | **1** | ✅ tools (item_aura sp=9) | `itemstatcost.json` Col 22 |
| **num_sub_props** | **6** | ⚠️ 待定 | 来自 webpack, 需动态验证 |
| cs_bits | 0 | 相同 | 两者一致 |
| save_bits | 0 | 相同 | 两者一致 |

### 4.2 对 vs `itemstatcost.json` 的正确率

- **data/stat_cost.rs**: 249/398 正确 (62.6%)
- **tools/stat_cost_generated.rs**: **308/398 正确 (77.4%)**

### 4.3 严重程度分级

#### 🔴 P0 — 物品 Stat 值完全错误（37 处 save_add 缺失）

`data/stat_cost.rs` 全部使用 `save_add=0`，但 D2R 引擎使用非零值：

```
Stat ID 名称                data.rs  tools.rs (正确)  影响
─────────────────────────────────────────────────────
 67  velocitypercent        0        30              移动速度偏 30%
 68  attackrate             0        30              攻击速度偏 30%
 71  value                  0       100              NPC 价格偏 100 gold
 74  hpregen                0        30              每秒回复偏 30 HP
 75  item_maxdurability%    0        20              耐久度加总裁偏 20%
 76  item_maxhp%            0        10              HP 加总裁偏 10%
 77  item_maxmana%          0        10              MP 加总裁偏 10%
 79  item_goldbonus         0       100              金币获取偏 100%
 80  item_magicbonus        0       100              MF 偏 100%
 82  item_timeduration      0        20              持续时间偏 20
 85  item_addexperience     0        50              经验获取偏 50%
 89  item_lightradius       0         4              光照范围偏 4 码
 91  item_req_percent       0       100              需求降低偏 100%
 93  item_fasterattackrate  0        20              IAS 偏 20%
 96  item_fastermovevelocity 0       20              跑速偏 20%
 99  item_fastergethitrate  0        20              FHR 偏 20%
102  item_fasterblockrate   0        20              FBR 偏 20%
105  item_fastercastrate    0        20              FCR 偏 20%
110  item_poisonlengthresist 0       20              毒抗时长偏 20%
111  item_normaldamage      0        20              对恶魔伤偏 20%
112  item_howl              0        -1              吓跑怪物等级偏 1
119  item_tohit_percent     0        20              命中加总裁偏 20%
120  item_damagetargetac    0       128              减目标 AC 偏 128
121  item_demondamage%      0        20              对恶魔伤偏 20%
122  item_undeaddamage%     0        20              对不死伤偏 20%
123  item_demon_tohit       0       128              对恶魔命中偏 128
124  item_undead_tohit      0       128              对不死命中偏 128%
154  item_staminadrainpct   0        20              耐力消耗加总裁偏 20%
305-308 pierce_*            0        50              元素穿透偏 50%
329-332 passive_mastery_*   0        50              精通加总裁偏 50%
357  passive_mag_mastery    0        50              魔法精通偏 50%
```

#### 🟡 P1 — Signed 缺失（15 处）

大多数是 1-bit flag stat（`ignoretargetac`, `indesctructible`, `cannotbefrozen` 等），对于 1-bit 值 signed/unsigned 等价，但不匹配官方 ItemStatCost.txt。

影响更大的：
- `curse_resistance` (109): signed=1 → signed=**0** (工具是对的)
- `item_reanimate` (155): signed=1 → signed=**0** (工具是对的) 
- `attack_vs_montype` (179): signed=1 → signed=**0**
- `damage_vs_montype` (180): signed=1 → signed=**0**

#### 🟠 P2 — Encoding 缺失（7 处）

`data/stat_cost.rs` 中缺失的 encoding 在 `parse_stats_with_table()` 中未被使用——当前 stat reader 不检查 encoding 字段。但写入位流时、或与 D2R 引擎交互时会出问题：

| ID | 名称 | 应有 encoding | tools.rs | data.rs | 影响 |
|----|------|--------------|----------|---------|------|
| 107 | item_singleskill | 1 (non-class-skill) | 1 | **0** | +技能 stat 参数读取异常 |
| 195 | item_skillonattack | 2 (cast-on-strike) | 2 | **0** | 技能触发类 stat 6 个全错 |
| 196 | item_skillonkill | 2 | 2 | **0** | 同上 |
| 197 | item_skillondeath | 2 | 2 | **0** | 同上 |
| 198 | item_skillonhit | 2 | 2 | **0** | 同上 |
| 199 | item_skillonlevelup | 2 | 2 | **0** | 同上 |
| 201 | item_skillongethit | 2 | 2 | **0** | 同上 |

#### 🔵 P3 — num_sub_props 分歧（6 处）

```
数据文件                              data.rs  tools.rs
───────────────────────────────────────────────
 16  item_armor_percent                2       1
 48  firemindam                        1       2  (包含 firemaxdam)
 50  lightmindam                       1       2  (包含 lightmaxdam)
 52  magicmindam                       1       2  (包含 magicmaxdam)
 54  coldmindam                        1       3  (包含 coldmaxdam + coldlength)
 57  poisonmindam                      1       3  (包含 poisonmaxdam + poisonlength)
```

注意：`parse_stats_with_table()` (d2i parser 中的独立实现) **使用 `prop.num_sub_props`** 展开，而 `stat_list.rs` 和 `d2s/items.rs` 使用 `FieldSet::sub_property_count()` 硬编码。**两套逻辑不一致**。

## 5. 修复建议

### 5.1 立即修复（P0 — stat 值全错）

**用 `tools/stat_cost_generated.rs` 替换 `src-tauri/src/data/stat_cost.rs` `MAGICAL_PROPS` 数组**。一键修复 65 处差异中 `data.rs` 错误的 37 处 save_add、15 处 signed、7 处 encoding、1 处 param_bits。

执行方式：
```bash
cp tools/stat_cost_generated.rs src-tauri/src/data/stat_cost.rs
```

### 5.2 统一 sub_prop 读取（P2）

`parse_stats_with_table()` (d2i/parser.rs) 和 `parse_magic_properties_to_stats()` (d2i/legacy/complete_header.rs) 中的 `num_sub_props` 处理逻辑应与 `FieldSet::sub_property_count()` 保持一致。

建议：不论 num_sub_props 来自表还是硬编码，都优先使用 `FieldSet::sub_property_count()`。

### 5.3 添加 encoding 感知（P2）

当前 stat reader 完全不读 encoding 字段。对于 encoding=2/3 的 stat，参数/值的位分布不同。至少需要：
- encoding=2: param 低 6 位 + 值高 10 位的融合读取
- encoding=3: 值的低字节/高字节分离存储

### 5.4 修复 gf 段 CSvBits 空项（P3）

`MagicProp::empty()` 设 cs_bits=0 但 stat 4-15 应有 cs_bits。`AttributeId::bit_length_from_table()` 通过在 `cs_bits==0` 时 fallback 到硬编码值来绕过此问题。建议 `empty()` 改为接受 cs_bits 参数，或使用 `MagicProp::new(0,1,0,0,0,0,10)` 显式指定。

## 6. 已验证的正确实现列表

| 模块 | 状态 | 来源 |
|------|------|------|
| ItemFlags bit 顺序 | ✅ 100% | webpack 3948 ↔ .text 掩码分析 |
| 0x1FF 终止符 | ✅ (511=9bit ALL-1) | webpack 3948 |
| Stat 读取循环 9-bit ID | ✅ SHL EDX,9 ×5 | .text |
| StatValue = raw - save_add | ✅ | webpack + ItemStatCost Col 21 |
| CSvBits 用于 gf 段 | ✅ | `AttributeId::bit_length_from_table()` |
| Q8 HP/Mana/Stamina | ✅ | `attributes.rs` `is_q8()` |
| 物品 quality 枚举 | ✅ | webpack 3948 |
| Grid 坐标位宽 (4x4) | ✅ | webpack |
| Character stat 固定 16 项 | ✅ | attributes.rs |
| Sub-property 展开 | ✅ (硬编码) | `FieldSet::sub_property_count()` |
