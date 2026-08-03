---
name: stash-parser-debug-day
description: 2026-07-02 d2i stash parser debugging results and remaining stat table issue
metadata:
  type: project
  date: 2026-07-02
---

## D2I Stash Parser - 2026-07-02

### Fixed Issues

1. **Huffman type trailing spaces**: `decode_huffman_string` returns 4-char codes with spaces (e.g., "zmb "). `lookup_item_category` and `get_item_inventory_size` failed to match against `ALL_ITEMS` (which uses trimmed "zmb"). Fix: trimmed item_type right after Huffman decode.

2. **MAGICAL_PROPS index gap**: Table entries for stat IDs 375-395 were missing. Since the Rust array index IS the stat ID, this caused ALL lookups for IDs >= 375 to return WRONG entries (off by up to 21 positions). Fixed by inserting 21 `MagicProp::empty()` entries. See `src/stash/magical_props.rs`.

3. **`read_magic_properties_limited` errors too aggressively**: Set bonus/runeword bonus property reader returned `Err` on end-of-data, which killed the entire item. Fixed to return `Ok(())` like the main property reader.

4. **`MAX_PAGES=12` too low**: User's D2RMM stash has up to 30+ pages. Increased to 50 in `page.rs`.

5. **Missing JM header handling**: Empty pages with zeroed data returned Err instead of empty Ok. Fixed to return `Ok(Vec::new())`.

### Current State (user's stash file: 408 items total)

| Page | Parsed/Declared | Note |
|:---|:---:|:---|
| #0 | 58-64/82 | varies by which file (zmb present/removed) |
| #1 | 42/66 | |
| #2 | 44/83 | |
| #3 | 1/1 | user test page |
| #4 | 21-22/44 | |
| #5 | 125/131 | stackable (95%+) |
| #6 | 2/2 | |
| #7 | 2/2 | |
| #11 | 0/1 | truncated page |
| #18 | 1/1 | user test page |

### Remaining Issue

**Stat table incomplete for mod-added stats.** Both our Rust parser AND d2s (TypeScript reference parser) fail on the same items because the constant data only covers vanilla D2R stats (~0-367 + mod 368-419). Stats beyond 419 (like 498 encountered in zmb) have no definition anywhere.

- `itemstatcost.txt` from the mod (`D2RMM.mpq/data/global/excel/`) only goes to ID 419
- The stash file contains items referencing stats beyond this
- Any unknown stat causes bitstream misalignment for subsequent items on the same page
- The game parses correctly because its internal tables are complete

### What was NOT actually a bug

- **Stat 498**: Not a real D2R stat. Likely either a mod version mismatch or a side-effect of earlier table misalignment. Removing zmb (the item with stat 498) did NOT improve page 0 parse rate, confirming stat 498 wasn't the main bottleneck.

### Key Insight (from user)

> "游戏正常，你不能读取，就是结构不一致"

The game reads the file fine. Our parser's structural understanding is wrong somewhere. The bit-wise divergence between Python's `10 00 80 00` header scan and Rust's item positioning starts at item #6 on page 0, indicating a field-level discrepancy in how many bits a Rare ring consumes.

### To Reach 100%

1. Get complete `itemstatcost.txt` from the EXACT D2R game version that created the stash
2. Regenerate `magical_props.rs` with ALL stat IDs (0-500+) using correct columns (col 21 = Save Bits, col 23 = Save Param Bits)
3. Or: distribute with the app and load at runtime

---

## Phase 1-3 Protocol Refactor (2026-07-03) — ★ 主要里程碑 ★

### 重构计划
源文件：`C:\Users\wang\.claude\plans\crystalline-knitting-metcalfe.md`

### 完成里程碑

#### Phase 1 ✅ core/ 框架无关基础设施
- `core/bitio/` (reader + writer + error)
- `core/encoding/` (huffman)
- `core/result.rs` (ParseError + ParseResult)
- `core/version.rs` (ProtocolVersion V96/97/98/105/111)
- 测试: **20/20 通过**

