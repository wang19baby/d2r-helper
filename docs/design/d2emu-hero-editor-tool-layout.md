# d2emu Hero Editor (Save Editor) 工具布局拆解

> 参考站点: <https://d2emu.com/d2s> (canonical URL — `/hero` 旧路径已 301 到 `/d2s`)
> 抓取时间: 2026-07-05
> 配套文档: `d2emu-hero-design-system.md` (设计令牌/通用组件) — 本文档专注于**工具型 UI** 的具体布局
> 目标: 把"Save Editor"那种信息密集型工具的布局语言迁移到 `Inventory` / `StashManager` / `Marketplace` 页面

---

## 0. 与设计系统文档的关系

`d2emu-hero-design-system.md` 提取的是**通用设计 token**(颜色/字体/按钮/卡片)。
本文档聚焦**功能布局**:7-cell 装备网格、Belt 4×4、Stash 多页签、D2 物品 cell 的
quality border 系统、GOLD 步进器、tab 切换条等。**两者应同时使用**:
设计 token 给样式,本文给布局与组件契约。

---

## 1. Hero Editor 整体信息架构(自顶向下 5 段)

```
┌─────────────────────────────────────────────────────────────────────┐
│ [H E R O   E D I T O R]                          [×]               │  ← 标题栏 (trajan gothic)
├─────────────────────────────────────────────────────────────────────┤
│ [⚠ Welcome to d2emu hero editor!]      ┌─────────────────────────┐  │
│ [ROTW support! Create and move items] │    [☁]  LOAD HERO SAVE   │  │  ← 通知条 + 拖拽上传
│                                       │ Drop .d2s + .d2i here   │  │
├───────────────────────────────────────┴─────────────────────────┤
│ [avatar][NAME]  [Level ±99] [Class:Warlock] [HC][Dead] [3 个 CTA]  │  ← 角色主行
│         ECHOINGSTRIKE                          [DOWNLOAD SAVE]    │
│                                              [DOWNLOAD STASH]     │
│   Advanced Game Fields ▶                     [SAVE + SHARE]      │
├─────────────────────────────────────────────────────────────────────┤
│ [INVENTORY|CHRONICLE|SKILLS|STATS|WAYPOINTS|QUESTS|BOUND DEMON]   │  ← Tab Bar
│                                                  [☐ Display Names] │  ← 切换
├─────────────────────────────────────────────────────────────────────┤
│   Equipment 7格       Belt 4×4       Stash 8 列 × 多行               │  ← 三栏工具区
│                                  [Personal|Shared1-5|Stackables|Temp]│  ← Stash sub-tabs
│   Inventory 4×4     Horadric Cube    Mercenary 装备                  │
│   [GOLD 990000]    [Clear Inventory]    [GOLD 2500000]  [Clear Stash]│
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. 关键视觉规范(实测,非猜测)

### 2.1 颜色(来自 hero-editor-thumb.png 像素级观察 + index.html inline style)

| Token | Hex | 出现位置 | 备注 |
|---|---|---|---|
| `--d2emu-bg` | `#0a0a0a` | 页面底 | meta `theme-color` 也是这个 |
| `--d2emu-panel` | `#1a1d21` | `<script>` 注入到 `documentElement` | 比 `--d2emu-bg` 略浅一档 |
| `--d2emu-text` | `#e5e2de` | 浅米色正文 | 不用纯白,带温感 |
| `--d2emu-gold` | `#c7b377` | Welcome 边框 / Support 按钮 | 比 #FBB13A 暗一档,**实际品牌色** |
| `--d2emu-gold-deep` | `#a08655` | 烫金渐变底部 | |
| `--d2emu-red` | `#800000` | Hover 红 / 主行动 | |
| `--d2emu-cell-border` | `#252525` | 物品 cell 静默状态 | |
| `--d2emu-cell-active` | `#c7b377` | 选中物品的金边框 | |
| `--d2emu-quality-unique` | `#c7b377` | Unique 物品外框 | = gold |
| `--d2emu-quality-set` | `#2c8c3a` | Set 物品外框 | 暗绿 |
| `--d2emu-quality-rare` | `#c4a847` | Rare 物品外框 | 暗黄 |
| `--d2emu-quality-magic` | `#5d6cff` | Magic 物品外框 | 暗蓝 |
| `--d2emu-quality-socketed` | `#6a4a8a` | 紫色 socket 框 | |
| `--d2emu-tab-active` | `#800000` | INVENTORY 红底 | 反白字 |

