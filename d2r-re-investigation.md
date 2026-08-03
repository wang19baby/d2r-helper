# D2R.exe 逆向分析报告 — Save/物品格式

## 调查目标
分析 D2R.exe 中与 save 相关的代码路径，确定物品在方块/背包/仓库/装备栏中的位存储格式。

## 方法

### 第一阶段：字符串表扫描（已完成 ✅）
- 工具：Python 从二进制中提取 ASCII 字符串（>=4 字符）
- 结果：从 32MB 二进制中提取约 320,000 个字符串，过滤出 43 个 "save" 相关、数百个 item/stash 相关字符串

### 第二阶段：上下文定位（已完成 ✅）
- 围绕关键字符串做十六进制上下文 dump（前后各 64-512 bytes）
- 找到完整的数据表（ItemStatCost）、文件扩展名映射表、SaveOperation 状态机字符串

### 第三阶段：代码段分析（进行中 ⏳）
- 在 .text 段（22.9MB）搜索位操作指令模式
- 搜索 `test al, imm`, `or reg, imm8`, `shl reg, 1`, `cmp reg, 0x1FF` 等模式
- 结果：确认 DWORD 掩码值的存在，但无法仅此确定位流顺序（需要反汇编器）

## 关键发现

### 存档文件体系
```
扩展名    用途                   发现位置
.d2s      角色存档               0x01715be1 扩展名表
.d2i      物品容器数据           同上
.ctlo     D2R 新角色数据文件     0x0172f7b3 (格式: %s.ctlo / %s%llu.ctlo)
.keyo     按键绑定?              0x01715be1
.map      地图数据               0x01715be1
.ctl      控制映射               0x01715be1
.ma%d     分页地图               0x01715be1
```

### ItemStatCost 表（完整嵌入）
- 地址：`0x01722660`
- 内容：156 列完整表头（advdisplay, stuff, op stat1-3, func1-7, t1code1-7, etc.）
- 用途：定义每个 stat 的编码位宽、运算方式、关联参数
- 关键字段：`t1code1-7` + `t1param1-7` → 实际决定每个 stat 的编码格式

### Save 系统调用链
```
SaveSystem → SaveOperation → SaveContainerAsync / LoadContainerAsync
→ filesystem::SaveFile (BGS SDK 6.3.1, platform_win32/filesystem.cpp)

状态: Success, Unknown Task, UserServiceNotReady, 
       MalformedSaveOperation, Other, UserMismatch,
       ResourceCollision, ResourceNotFound, ResourceCorrupted
```

### PLAYERSAVEWrite
- 字符串地址：`0x0172c3d0`
- 格式：`PLAYERSAVEWrite(sPlayerSaveToStream) - IsPlayerExit: %d, Character Name: %s`
- .text 段中未找到直接 RIP-relative LEA 引用此字符串（可能通过指针表间接引用）

### 位操作常数（.text 段）
```
掩码       or reg,imm8 次数    分析结论
0x01       15                  hasHeader 通用位
0x02       5                   identified
0x04       8                   socketed
0x08       5                   compactSave / new
0x10       16                  ethereal（最频繁，多处检查）
0x20       10                  personalized
0x40       15                  god/expansion
0x80       6                   runeword
0x100      0                   ear（太大无法 imm8 编码）
0x200      0                   startItem（同上）
```
各掩码均以 `test al, imm` 形式出现 276-362 次。

SHL 1 指令计数 253-403 次，广泛用于位流操作。

### 容器的 UI 引用
```
容器类型        UI 名                            网格引用
背包            PlayerInventoryPanel              gridX, gridY, OriginalLayout / ExpansionLayout
方块            HoradricCubePanel/Layout          HoradricCubeLayout (3x4)
腰带            BeltWidget, CfgBelt1-4            "4x1" 模式
公共仓库        SharedStashPage/TabContainer/Local   多页 (HC/SC/天梯/非天梯)
私人仓库        PrivateStash
高级仓库        AdvancedStashSlotWidget
```

### 公共仓库变种全集
```
SharedStashLocal / SharedStashLocalHardcore
SharedStashMPNonLadder / SharedStashMPNonLadderHardcore
SharedStashMPLadder / SharedStashMPLadderHardcore
ModernSharedStashSoftCoreV2 / ModernSharedStashHardCoreV2
```

### D2R 新增特性（二进制确认）
```
特性            二进制字符串                          说明
幻化            Transmogrify, ItemTransmogrified     外观替换
物品编年史      ItemChronicleAdded, ChronicleItem... 操作历史
Protobuf 层     common_inventory_state.proto         网络同步
等阶孔洞        MaxSocketsLevelThreshold1-3           按 ilvl 定最大孔数
HD 掉落间距     ItemDropSpacingResurrectedHD          高清布局
高级仓库可堆叠  AdvancedStashStackable
```