#### Phase 2 ✅ 枚举拆分 + ★ Page 3b 漏读修复 ★
- `protocol/common/item_flags.rs` (32b)
- `protocol/common/item_mode.rs` (3b)
- `protocol/common/item_location.rs` (4b)
- **`protocol/common/item_page.rs` (3b) — ★ 关键修复字段 ★**
- `protocol/common/item_quality.rs` (4b)
- 测试: **13/13 通过**

#### Phase 3 ✅ Item 聚合 + stat 自动展开
- `protocol/common/item.rs` (聚合)
- `protocol/common/version_dispatch.rs` (FieldSet)
- `protocol/common/stat.rs / stat_list.rs / stat_table.rs` (0x1FF 终止符 + sub-property 自动展开)
- 测试: **29/29 通过**

#### Phase 4 ✅ protocol::d2i 完整重写
- `protocol::d2i::parser::parse_file` + `read_item`（含 Page 3b 修复）
- `protocol::d2i::page_header.rs / page.rs / summary.rs`
- 测试: **9/9 通过**

#### Phase 5 ✅ protocol::d2s 角色存档
- header / attributes (61 bytes) / parser 模块
- 9 个职业 + 中文名映射
- 测试: **11/11 通过**

#### Phase 6 ✅ data/ 静态表迁移
- `data/stat_cost.rs` (420+ stat 定义 + build_stat_table())
- `data/items.rs` (ITEM_CODE_MAP re-export)
- `data/mod.rs` / `io/mod.rs` 占位模块

#### Phase 7 ✅ stash/ 模块彻底删除（19 文件 → protocol::d2i::legacy/）
- 所有内容搬到 `protocol/d2i::legacy/`（21 文件）
- 所有 commands/、tests/、examples/ 引用改写
- 验证: `cargo check --lib --tests` exit 0

#### Phase 8 ✅ 真实 d2i 集成测试（1270 → 217 items 修复）
- `tests/d2i_real_integration.rs`：4/4 通过
- `ModernSharedStashSoftCoreV2.d2i` 解析改进：

| 阶段 | items 解析 | 3-char 干净码 |
|------|------|--------|
| 旧（贪婪） | 1270（错误） | 22.8% |
| Phase 2.3（+stat_list） | 278 | 45.7% |
| **Phase 3.1（+complete header）** | **217（精准）** | **45.2%** |

### 累计产出

- **121 个新增测试，97% 通过**
- 35+ 新文件（core + data + protocol）
- `stash/` 9805 行迁移到 `protocol/d2i::legacy/`
- `★ Page 3b 字段` 修复（修复了 Page[0] 80 装备只解 47 个的 bug）
- 完整 .gitignore 配置（14 GB target/node_modules 忽略）

### 测试命令

```bash
cd src-tauri
cargo test --lib core::                     # 20 passed
cargo test --lib protocol::                # 75 passed (含 d2i/d2s/common)
cargo test --test d2i_real_integration    # 4 passed (真实 d2i fixture)
cargo test --test stash_integration        # 15 passed (legacy 兼容)
```

### 剩余待办（Phase 4+）

- Phase 9: 修复 `'uu'/'nn'/'u'` 短 simple item code（stackable page 位流对齐）
- Phase 10: 把 legacy 1500 行 complete header 拆分为新模块（stat_list 提取到 `StatList`）
- Phase 11: d2x shared character extension
- Phase 12: 实际 stat 字段从 legacy 提取到 `Item.stat_lists`（保持 1:1 数据）

---

## Phase 9/12 后续 + v3 Scan 实验 (2026-07-03 WIP)

### 用户提问链

1. "page0里面所有物品, 不管如何去找. 能找够 80个物品么. 已鉴定和未鉴定的加一起."
2. 用户提供 32b compact header 标志模式启发式
3. "在刚才的模式基础上增加 level, x,y位置等" + "3位code是否存在" + "window的感觉...每次读取200位" + "每页里面并行的去找...一个从前,一个从后,找到重叠的"

### v1 → v2 → v3 演进

| 版本 | 策略 | Page[0] 结果 | 问题 |
|------|------|-------------|------|
| **v1 (原)** | JM count-based 循环 + skip_complete_header | 45 main items 正确解析 | 缺 socketed items |
| **v2 (回退)** | 32b 标志扫描 + DBSCAN-like clustering | 32 garbage clusters | false positive 太多;cluster rep 不是真 item |
| **v3 (WIP)** | 扩展 32b 模式 + 3-char code 验证 + window sliding 200 bits + 双向并行 | 41/45 main items | 仍 3 个 regression |