### 2.2 字体

| 角色 | 字体 | 来源 | 备注 |
|---|---|---|---|
| H1 "H E R O  E D I T O R" | Cinzel / serif + `letter-spacing: 3px` + `text-transform: uppercase` | inline style | 全大写、字符间距 3px 是关键 |
| Body / 表单 | Roboto | `font-family: 'Roboto', sans-serif` | d2emu 主站通用字体 |
| 物品 cell 数量 badge | Roboto Mono | 视觉观察 | 等宽对齐 |

> **d2emu 没有用 Blizzard 官方 Exocet 字体**(那是 d2emu 主站其他页面用的)。
> Hero Editor 截图里看到的"暗黑奇幻感" 来自**字距 + 全大写 + 暗背景**,不是字体本身。

### 2.3 物品 Cell (`.d2emu-cell`)

**实测尺寸 64×64 px**(从 hero-editor-thumb.png 的 1881×1080 分辨率推算)。

```css
.d2emu-cell {
  width: 64px;
  height: 64px;
  background: rgba(0,0,0,0.35);
  border: 1px solid var(--d2emu-cell-border);
  border-radius: 2px;
  position: relative;
  display: grid;
  place-items: center;
  cursor: pointer;
  transition: border-color 120ms, box-shadow 120ms;
}
.d2emu-cell:hover { border-color: var(--d2emu-cell-active); }
.d2emu-cell.is-selected {
  border-color: var(--d2emu-gold);
  box-shadow: 0 0 0 2px rgba(199,179,119,0.25);
}

/* Quality border 2px, override 1px default */
.d2emu-cell[data-quality="unique"]   { border: 2px solid var(--d2emu-quality-unique); }
.d2emu-cell[data-quality="set"]      { border: 2px solid var(--d2emu-quality-set); }
.d2emu-cell[data-quality="rare"]     { border: 2px solid var(--d2emu-quality-rare); }
.d2emu-cell[data-quality="magic"]    { border: 2px solid var(--d2emu-quality-magic); }
.d2emu-cell[data-quality="socketed"] { border: 2px solid var(--d2emu-quality-socketed); }

/* Quantity badge */
.d2emu-cell-qty {
  position: absolute;
  right: 2px;
  bottom: 2px;
  background: rgba(0,0,0,0.85);
  color: #fff;
  font: 600 11px/1 'Roboto Mono', monospace;
  padding: 2px 4px;
  border-radius: 2px;
}
```

### 2.4 Stash Tabs(顶端 sub-tab 条)

```css
.d2emu-subtab-bar {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--d2emu-cell-border);
  margin-bottom: 8px;
}
.d2emu-subtab {
  padding: 6px 14px;
  background: transparent;
  border: 1px solid transparent;
  border-bottom: none;
  color: var(--d2emu-text-muted);
  font: 600 11px/1 'Roboto', sans-serif;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  cursor: pointer;
  border-radius: 4px 4px 0 0;
}
.d2emu-subtab.is-active {
  background: var(--d2emu-panel);
  border-color: var(--d2emu-cell-border);
  color: var(--d2emu-text);
}
```

---

## 3. 三栏工具布局

### 3.1 主容器 grid

d2emu 用了**手写 flex 分布 + 固定 cell 宽**,没用 CSS Grid。
我们的 marketplace 应该统一用 CSS Grid 干净一些:

```css
.d2emu-editor-layout {
  display: grid;
  grid-template-columns: minmax(280px, 1fr) minmax(240px, 1fr) minmax(320px, 2fr);
  gap: 16px;
  padding: 16px;
}

/* 响应式: <1100px 折叠 stash 为第二行 */
@media (max-width: 1099px) {
  .d2emu-editor-layout { grid-template-columns: 1fr 1fr; }
  .d2emu-editor-stash { grid-column: 1 / -1; }
}
```

### 3.2 Equipment 7-cell 布局

D2 角色身上 7 个装备位:**Helm / Amulet / Armor / Belt / Gloves / Boots / Weapon + Shield**

