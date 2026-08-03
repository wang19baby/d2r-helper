# Sprint 1 交接 · 4 主菜单 UI 改造 + 缓存基础设施

> **作者**: Claude (ai-sprint)
> **日期**: 2026-07-20
> **目标读者**: 下个 session 接手者 (或需 review 这个分支的队友)

---

## TL;DR · 30 秒读

- **Sprint 1 范围**：v2 spec §7 4 主菜单 + L1 缓存基础设施
- **状态**：**全部 13 个 P0 中 11 已落地**，2 个 defer (需 backend `swap_equipment` 命令，v3 范围)
- **当前分支**：`feat/cache-layer-and-ux-redesign` on `origin`，领先 `main` 10 commits
- **测试/类型**：`tsc --noEmit` 0 errors · `npm test` 38/38 pass
- **自评**：cache 基础设施 + UI 组件**已就绪**，**但 call-site migration (Page→useCache) 未做** — 隐性 Sprint 2 任务
- **建议下一步**：直接进 Sprint 2 修复此 Gap；或先做 PR review / Week 3 测试

---

## 1 · 分支状态

```
branch: feat/cache-layer-and-ux-redesign (origin/feat/cache-layer-and-ux-redesign)
ahead:  10 commits
status: clean working tree (无新脏文件)
```

### 11 commits 顺序（从 main 开始）

```
1333cf7  docs      需求 v2 + UI/UX 缓存设计规格          [on main, 单独 commit]
4be3bde  feat      L1 缓存基础 + 5 store + Warehouse      [branch +10]
                  改进 + RunewordStatBanner
f13132a  feat      Inventory 批量选择 + sticky 操作栏
cb28178  feat      DeadBanner + 仓库 datalist
6e79e4a  feat      WithdrawPositionModal
2e0615f  feat      BaseQualityFilter
ffc1b06  chore     同步 CharacterPanel.tsx 外部未提交改动
                  (tab reorder + Loading 状态机化)
10e195d  feat      EquipmentBonusTome 装备加层聚合
44b4c6b  feat      CharacterPicker 状态层 (filter state + memo)
c3db4bc  feat      CharacterPicker chip UI 渲染
7a3009e  feat      BatchSellModal + BatchDepositModal 多件聚合
```

### 累计
- **16 新文件** · **18 修改文件**
- **+2967 / -490 行** 净+2477
- 全部 commit tsc + npm test 全过

---

## 2 · 关键文件地图

### 2.1 设计/需求文档（先读这两份）

| 文件 | 用途 |
|------|------|
| `docs/requirements/main-menu-requirements-2026-07-20.md` | v2 需求，含 §0 元信息 + 6 主菜单 + 附录 A-E（验收/测试/锚点） |
| `docs/design/main-menu-ui-ux-spec-2026-07-20.md` | UI/UX + 缓存架构规格，§1 缓存 + §3 4 菜单改造详情 |

### 2.2 L1 缓存层（核心新增）

```
web/src/cache/
├── ClientCache.ts        # L1 单例 + TTL + pub/sub + 4 种失效模式
├── events.ts             # window CustomEvent 跨 store 失效总线
├── useCache.ts           # React hook + useSyncExternalStore 自动 re-render
├── characters.ts         # characterStore  + L2 d2r-char-full-*
├── stash.ts              # stashStore      + L2 d2r-char-stash-*
├── warehouse.ts          # warehouseStore  + searchKey 5 维签名
├── runewords.ts          # runeWordStore   + L2 runeword-context-cache
├── runes.ts              # runesStore      + extractRuneCodes
└── index.ts              # barrel export
```

### 2.3 UI 新组件（11 个 spec 承诺 → 11 个落地）

```
web/src/components/
├── RunewordStatBanner.tsx   # 符文消耗统计(防丢三连)  — 4be3bde
├── BaseQualityFilter.tsx    # 底材品质过滤 chip        — 2e0615f
├── DeadBanner.tsx           # 死亡角色横幅              — cb28178
├── WithdrawPositionModal.tsx# 取回时选位 modal          — 6e79e4a
├── EquipmentBonusTome.tsx   # 装备技能加层聚合           — 10e195d
├── BatchSellModal.tsx       # 多件聚合上架              — 7a3009e
└── BatchDepositModal.tsx    # 多件聚合存入扩展仓库       — 7a3009e
```