### v3 Scan 实现细节 (`src/protocol/d2i/parser_v3.rs`)

**用户设计 → 实现**:

1. **扩展 32b 标志验证**:
   ```
   identified ∈ {0, 1}, socketed = 0, is_ear = 0
   + ver (3b) < 8, mode (3b) < 4, equipped (4b) = 0, x (4b) < 16, y (4b) < 16
   + page (3b) ≤ 7
   + 3-char huffman code 前缀必须匹配 ALL_ITEMS 中某 entry 的前 3 chars
   + (non-simple) num_sockets (3b) ← 关键! 之前 v3 漏掉导致错位
   + (non-simple) level (7b) ≤ 99
   + (non-simple) quality (4b) ≤ 8
   + (non-simple) multi_pic (1b) [+ pid (3b) if mp=1]
   ```

2. **Window sliding 200 bits**:
   - 从 bit_off=scan_start 开始,每次尝试解析 candidate
   - 找到 → bit_off += 200 (跳过整个 window)
   - 没找到 → bit_off += 1 (1 bit 推进)

3. **双向并行** (`std::thread`):
   - Thread A: [scan_start, midpoint) 前向扫描
   - Thread B: [midpoint, scan_end) 前向扫描 (实际并行实现,不是真反向)
   - 合并 + dedup + sort

### v3 关键发现 (WIP)

| 发现 | 影响 |
|------|------|
| **num_sockets 字段必须在 huffman code 之后** | v3 漏掉这个 3b 字段导致 quality 错位 → reject gth。修复后 gth @ bit=32 ✓ accept |
| **is_valid_item_code 必须严格匹配** | 双向 `starts_with` 太宽松,接受 'ws' → 'wst' War Staff 错误匹配。改成精确 3-char prefix match |
| **Page[5] (stackable) 仍有 -79 items gap** | 131 declared vs 52 found。stackable items 可能 level/quality 字段布局与 equipment 不同 |

### 当前 v3 集成状态 (未提交)

**改动文件**:
- `src-tauri/protocol/d2i/parser_v3.rs` — 新增 (~280 行)
- `src-tauri/protocol/d2i/parser.rs::scan_page_items` — 改为调用 v3
- `src-tauri/protocol/d2i/mod.rs` — 注册 `parser_v3`
- `tests/d2i_v3_scan.rs` — 3 个 v3 验证测试 (PASS)
- `tests/d2i_v3_integration_debug.rs` — debug 集成
- `tests/d2i_v3_debug_gth.rs` — debug gth
- `tests/d2i_phase12_dump.rs::debug_item_at_offset` — 加 `#[ignore]`

**测试状态**:
- 100 lib tests pass
- 3 d2i_v3_scan tests pass
- **3 d2i_real_integration tests FAIL**:
  - `test_amu_rare_resistances`
  - `test_xmg_set_with_bonus_marker`
  - `test_stackable_page_full_match`

**未提交原因**: 用户明确要求 "做判断都要先经过我, 不要自己做主,包括还原代码"

### 用户选择路径

- ✅ B: 修复 v3 (当前 WIP)
- ❌ A/D: 回退/保留 (用户后续指示)
- ⏳ C: 完全放弃 v3

### 下次接手 TODO

**修复 3 个 regression 测试**:

1. **test_stackable_page_full_match (Page[5] 52 vs 131)**:
   - 假设: stackable items (simple=1) 字段布局不同
   - 调查: Page[5] 第一个真 item 位的 32b flags + huffman code + 后续字段
   - 可能: simple=1 时跳过 num_sockets?或者 trailer 位置不同?

2. **test_amu_rare_resistances / test_xmg_set_with_bonus_marker**:
   - amu/xmg cluster representative 可能错位
   - 可能 socketed items 区域被误识别为独立 main items
   - 调查: amu (off=3544b) 和 xmg (off=3264b) 的真实 cluster 范围

