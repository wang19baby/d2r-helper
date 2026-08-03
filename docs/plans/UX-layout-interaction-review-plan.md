# D2R 市场 — 布局与交互审查开发计划

> 基于 2026-07-16 全页面审查产出。包含问题分类、优先级、修复方案和责任划分。

---

## 目录

1. [P0 — 严重问题](#p0--严重问题)
2. [P1 — 高优先级](#p1--高优先级)
3. [P2 — 中优先级](#p2--中优先级)
4. [P3 — 打磨](#p3--打磨)
5. [实施建议](#实施建议)

---

## P0 — 严重问题

影响用户体验或代码可维护性的关键问题，建议首个迭代修复。

### P0-1: 触控设备悬浮操作不可用

**涉及页面**: `Warehouse.tsx`、`Inventory.tsx`、`Catalog.tsx`

**问题描述**:
- Warehouse 物品行的操作按钮（整理/取回/删除）使用 `opacity-0 group-hover:opacity-100` —— 在不支持悬浮的触控设备上永久不可见
- ItemTooltip 仅 `mode="hover"` 触发，触控无法唤出
- ItemTooltip 在每个物品卡片内直接渲染（<Inventory> 中每张卡片都有带事件监听的 tooltip 组件），大量 DOM 节点和事件监听

**目标**: 触控设备可等价操作所有功能
**影响范围**: 3 个文件

**改动方案**:
1. Warehouse 操作按钮：`opacity-0 group-hover:opacity-100` → 始终显示 icon，或在 `@media (hover: hover)` 时才隐藏；移动端始终可见
2. ItemTooltip 增加 `click` 模式或 `focus` 模式，悬浮和点击都可触发
3. ItemsTooltip 换成**全局单例**：监听 document-level mouseover/focus，只在有 target 时渲染单个 tooltip

**验证**: 触控设备（或 DevTools 模拟触控）下所有操作可完成

---

### P0-2: 销毁操作缺少确认

**涉及页面**: `Listings.tsx`、`Warehouse.tsx`

**问题描述**:
- `Listings.cancel()` 直接调用 API 取消上架，无确认环节
- `Warehouse.remove()` 直接调用 API 删除仓库物品，无确认环节
- `Listings.quickSell()` 使用 `window.confirm()` 浏览器原生弹窗，破坏 D2 主题一致性

**目标**: 所有不可逆操作必须二次确认
**影响范围**: 3 个操作

**改动方案**:
1. 创建一个 `D2ConfirmModal` 组件（主题一致，红底警告）
2. cancel / remove 改为弹 `D2ConfirmModal`
3. quickSell 改为使用同一组件

**验证**:
- 取消上架 → 弹出"确认取消"→ 确认后执行
- 删除物品 → 弹出"永久删除"→ 确认后执行
- 快速卖出 → 弹出"折价卖出"（显示折算比例）→ 确认后执行

---

### P0-3: Config 页面结构臃肿

**当前**: `Config.tsx` 767 行，单一组件，4 个手风琴区块 + 搜索 + 状态 Widget

**问题**:
- 可维护性低，新增配置项需要深入超大文件
- 搜索功能是一个带关键词匹配的 `matchAny` 辅助函数，与页面强耦合

**目标**: 将 Config 拆分为可独立维护的模块
**影响范围**: `Config.tsx`

**改动方案**:
1. 创建目录 `pages/config/`
2. 提取组件：
   - `ConfigHeader.tsx` — 标题 + 搜索
   - `ConfigStatusBar.tsx` — 3 个 StatusWidget + 系统信息
   - `GameDirectorySection.tsx` — 游戏目录手风琴
   - `ModLanguageSection.tsx` — 模组 & 语言手风琴
   - `ResourceImportSection.tsx` — 资源导入手风琴
   - `SaveBackupSection.tsx` — 存档备份手风琴
3. 创建 hooks：
   - `useImportProgress.ts` — 抽取 Config + App 中的重复轮询逻辑

**验证**: 所有功能不变，搜索仍可跨区块定位

---

### P0-4: CharacterPanel 组件臃肿

**当前**: `CharacterPanel.tsx` 988 行，7 个微标签（装备/背包/技能/小站/任务/奖励/佣兵）全在一个文件

**问题**:
- 布局和逻辑耦合，每个标签页改动都要触及大文件
- 技能、小站、任务数据同步加载（技能 JSON 685KB），增加首屏负担

**目标**: 按标签拆分为独立组件，支持延迟加载
**影响范围**: `CharacterPanel.tsx`、`data/skills.ts`、`data/waypoints.ts`、`data/quests.ts`

**改动方案**:
1. 创建 `pages/characters/` 目录
2. 提取子组件：
   - `CharacterOverview.tsx` — 装备 + 属性面板
   - `CharacterInventory.tsx` — 背包 + 方块 + 腰带
   - `CharacterSkills.tsx` — 技能树（含 SkillTree.tsx 集成）
   - `CharacterWaypoints.tsx` — 小站进度
   - `CharacterQuests.tsx` — 任务进度
   - `CharacterRewards.tsx` — 奖励摘要
   - `CharacterMerc.tsx` — 佣兵信息
3. 使用 `React.lazy(() => import('./CharacterXxx'))` + `<Suspense>` 实现标签切换懒加载
4. 技能数据（`skillTooltips.json` 685KB）改为只在 `CharacterSkills` 标签激活时加载

**验证**:
- 7 个标签切换正常
- 首屏 JS 体积减小
- 技能标签首次加载时显示 loading

---

## P1 — 高优先级

### P1-1: Modal 焦点陷阱

**涉及**: Listings 改价 Modal、SellModal、BuyModal

**问题**: Modal 打开后按 Tab 可以逃逸到背景页面元素，不符合 WCAG 2.2 2.4.3

**改动**: 创建 `useFocusTrap` hook，Modal 打开时：
1. 收集 focusable 元素列表
2. Tab / Shift+Tab 循环在列表内
3. 按 Escape 关闭
4. 关闭后 focus 回到触发元素

**验证**: Tab 循环在 Modal 内，Escape 关闭关闭

---

### P1-2: TabBar 键盘可访问性

**涉及**: `TabBar.tsx` (main + sub)

**当前**:
- `role="tablist"` + `role="tab"` + `aria-selected` 正确
- 但缺少 `onKeyDown` 处理 ArrowLeft/Right/Home/End

**改动**: 添加键盘事件处理：
- ArrowLeft: 激活前一个 tab
- ArrowRight: 激活后一个 tab
- Home: 激活第一个 tab
- End: 激活最后一个 tab
- 使用 `aria-orientation` 和 `aria-controls` 引用 tabpanel

**验证**: 键盘用户可用左右箭头切换

---

### P1-3: 大型列表缺少分页

**涉及**: `Catalog.tsx`、`History.tsx`

**当前**: 所有上架/历史记录一次性从后端加载。物品数量和记录数增长后会影响性能和 UX。

**改动**:
1. 后端 `get_listed_items` / `get_history` 增加 `LIMIT` + `OFFSET` 参数
2. 前端添加"加载更多"按钮或后端分页（推荐滚动加载 + 初始 50 条）
3. Catalog 门槛较低（当前量级不大），可先增加 LIMIT，后续视使用量再加分页组件

**验证**: 数据量大的情况下页面不会卡顿

---

### P1-4: 导入轮询逻辑重复

**涉及**: `App.tsx`、`Config.tsx`

**当前**: 两个组件各自实现了 setInterval 轮询 `get_import_progress`，逻辑重复

**改动**:
1. 创建 `hooks/useImportProgress.ts`
2. 返回 `{ running, tables, startPolling, stopPolling }`
3. App.tsx 和 Config.tsx 统一调用此 hook

**验证**: 轮询行为不变，代码减少

---

## P2 — 中优先级

### P2-1: 页面间距不一致

| 页面 | gap |
|---|---|
| Home, Inventory, Catalog, Listings, Warehouse, History | `12px` |
| Characters, RunewordCalc, Grail, Builds | `16px` |

**改动**: 统一为 `gap: 12`，在需要更宽松间距的页面额外添加 `<div style="height: 4px">` 或使用子间距。

---

### P2-2: 加载状态指示器不一致

**涉及**: 5 个页面

**当前**:
- Inventory, Catalog, Warehouse, History, Characters, Grail → 使用 `<D2EmuLoading>`（品牌化旋转符文）
- Home, Listings → 使用内联 `fa-spinner fa-spin`

**改动**: 全站统一使用 `<D2EmuLoading>`，移除内联 spinner。

---

### P2-3: Catalog 价格滑块 UX

**问题**:
- 滑块到达最大值时自动清除价格上限（`setPriceMax(null)`），用户难以理解
- 步长计算 `Math.max(100, Math.floor(priceCeiling / 100))` 导致大范围时步长过大

**改动**:
1. 将"清除限制"明确为独立按钮（现有 `×` 按钮），滑块不过最大值截断
2. 步长改用对数步进：10, 50, 100, 500, 1000...

---

### P2-4: StashManager.tsx 死代码

**问题**: `StashManager.tsx` 包含网格视图 + 物品详情 + 存入，但 `App.tsx` 未引用该组件。`parseTip` 已提取到 `lib/parseTip.ts`，StashManager 中的原始实现未删除。

**改动**:
1. 如果计划弃用则删除 `StashManager.tsx`
2. 如果计划上线则补路由（`"stash-manager"` 或替换 Inventory）
3. 无论如何删 StashManager 中的旧 `parseTip` 内联函数

---

### P2-5: `d2emu-btn-success` 类名误导

**问题**:
- CSS: `.d2emu-btn-success { background: var(--color-d2emu-red) }` —— 红色，语义为"行动"（购买、存入、确认）
- 而同尾缀 `--color-d2emu-good` 为绿色，用于文本正收益

**改动**:
1. `d2emu-btn-success` → `d2emu-btn-action`（红色主要动作按钮）
2. 或改用 `d2emu-btn-primary` 语义
3. 更新所有引用（Inventory, Catalog, Listings, BuyModal, Config, Support, StashManager）

---

### P2-6: Warehouse 过滤器窄屏换行

**问题**: 1 个输入 + 4 个 `<select>` 在 `flex-wrap` 下，窄屏换行成一列

**改动**: `< 640px` 时改用 2 列网格布局，或折叠为展开式过滤器

---

## P3 — 打磨

### P3-1: 页面切换过渡动画

**当前**: 页面切换是 `{page === 'home' && <Home />}` 即时切换

**改动**: 添加 fade + slide 过渡（10-15 行 CSS keyframe，或简单的 opacity transition）

---

### P3-2: Home 导航卡片交互迁移 CSS

**当前**: `onMouseEnter`/`onMouseLeave` 直接操作 `e.currentTarget.style`

**改动**: 迁移到 Tailwind 类 `hover:translate-y-[-2px] hover:border-[var(--color-d2emu-gold)]`，删除内联事件

---

### P3-3: Listings 操作按钮窄屏折叠

**问题**: 每张卡片 4 个按钮，`sm:grid-cols-2` 断点下横向拥挤

**改动**: 窄屏折叠为 2×2 网格，或将次要操作移入 overflow 菜单

---

### P3-4: Builds / Grail 搜索无结果反馈

**问题**: 搜索无匹配时没有任何 EmptyState 提示

**改动**: 添加统一的 `<EmptyState compact icon="magnifying-glass" title="无匹配结果" hint="换关键词试试" />`

---

### P3-5: ItemTooltip 焦点模式

**改动**: tooltip 元素加 `tabindex="0"`，`onFocus`/`onBlur` 同等于 `onMouseEnter`/`onMouseLeave` 行为

---

### P3-6: TabBar 键盘导航支持（延续 P1-2 细节）

**涉及**: `TabBar.tsx`

**改动**:
- 在 main 和 sub 变体中添加 `onKeyDown` 处理
- 支持 ArrowLeft/ArrowRight/Home/End
- 增加 `aria-orientation` 属性
- 当前 tab 设置 `tabindex="0"`，非当前 tab 设置 `tabindex="-1"`

---

## 实施建议

### 推荐顺序

```
Phase 1 — 基础安全（P0）
  ├── P0-2 销毁确认      ← 影响数据安全，最高优先级
  ├── P0-1 触控修复      ← 影响 50% 以上设备可用性
  └── P0-3/P0-4 重构    ← 可并行（不同文件）

Phase 2 — 可访问性 & 规模化（P1）
  ├── P1-1 焦点陷阱
  ├── P1-2 TabBar 键盘
  ├── P1-3 分页（先 LIMIT 后 UI）
  └── P1-4 共用 hook

Phase 3 — 代码卫生 & 一致性（P2）
  ├── P2-1 间距统一
  ├── P2-2 加载状态统一
  ├── P2-4 死代码清理
  ├── P2-5 按钮类名重命名
  └── P2-3/P2-6 次要 UX

Phase 4 — 打磨（P3）
  ├── P3-1 过渡动画
  ├── P3-2 Home 卡片
  ├── P3-3 窄屏折叠
  ├── P3-4 空结果提示
  ├── P3-5 Tooltip 焦点
  └── P3-6 TabBar 键盘（P1-2 补完）
```

### 估计工作量

| Phase | 估算（人天） | 文件数 |
|---|---|---|
| Phase 1 | 3-4 天 | 8-12 |
| Phase 2 | 2-3 天 | 6-10 |
| Phase 3 | 1-2 天 | 8-10 |
| Phase 4 | 1 天 | 6-8 |
| **合计** | **7-10 天** | **~25-40** |

### 注意事项

1. **P0-1 (ItemTooltip 全局化)** 涉及组件重构，建议单独分支
2. **P0-4 (CharacterPanel 拆分)** 涉及 Lazy Loading，需要确认 React.Suspense 与当前构建工具兼容
3. **P1-3 (分页)** 需要前后端协同修改，后端接口加 LIMIT/OFFSET
4. **P2-4 (StashManager)** 先确认产品方向再决定删还是接