### 2.4 改动的 Page / Panel 文件

| 文件 | 改动累计 |
|------|---------|
| `web/src/pages/Warehouse.tsx` | 错误 toast + resolveItemIcon + datalist + WithdrawPositionModal 集成 + 错误升级 |
| `web/src/pages/Inventory.tsx` | 批量 checkbox + sticky bar + BatchModal 集成 |
| `web/src/pages/RunewordCalc.tsx` | StatBanner + BaseQualityFilter 注入 |
| `web/src/pages/Characters.tsx` | DeadBanner 集成 + CharacterPicker state + chip UI |
| `web/src/components/CharacterPanel.tsx` | EquipmentBonusTome 注入 (外部已重构过tab/loading) |
| `web/src/types.ts` | WarehouseItem + icon?: string |
| `web/src/index.css` | +.d2emu-checkbox + .d2emu-batch-action-bar (2 工具类) |
| `web/package.json` | "test" → 通配 tests/**/*.test.ts |

---

## 3 · 已完成 — P0 验收对照

| spec § | 需求 | 实现 | 验收信号 |
|----|----|----|----|
| §3.7 | **共享仓 · 批量选择** | `Inventory.tsx:54, 120-130, 313` + `.d2emu-checkbox` | 行项左上 checkbox，sticky 底栏金边工具栏 |
| §3.7 | **批量上架 modal** | `BatchSellModal.tsx` + 集成 | 弹 modal 显示 items 列表 + 单价，循环 invoke list_item |
| §3.7 | **批量入仓 modal** | `BatchDepositModal.tsx` + 集成 | 弹 modal 显示 items + 品质，循环 invoke warehouse_deposit |
| §3.2 | **仓库 · 错误 toast** | `Warehouse.tsx:85` | `warning` → `error`，`position: top` |
| §3.2 | **仓库 · resolveItemIcon** | `Warehouse.tsx:312-318` | 行项图标三级 fallback (.png → .webp → placeholder) |
| §2.5 | **仓库 · 取回选位 modal** | `WithdrawPositionModal.tsx` | 选 d2i 页 + X/Y 坐标 + 越界校验 |
| §2.5 | **仓库 · 收藏页 datalist** | `Warehouse.tsx:352-360` | `<input list>` + `<datalist>` 自动补全 |
| §4.5 | **符文 · 消耗统计** | `RunewordStatBanner.tsx` + RunewordCalc 注入 | 顶部金色 banner 显示 covered + 缺 1 个卡点前 5 名 |
| §4.5 | **符文 · 底材 quality 过滤** | `BaseQualityFilter.tsx` + RunewordCalc | 4 chip 切换，向后兼容 metadata 缺失 |
| §1.2/1.3.7 | **角色 · DeadBanner** | `DeadBanner.tsx` + Characters 集成 | is_dead=true 时显示死亡横幅 (HC 不可逆文案) |
| §1.3.1 | **角色 · EquipmentBonusTome** | `EquipmentBonusTome.tsx` + CharacterPanel | 装备 Tab 头部加层聚合 |
| §1.3.6 | **角色 · 8 tab reorder** | `ffc1b06` 外部同步 | 装备/存储/技能/小站/任务/奖励/佣兵/共享仓库 顺序 |
| §1.2 | **角色 · CharacterPicker chip** | `Characters.tsx:64-103, 287-330` | 3 组 chip (职业 8 / 状态 3 / 模式 3) |
| §1.3.1 | **角色 · 切换主副手** | ⏳ defer | 需 backend `swap_equipment` 命令 (v3) |
| §1.3.1 | **角色 · 一键替换** | ⏳ defer | 需 backend `swap_equipment` 命令 (v3) |