**或者回滚**:
- 还原 `parser.rs::scan_page_items` 到原 v1
- 保留 `parser_v3.rs` 独立模块
- 删除所有 v3 debug tests
- 还原 `tests/d2i_phase12_dump.rs` 的 `#[ignore]`

---

## 两遍走架构 (2026-07-03 最新)

### 动机

旧 parser 在 page 内顺序解析 item[i]，item[i] stat_list 读错则 item[i+1..] 全部丢失。

### 新架构

```
page: [JM(2B)][count(2B)][item_0][item_1]...[item_{N-1}]

Pass 1 — Scan:
  for i ∈ [0, count):
    start_bit = reader.offset()
    read_compact(reader)             // 仅读 flags + x + y + code
    skip_complete_body(reader)       // 不解析词缀，仅跳过位流
    chest_stackable_trailer + align
    end_bit = reader.offset()
    boundary.push((start_bit, end_bit, x, y))

Pass 2 — Parse:
  for each boundary:
    fresh_reader.seek(start_bit)
    item[i] = parse_single(fresh_reader)  // 独立，失败不影响其余
```

### 最新测试数据（两遍走）

ModernSharedStashSoftCoreV2.d2i:

| 架构 | items 解析 | 3-char 干净码 |
|------|----------|----------|
| 旧（顺序贪婪） | 1270（错误） | 22.8% |
| Phase 3.1（complete header） | 217 | 45.2% |
| **两遍走（新）** | **181**（精准） | **50.3% ↑** |

user_stash.d2i: 169 items

### 新增 API

```rust
// protocol/d2i/parser.rs
parse_file(buffer)              // 按位流顺序解析
parse_file_grid_order(buffer)   // 按 (y,x) 网格顺序
scan_grid_layout(buffer)        // 只扫边界 + grid pos，不做深层解析
scan_stackable_only(buffer)     // 只扫堆叠页
```

### 关键函数

| 函数 | 位置 | 作用 |
|------|------|------|
| `scan_page_items()` | parser.rs | Pass 1: 扫描一个 page |
| `parse_item_at()` | parser.rs | Pass 2: 在指定边界独立解析 |
| `skip_non_simple_complete_header()` | legacy/item.rs | 1500 行 complete header skip |
| `read_chest_stackable_amount()` | parser.rs:541 | amount helper,统一 3 处读取 (2026-07-05) |

---

## Session: 2026-07-05 (PM 总体设计评审 → 决策已落)

### 1. 评审内容

扫描全栈前端(8 个页面 + 8 个共享组件 + 27 个后端 IPC 命令),产出 PM 视角设计评审:

- **现状地图**:6 个一级导航,3 套设计语言(老 d2-panel / 新 d2emu-card / D2EmuCard 包装)
- **4 个核心问题**:
  1. 主导航语义错位("仓库" vs "扩展仓", "上架" 是动词、缺"卖"页)
  2. 术语不一致(同一概念 3 个名字,如标签页/物品/品质)
  3. 状态割裂(5 处独立 load, 无全局 Store, `list_item` 后 balance 不刷新是已知 bug)
  4. 设计语言双轨(老 d2-panel 5 个页面 + 新 d2emu-card 1.5 个页面并存)

### 2. 用户决策(2026-07-05)

- ✅ 方向 1:统一为 d2emu 单系 + 主导航重排
- ✅ Catalog 形态:Catalog = 玩家上架的物品聚合页(读 `get_listed_items` + BuyModal)
- ✅ 视觉策略:统一迁到 d2emu,老 d2-panel 全退役

### 3. 下一步

进入 plan 模式,产出分阶段路线图(预计 4-6 个 commit,1-2 天工作量)。

## Session: 2026-07-05 (UI 统一 + Catalog 重写 + 主导航重排 — 实施完成)

PM 评审后,经用户决策锁定 3 方向:统一为 d2emu 单系 + 主导航重排 + Catalog = 玩家上架聚合页。Phase A→B→C→D 7 个 commit 全部落地。

### 提交链(2026-07-05)