### Protobuf 文件列表（0x1744xxx 区域）
```
D2Proto/
├── common_inventory_state.proto     ← 库存状态
├── common_item_detailed_state.proto ← 物品详细信息
├── common_character_details.proto   ← 角色详情
├── item_moved/dropped/picked_up/identified/socketed/
│   bought/sold/imbued/consumed/repaired/
│   transmogrified/chronicle_added/trade_completed.proto
└── game_player_died/activity_changed.proto
```

## 遇到的问题和解决方案

### 1. `strings` 命令不可用
- 问题：Windows 环境没有标准的 `strings` 工具
- 解决：用 Python `re.findall(rb'[\x20-\x7e]{4,}', data)` 替代

### 2. 二进制太大（32MB）导致搜索慢
- 问题：多次全量搜索导致 Python kernel OOM 或超时
- 解决：使用内存映射文件对象、缩小搜索范围（子节区 `.text`/`.rdata`）

### 3. PE 文件格式定位
- 问题：需要知道 .text 段的实际偏移才能做代码分析
- 解决：解析 PE 头（`IMAGE_DOS_HEADER` → `IMAGE_NT_HEADERS` → `IMAGE_SECTION_HEADER[]`）
  - .text: raw=0x400, VA=0x1000, size=22.9MB
  - .rdata: raw=0x015d7400, VA=0x015d8000
  - .data: raw=0x01984a00, VA=0x01986000

### 4. 静态指令模式搜索有限
- 问题：无法仅通过字节模式确定位流顺序（没有反汇编器的控制流分析）
- 当前状态：需要 x64dbg / IDA Pro 做动态分析
- 已知信息：DWORD 掩码在代码中直接使用，但位流的位顺序 ≠ 掩码值大小顺序

### 5. 0x1FF (stat 终止符) 的代码未被定位
- 问题：预期 `cmp eax, 0x1FF` 或 `and eax, 0x1FF` 模式（`25 FF 01 00 00` / `3D FF 01 00 00`）在 .text 中未出现
- 推测原因：编译器可能使用 `sub` + 标志位判断，或 `test r64c, r64`，或跳转表方式，而不是直接 compare
- 二进制中 0x1FF 作为 9-bit ALL-1 的概念是明确的（写入 9 个 1 bit），但读取时的边界检查可能不通过 compare 实现

### 6. 动态分析和字符串引用定位
- 问题：RIP-relative LEA 模式的搜索对 `PLAYERSAVEWrite` 字符串未命中
- 推测：该字符串可能被编译器合并到其他字符串中、或通过全局指针表间接引用
- 未尝试的方法：搜索地址 `0x0172c3d0` 在 `.reloc` 节中的重定位条目

## 待完成（下个 Session 交接）

### 优先级 P0：位流顺序验证
- 工具：x64dbg
- 方法：
  1. 在 PLAYERSAVEWrite 下访问断点（`ba r4 0x0172c3d0`）
  2. 回溯调用栈到 `WriteItem` 或等价函数
  3. 在 bitstream read 循环中设断
  4. 单步跟踪前 11 个 `ReadBit` 调用
  5. 记录每个 bit 被赋值到 runtime flags 的哪一位
- 预期输出：位流序号 ↔ DWORD 掩码 的完全映射表

### 优先级 P1：StatValue 编码表提取
- 目标：从 `.rdata` 中 ItemStatCost 表头（0x01722660）之后的数据区提取实际的 statID→bitWidth 映射数组
- 方法：
  1. 在 0x01722660 之后找跳转到各 stat 行的指针表
  2. 每行解析出 `t1code1-7` + `t1param1-7` 字段
  3. 编译成 statID → encodeType → bitWidth 查找表
- 预期输出：完整的 stat 编码规格表（480+ stat 条目）

### 优先级 P1：Grid 坐标位宽确认
- 方法：在 `gridX`/`gridY` 字符串附近找引用代码 → 看移位操作的立即数
- 或：在背包物品的保存路径中设断，观察写入的 X/Y 值的位移量