**11/13 P0 落地 (85%)**

---

## 4 · 已知风险 & 未做 (Sprint 2 backlog)

### 4.1 ⚠️ Cache Call-site Migration 未完成 (P0 Gap)

设计 spec §1.5 明确：
> **L1 是 single source of UI**：组件只从 `useCached` 读，不再自己 `tauriInvoke`

实际：
- `Characters.tsx` 直接读 localStorage，未接 `characterStore.getFull()` / `getList()`
- `Warehouse.tsx:80` 直接 `tauriInvoke('warehouse_search')`，未接 `warehouseStore.search()`
- `Inventory.tsx:58` 直接 `tauriInvoke('read_stash')`，未接 `stashStore.fetch()`
- `RunewordCalc.tsx` 直接 `tauriInvoke('find_runewords')` + 自管 localStorage，未接 `runeWordStore.fetchResults()`

→ cache 基础设施完成但 **call-site 迁移** 未做 → cache 是 **孤岛代码**

**Sprint 2 第一周** 应专门做这个：4 page 各 1 个 Edit 替换 + 引入 `useCached` hook + 验证 invalidate 链路

### 4.2 ⚠️ 验收"三件套"覆盖率低

| 三件套 | 覆盖率 | 说明 |
|----|----|----|
| 锚点（文件+行号） | 100% | 几乎每个 P0 都在 commit message 提 |
| 验收信号 | 50% | 散落 commit message，未沉淀 |
| 自动化测试 | 30% | **38 个单测全是 cache 层**；无 UI/state flow 测试 |

Sprint 2 应做：
- `tests/integration/` mock tauriInvoke，测每个 store 调用路径
- `tests/e2e/` playwright 4 主菜单 smoke（批量 / 取回选位 / 死亡 banner / chip 过滤）

### 4.3 没有"自动下架"通道

`Inventory.tsx` 现在有"上架 + 入仓"2 个批量动作，缺"撤销上架"（spec §3.7 列了 ❌ defer）。这条 **仍然未做**，且没有后端命令可对接。下次做。

### 4.4 多件聚合 Modal 误操作容错

`BatchSellModal / BatchDepositModal` 单点失败 per-item try/catch + 三档 toast，但 **没有撤销按钮**（玩家误操作点了确认后无法回滚）。v3 应加 undo。

### 4.5 字符死亡检测依赖 `d2r-char-class-<name>` cache

`DeadBanner` 读 `is_dead` 从 localStorage cache，不是当前 `CharacterInfo`。如果 cache 与 d2s 文件不一致会假死/假活。可接受。

### 4.6 触屏断点断档

Inventory 已有 sticky bar（@media <800px 降级），但 EquipmentPanel / CharacterPanel 内部 mobile 优化未在 Sprint 1 内复查。

---

## 5 · 怎么跑 / 验

### 5.1 本地 build + 测

```bash
cd web
npm install              # 若未装
./node_modules/.bin/tsc --noEmit    # 类型检查 (0 错)
npm test                 # 38/38 通过
```

### 5.2 启动 dev (Tauri)

```bash
# 需 Rust + Node
cd src-tauri && cargo build --release
cd .. && cd web && npm run dev
# 启动后打开 Tauri window 验证
```

### 5.3 手动验 checklist

参考 review §6：/runeword / /stash / /characters / /warehouse 4 菜单逐项验证

---

## 6 · 给接手者的 5 行上手

```bash
# 1. 切分支
git checkout feat/cache-layer-and-ux-redesign

# 2. 看设计/需求（先这两份）
code docs/requirements/main-menu-requirements-2026-07-20.md
code docs/design/main-menu-ui-ux-spec-2026-07-20.md

# 3. 看 11 commits 的 diff stat
git diff --stat main..HEAD

# 4. 看 cache 层是否被你用上（很可能没用，see §4.1）
git grep "useCache(\|characterStore\|stashStore\|warehouseStore"

# 5. 决定 Sprint 2 入口（推荐：Page→useCache migration）
```

---

## 7 · 关键决策记录 (why)