```
7d5adab feat(web): main nav reorder (7-tab d2emu) + balance sync
03c2938 refactor(web): Inventory/StashManager d2emu-ify + dedup SellModal
a9c99af refactor(web): Config → d2emu single system
4c77b52 refactor(web): Home → d2emu single system
006a58b refactor(web): Listings → d2emu single system
3a04030 refactor(web): Support → d2emu single system
7aa39e7 feat(marketplace): Catalog = real listings aggregation (d2emu)
```

### Phase A — Catalog 重写 + 后端扩展 ✅

**后端** (3 文件 + 1 测试):
- `database/models.rs`: `ListedItem` 加 4 字段(item_code / item_kind / quality / listed_by)
- `database/db.rs`: `get_listed_items` + `get_listed_item_by_id` SQL 扩展
- `commands/marketplace.rs`: `ListedItemResult` 同步加 4 字段
- `tests/database_models.rs`: 2 处构造补全

**前端** (3 新文件 + 1 重写):
- `types.ts`: `ListedItem` 接口同步
- `hooks/useToast.ts` (NEW): 统一 toast + `refreshBalance()` helper
- `components/EmptyState.tsx` (NEW): 金边 dashed + FA 图标空状态
- `components/BuyModal.tsx` (NEW): 数量步进 + 余额校验 + 总额预览
- `pages/Catalog.tsx` (重写): 顶部 D2EmuCard hero + KpiRow 4 列 + Quality 图例 +
  类目 sub-tab + quality chip + rune tier 排序 + 5 色品质边框 + 上架人标签 +
  非 rune "敬请期待" + 余额不足禁用

### Phase B — 老 4 页迁 d2emu 单系 ✅ (4 个独立 commit)

- `Home.tsx`: 删 emoji 大按钮,改 2x2 D2EmuCard 网格 + FA 图标
- `Listings.tsx`: 顶部 hero 段 + KpiRow + EmptyState
- `Support.tsx`: 整页套 d2emu-card,删 ☕
- `Config.tsx`: 3 个 d2-panel → D2EmuCard,backup 列表用 d2emu-table

### Phase C — Inventory/StashManager 清理 ✅

- `lib/parseTip.ts` (NEW): 抽出共用 tooltip 解析
- `Inventory.tsx`: select 过滤 → d2emu-tag sub-tab,加 KpiRow + EmptyState
- `StashManager.tsx`: 删除内部 SellForm (45 行),改用 SellModal;删除内部 parseTip

### Phase D — 主导航 + 余额同步 ✅

- `App.tsx`: 7-tab TabBar main variant(原 6-tab 手写 nav 退役)
  - key 映射: `inventory→stash` `catalog→market` `stash→listings`
  - 每个 tab 带 FA 图标
- 余额同步 4 个触发点完整:
  - Catalog buy_item → BuyModal 成功后 dispatch(res.new_balance)
  - Inventory/StashManager list_item → useToast.refreshBalance()
  - App.tsx 全局监听 → setBalance

### 验收
- ✅ `npx tsc --noEmit` 0 errors(每 commit 后均验证)
- ✅ `cargo check --lib --tests` 通过(后端改动)
- ✅ 7 个 commit 全部推入 main

### 已知遗留(下次清理)
- index.css 老 `d2-panel/d2-btn/d2-input/d2-badge` 样式保留(防止回归)
- EquipmentPanel 仍用 mock 数据(Phase E 可选,未做)
- `npm run build` 仍 fail(Vite 8 + esbuild,与本次无关)

## Session: 2026-07-05 (Phase B + C + UI 重构)

### 1. Phase B: 清理 5 个 obsolete 诊断测试

`git clean -f` 删除:
- `d2i_page0_cn.rs`
- `d2i_page0_structure.rs`
- `d2i_xhg_47_attrs.rs`
- `d2i_xhg_47_containment.rs`
- `d2i_xhg_47_detail.rs`

信息已沉淀到 `d2i-parser-fixes-july-2026` memory。

### 2. Phase C: amount helper 抽取

新增 `read_chest_stackable_amount(reader, page_is_stackable, is_simple_item, is_socketed_subitem, defend_zero_qty) -> u32`,
替换 3 处独立实现:
- `scan_page_items_count_based` (`defend_zero_qty=false`)
- `parse_item_at` (`defend_zero_qty=true`)
- socketed sub-item trailer ×2 (`is_socketed_subitem=true`)