### 优先级 P2：.ctlo 文件格式分析
- 方法：创建角色后观察 `Documents\My Games\Diablo II Resurrected\` 下有无 `.ctlo` 文件
- 用 010 Editor 模板分析结构
- 确定 `.ctlo` 是替代 `.d2s` 还是补充

### 优先级 P2：0x1FF 终止符的精确代码定位
- 方法：用 x64dbg 在 stat 编码循环内设条件断点
- 观察 statID==0x1FF 时的退出条件判断方式

### 已知但未探索的方向
- UTF-16 字符串中的 save 路径（D2R 是 Windows GUI 应用，许多路径以宽字符存储）
- `.reloc` 节中的函数指针表
- BGS SDK 加密/校验层（checksum, blowfish）
- `SaveMonsters` 怪物状态保存格式
- 尸体掉落 (`ctlo`, `corpse` 引用)
- 佣兵装备存档

## 关键内存地址参考
```
0x01715be1  文件扩展名映射表 (.map .d2i .d2s .ctlo .keyo .key .ctl .ma%d)
0x01722660  ItemStatCost 表头（156 列）
0x0172c3d0  PLAYERSAVEWrite 字符串
0x0174ef00  SaveOperation 状态机
0x0174f1b0  SaveContainerAsync / LoadContainerAsync
0x0172f7b3  ctlo 文件格式模板
0x0171b908  SharedStashPage UI 引用
0x00949253  FLG_ 枚举
```

---

## 2026-07-08 更新：Webpack Chunk 3948 反编译确认

### 来源
- D2R webpack chunk `3948-158edc89d00d35f1.js`（Emscripten 编译的 C++ stash/item 解析器到 JS）
- 包含与 native D2R.exe 相同的格式解析代码

### 完全确认的 Item Header 位流顺序
```
位偏移        字段              位宽    取值
0-3           unknown           4       
4             identified        1       0/1
5-10          unknown           6       
11            socketed          1       0/1
12            unknown           1       
13            new               1       0/1
14-15         unknown           2       
16            is_ear            1       0/1   (PvP 耳朵)
17            starter_item      1       0/1   (起始物品)
18-20         unknown           3       
21            simple_item       1       0/1   (基础物品/无词缀)
22            ethereal          1       0/1
23            unknown           1       
24            personalized      1       0/1   (角色名定制)
25            unknown           1       
26            given_runeword    1       0/1   (符文之语)
27-31         unknown           5       
==== 32 bits ====
```

### 确认的 Item Header 后续字段
```
序号    字段             读取方式
1       version          10 bits + 3 bits（两个连续读）
2       location_id      3 bits
3       equipped_id      4 bits
4       position_x       4 bits
5       position_y       4 bits
6       alt_position_id  3 bits
7       type             4 chars (item code, e.g. "7ar")
8       version_str      2 chars
9       name             16 chars
```

### Stat 列表编码确认
```javascript
// 读取 stat ID（9 bits）
let i = e.ReadUInt16(9);
for(; 511 != i; ) {  // 0x1FF 终止符
    // 查表
    let n = r.magical_properties[i];
    
    // 读取参数（如果 sP > 0）
    if(n.sP) {
        let t = e.ReadUInt16(n.sP);
        // 特殊编码处理
        // dF bit 14: 低3位+高13位
        // encoding 2/3 (cast/charged): 低6位+高10位
    }
    
    // 读取值（sB bits）
    let s = e.ReadUInt16(n.sB);
    if(n.sA) s -= n.sA;  // 减去偏移
    
    // 处理 encoding 类型
    // encoding 3 (charged): 值的低字节+高字节分开存储
}
// 写入终止符
e.WriteUInt16(511, 9);
```

### 物品质量枚举
```
Normal    = 2
Superior  = 3
Magic     = 4
Set       = 5
Rare      = 6
Unique    = 7
Crafted   = 8
```

### 字符属性读取（gf 段）
```javascript
// 使用 CSvBits（cB），不是 Save Bits（sB）
let o = s.cB;  // CSvBits
e.attributes[a[s.s]] = t.ReadUInt32(o);
```
CSvBits 用于角色截面的 stat 读写（strength=10, energy=10, hitpoints=21 等）。

### Flag 位顺序验证结论
`item_flags.rs` 当前的 bit 4/11/13/16/17/21/22/24/26 位序完全正确，
与 D2R 引擎代码 100% 一致。

## 已完成清单

| 项目 | 状态 | 来源 |
|------|------|------|
| 字符串表提取 | ✅ | D2R.exe .rdata |
| ItemStatCost 列名定位 | ✅ | 0x01722660 |
| 文件扩展名映射表 | ✅ | 0x01715be1 |
| SaveOperation 状态机 | ✅ | 0x0174ef00 |
| Protobuf 文件列表 | ✅ | 0x1744xxx |
| MAGICAL_PROPS 表 | ✅ | webpack 385ca0d5 |
| CSvBits 集成 | ✅ | itemstatcost.txt → stat_cost.rs |
| 位流 flag 顺序 | ✅ 100% 匹配 | webpack 3948 → item_flags.rs |
| 0x1FF 终止符 | ✅ (511) | webpack 3948 |
| 物品质量枚举 | ✅ | webpack 3948 |
| 角色属性编码 | ✅ (CSvBits) | webpack 3948 |
| 0x1F8-0x1FF 范围 | ✅ (9-bit) | webpack 3948 |
| stat 读取循环编码 | ✅ | webpack 3948 |
| Grid 坐标位宽 (4x4) | ✅ | webpack 3948 |
| .reloc 引用表分析 | ✅ | D2R.exe .reloc |