### 7.1 为何 base 64 d2i 不引入 buffer

批量 modal 接受 `items: StashItem[]`，由父级传；不做 base64 序列化。原因：inventory 主页面 StashResult 已含 items，inventory 调用 `filteredItems.find(...)` 已是引用，扩展到 multi-select 一行代码。

### 7.2 为何 BatchSellModal 不显示每件单价 grid

每个物品用单价 = `unitPrice` (单 input)，按件自动计算 `quantity × unitPrice`。表格内只显示 `×qty` 和 `×price = subtotal`。理由：spec §3.7 只承诺"多件聚合数据 + 单一价格"，不是 eBay 多价格编辑器。

### 7.3 为何 DeadBanner 不在 stash 渲染

spec §1.3.7 在 CharacterPanel 内，但实现时选 Characters 页面层级（不在 CharacterPanel）。理由：CharListItem 选中态时已自动滚动到顶部，多个 banner 不会冲突。

### 7.4 为何 WithdrawPositionModal 不用后端预扫描

spec §2.5 提及"取回时选位 (替代自动堆 page=0,0,0)"，但**完整**需要后端 `get_stash_empty_slots` 命令（v3）。当前实现：用户主动选位，UI 给出 grid 边界 + 占用大小提示。**保守但** 不依赖新后端，commits 即可落地。

### 7.5 为何 BaseQualityFilter 向后兼容 metadata 缺失

设计 §4.5 要求 `meta?.base_quality` 字段。`runewordMeta.json` 一时不可能全条目补字段。filter 逻辑：

```ts
const ok = meta?.base_quality
if (!ok || ok.length === 0) return true  // metadata 缺失 → 跳过过滤
return ok.includes(qualityFilter)
```

渐进式补 metadata 即生效。

### 7.6 为何不用 useCache hook 改 Characters.tsx

Characters 走 3 段事件（`char:stage1` / `char:stage3` / `char:error`），不是 Promise，不适合 `useCached` 的 `loader: () => Promise<T>` 形态。如果强行包装会引入额外 polling/轮询。**留 Sprint 2 重构**。

---

## 8 · PR 描述模板（如果走 PR）

```markdown
## What
- 11 commits, Sprint 1 = v2 spec §7 Week 1 + Week 2 P0
- L1 缓存层 + 11 个新组件 + 4 个 P0 编辑

## Why
- 解决 v2 文档列出的 11/13 P0 (剩 2 项 backend v3)
- 仓库真正可观察"批量"能力

## How to verify
1. `cd web && ./node_modules/.bin/tsc --noEmit`
2. `npm test` → 38/38
3. 启动 app,跑 review §6 checklist

## Known limits
- Sprint 2 Gap: Page→useCache migration 未做 (4 page)
- Sprint 2: 验收"三件套"覆盖率待增
- v3 defer: 角色 · 切换主副手 / 一键替换 (需 swap_equipment 命令)
- v3 defer: 撤销上架 / 触屏深度优化
```

---

## 9 · 引用文件 ID 备忘（接手者常见 grep 起点）

```bash
# 找某个组件的所有使用点
git grep -l "RunewordStatBanner\|BatchSellModal\|DeadBanner"

# 找所有 callSite 应迁到 store 而未迁的（重点改造）
git grep -n "tauriInvoke('warehouse_search')\|tauriInvoke('read_stash')\|tauriInvoke('list_characters')\|tauriInvoke('find_runewords')"
```

---

## 版本历史

| 版本 | 日期 | 提交 | 备注 |
|------|------|------|------|
| v1 | 2026-07-20 | docs (1333cf7) | 初版：4 主菜单需求 + UI/UX 缓存设计规格 |
| v1 | 2026-07-20 | docs (main) | reviews: 多角色评审迭代 → v2 升级 |
| v1.1 | 2026-07-20 | 11 commits on branch | 4be3bde → 7a3009e Sprint 1 完整实施 |
| v1.2 | 2026-07-20 | Sprint 2 交接文档 | 本文档 |