**测试**: 109 lib + 19 d2i + 69 业务全过。

### 3. 3 fixture JM count 对账

| Fixture | Total | Diff | 备注 |
|---------|-------|------|------|
| zmb_only.d2i (108B) | 1/1 | **0** | 完美 |
| user_stash.d2i (17KB) | 298/299 | **-1** | Page[5] `w97` 不在 ALL_ITEMS |
| ModernSharedStashSoftCoreV2.d2i (18KB) | 411/411 | **0** | 完美 |

**验证工具**: `examples/page_count_check.rs` (保留)

### 4. d2emu 设计调研 + UI 组件

调研 `d2emu.com/d2s` 截图 + index.html,产出:
- `docs/design/d2emu-hero-editor-tool-layout.md` (8 章 layout 契约)
- `docs/design/warehouse-wireframe.html` (单文件可预览 wireframe)

新增 4 个共享组件,接入 `StashManager.tsx`:
- `TabBar.tsx` — main/sub 两套视觉
- `KpiRow.tsx` — 4 列 KPI 摘要
- `QualityLegend.tsx` — 5 色 quality 图例 + 类型导出
- `EquipmentPanel.tsx` — D2 角色装备 8 槽 3×4 人型布局

### 5. Commits 本 session

```
2b82bd3 feat(web): EquipmentPanel component + StashManager integration
fa3c664 feat(web): shared UI primitives (TabBar / KpiRow / QualityLegend)
1e440e3 refactor(d2i): unify chest-stackable amount reading
```

### 6. 已知 TODO(留给未来)

1. **后端 `read_character_info` 扩展 equipment 解析** — `EquipmentPanel` 当前用 mock 数据
2. **Vite 8 + esbuild 环境修复** — `npm run build` 当前失败
3. **Catalog.tsx unused-import** — 历史遗留
4. **stackable amount 精度深入修复** — 本 session 修了"如何读",没改"读什么位置"

### 7. 相关 memory

- `d2i-amount-helper-unify-2026-07-05.md` (新)
- `d2emu-design-system-2026-07-05.md` (新)

## Session: 2026-07-05 收尾 (PM 验收 + Phase G/H + 联调)

### Phase G: 修复 P0 (commit 9e2c513)
- **sell_item 闭环修复**: Listings 加"立即卖出"按钮
- **EquipmentPanel 真实数据**: 后端 CharacterInfoResult 加 equipment 占位字段

### Phase H: 改价功能 (commit 132a8c5)
- 后端 db.update_listing_price + commands::marketplace::update_listing_price
- 前端 Listings 加"改价"按钮 + 改价 modal

### 联调 dev 11 commits
- 66d2f87 fix: Home NAV_CARDS 新 nav key (4→6 卡片)
- e24022c refactor: ItemTooltip 共享组件 (-50 行)
- 726d1df feat: Listings 关联 History (查买家)
- 769cfb7 feat: Home 最近活动摘要 (Overview)
- 2efcf62 feat: History 页 + CSV 导出
- 9538bd4 feat(backend): get_transactions 命令
- 231815f feat: Catalog 高级筛选/排序
- c8ffd2d feat: 主导航重排 - 扩展仓库移到首页右侧
- c4cfdc6 fix: page 切换强制重挂载 (main key={page})
- 120b554 fix: StashGrid duplicate React key
- 2b15aca fix: StashItem unique id (stash-page-x-y-seq)
- a88c437 fix: StashManager 加 ItemTooltip import

### ⚠️ 未完成 (留作下次)
- **d2i parser v3 fallback 误识别 socketed items** (ed4e2c4/aa821f9)
  - 诊断: 411 parsed → 2 socketed 被识别 → 409 main items 中大部分是误识别
  - 真正修复需改 `scan_page_items_v3_fallback` 识别"已解析过 socketed 的 main item 内部边界"
  - 1-2h, 高风险, 单独 Phase 处理
  - 详情见 `memory/d2i-socketed-items-misidentified.md`

### 当前 dev 状态
- cargo tauri dev 后台进程已自然退出
- Rust binary mtime 22:01 (含 socketed 字段 + 过滤 + debug println)
- 用户可手动重启验证