```css
.d2emu-equipment {
  display: grid;
  grid-template-columns: 64px 64px 64px;   /* 3 列 */
  grid-template-rows: 64px 64px 64px 64px;
  gap: 6px;
  width: max-content;
}
/* Helm 在中上,Weapon+Shield 在左右中行,Belt 在底部居中 */
.d2emu-equipment-slot-helm    { grid-area: 1 / 2; }
.d2emu-equipment-slot-amulet  { grid-area: 2 / 2; }
.d2emu-equipment-slot-armor   { grid-area: 3 / 2; }
.d2emu-equipment-slot-weapon  { grid-area: 3 / 1; }
.d2emu-equipment-slot-shield  { grid-area: 3 / 3; }
.d2emu-equipment-slot-gloves  { grid-area: 4 / 1; }
.d2emu-equipment-slot-boots   { grid-area: 4 / 3; }
.d2emu-equipment-slot-belt    { grid-area: 4 / 2; }
```

> **可视化复刻 D2 装备人型图** — 玩家一眼就能认出。

### 3.3 Inventory grid (4 列 × 4 行)

固定 4×4 = 16 cell。Stackable items 用 qty badge。

```css
.d2emu-inventory {
  display: grid;
  grid-template-columns: repeat(4, 64px);
  grid-template-rows: repeat(4, 64px);
  gap: 6px;
}
```

### 3.4 Stash 8 列 × N 行

Stash 是 D2 经典 8 列 stash(每页 6 行 = 48 cell)。我们 marketplace 可以保持 8 列,
**行数根据实际内容自适应**(`grid-auto-rows: 64px`)。

```css
.d2emu-stash {
  display: grid;
  grid-template-columns: repeat(8, 64px);
  grid-auto-rows: 64px;
  gap: 4px;
}
```

---

## 4. Tab Bar 设计(主区域切换)

d2emu 的 tab bar 用了**底部边框消失 + 活动 tab 红色实底**:

```css
.d2emu-tab-bar {
  display: flex;
  gap: 0;
  background: var(--d2emu-bg);
  border-bottom: 2px solid var(--d2emu-cell-border);
}
.d2emu-tab {
  padding: 10px 18px;
  background: transparent;
  color: var(--d2emu-text-muted);
  font: 700 12px/1 'Roboto', sans-serif;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  cursor: pointer;
  border: none;
  border-right: 1px solid var(--d2emu-cell-border);
}
.d2emu-tab.is-active {
  background: var(--d2emu-red);
  color: #fff;
}
.d2emu-tab:hover:not(.is-active) {
  background: rgba(128,0,0,0.15);
  color: var(--d2emu-text);
}
```

**d2emu 的 7 个 tabs**: `INVENTORY / CHRONICLE / SKILLS / STATS / WAYPOINTS / QUESTS / BOUND DEMON`

我们对 marketplace 的建议 tab(7 个也合适):

| Tab | 内容 |
|---|---|
| INVENTORY | 我的 runes(从 stash 提取) |
| MARKETPLACE | 浏览在售 runes |
| LISTING | 我的上架(可改价/取消) |
| HISTORY | 已成交订单 |
| WAREHOUSE | 收藏(扩展仓) |
| SETTINGS | 配置 save_folder / game_root |
| SUPPORT | 帮助/反馈 |

---

## 5. 数字步进器 + GOLD 输入

d2emu 在 Inventory 和 Stash 底部各有一个 **GOLD 数字 + Clear 按钮**:

```css
.d2emu-row-with-action {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid var(--d2emu-cell-border);
}
.d2emu-stepper-inline {
  display: grid;
  grid-template-columns: 28px 90px 28px;
  align-items: center;
}
.d2emu-stepper-inline button {
  background: #1f1f1f;
  color: #fff;
  border: 1px solid var(--d2emu-cell-border);
  height: 28px;
  cursor: pointer;
}
.d2emu-stepper-inline input {
  text-align: center;
  background: #1a1a1a;
  border: 1px solid var(--d2emu-cell-border);
  border-left: 0;
  border-right: 0;
  height: 28px;
  color: var(--d2emu-gold);
  font: 600 13px/1 'Roboto Mono', monospace;
}
.d2emu-row-label {
  color: var(--d2emu-text-muted);
  font: 700 11px/1 'Roboto', sans-serif;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
```

---

## 6. 与 marketplace-tauri 的功能 mapping

| d2emu 元素 | 我们 marketplace 对应 | 优先级 |
|---|---|---|
| **LOAD HERO SAVE** 拖拽区 | "导入 .d2i + .d2s" 按钮区 | P0 |
| **角色主行**(头像+NAME+Level+Class) | 角色卡片(从 d2s 解析) | P0 |
| **3 个下载按钮** | "上架 / 取消 / 卖出" | P1 |
| **Tab Bar** | 主导航(7 个 tab) | P0 |
| **Equipment 7 cell** | "角色身上装备"(信息展示,不交易) | P2 |
| **Belt 4×4** | 不需要 | - |
| **Inventory 4×4** | "我的 runes"(从 stash 提取的) | P0 |
| **Stash 8 列 × N 行** | "扩展仓 + 在售物品"(不同 page) | P0 |
| **Stash sub-tabs** | Personal / Shared 1-5 / Stackables / Temp | P1 |
| **GOLD 输入** | 价格步进器(rune 数量 × 单价) | P0 |
| **Display Names 切换** | 中文名 / 英文名 toggle | P1 |
| **Clear Inventory** | "全部撤架" 操作 | P2 |
| **Horadric Cube** | 不需要 | - |
| **Mercenary** | 不需要 | - |
| **Chronicle** | 不需要 | - |
| **Skills / Stats / Waypoints / Quests** | 不需要(marketplace 只管交易) | - |
| **Bound Demon** | 不需要 | - |

### 推荐的 marketplace 主布局

```
┌─────────────────────────────────────────────────────────────────────┐
│ H E R O   E D I T O R (我们改为 "MARKETPLACE TAURI")               │
├─────────────────────────────────────────────────────────────────────┤
│ [⚠ Welcome!]                                  ┌──────────────────┐  │
│ [当前市场公告]                                  │ LOAD HERO SAVE   │  │
│                                              │ Drop .d2s+.d2i   │  │
├──────────────────────────────────────────────┴──────────────────┤
│ [avatar] ECHOINGSTRIKE  Lv 99  Warlock  [HC][Dead]                 │
│                                              [上架] [取消] [卖出]  │
├─────────────────────────────────────────────────────────────────────┤
│ [INVENTORY|MARKETPLACE|LISTING|HISTORY|WAREHOUSE|SETTINGS|SUPPORT] │
├─────────────────────────────────────────────────────────────────────┤
│   My Runes 4×4     Stash 8×N (sub-tabs)                              │
│                  [Personal|Shared1-5|Stackables|Temp]               │
│   [PRICE ±] [BUY]                          [GOLD ±] [Clear Stash]   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 7. 风险与说明

- **装备图资源**: d2emu 不展示装备图(只展示 cell 边框 + 数量 badge)。
  我们需要 64×64 PNG 图标。数据源:
  - Diablo 2 经典 28×28 invset.dc6 → 转 PNG (Blizzard 版权风险,**不直接复制**)
  - 推荐: 用 emoji + CSS 渐变模拟 (🎰⚔️🛡️⛓️) + 装备 code (r06, r07) 文本
- **Quality border 颜色**: 上面 token 是基于 d2emu 截图视觉估色,**不是 d2emu 官方规范**。
  我们应该用 Diablo 2 官方 token(玩家已经习惯):
  - Unique: `#9c8d65` (烫金)
  - Set: `#1a9c1a` (暗绿)
  - Rare: `#c4a847` (暗黄)
  - Magic: `#5d6cff` (暗蓝)
- **响应式**: d2emu 在 899px 以下折叠侧栏,桌面端我们默认 1080px+ 完整布局。

---

## 8. 落地路径

1. **`web/src/components/ItemCell.tsx`** (新) — 接受 `code / quality / quantity` props,
   渲染 64×64 cell + quality border + qty badge
2. **`web/src/components/InventoryGrid.tsx`** (新) — 接受 `columns/rows/items` props,
   渲染 grid
3. **`web/src/components/TabBar.tsx`** (新) — 通用 tab bar 组件
4. **`web/src/components/Stepper.tsx`** (新) — 数字步进器
5. **`web/src/pages/MarketplaceLayout.tsx`** (新) — 复用三栏 grid
6. **`web/src/index.css`** — 追加 `.d2emu-cell / .d2emu-tab / .d2emu-stepper` 等样式

> 与设计系统文档 (`d2emu-hero-design-system.md`) 配合使用。
> 两份文档提供完整的 d2emu 设计迁移参考,**先有样式 token,后有布局契约**。