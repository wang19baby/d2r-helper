# 一级菜单 · UI/UX 与前端缓存设计规格

> **版本**：v1 / 2026-07-20
> **依据**：`docs/requirements/main-menu-requirements-2026-07-20.md`（P0/P1 需求）
> **目标**：把 4 个一级菜单（角色 / 仓库 / 共享仓库 / 符文计算）的 UI 改造与前端缓存策略，固化为可执行的开发规格
> **设计系统**：沿用现有 d2emu hero 风格 + Gothic Grimoire 装饰层（`docs/design/d2emu-hero-design-system.md`）

---

## §0 · 设计系统速查（沿用 + 不重发明轮子）

### §0.1 色彩令牌（已存在于 `index.css`）

| 类别 | Token | 用法 |
|------|-------|------|
| 背景 | `--color-d2emu-bg: #0a0a0a`<br>`--color-d2emu-panel: #111111`<br>`--color-d2emu-panel2: #0e0e0e`<br>`--color-d2emu-field: #1a1a1a` | 页面 / 卡片 / 字段背景，三级灰度 |
| 描边 | `--color-d2emu-line: #252525`<br>`--color-d2emu-line-soft: #1a1a1a` | 卡片 / 输入框描边 |
| 文字 | `--color-d2emu-text: #e8e8e8`<br>`--color-d2emu-muted: #aaaaaa` (4.2:1 对比度)<br>`--color-d2emu-label: #908d89` | 主文 / 副文 / 小标 |
| 主金 | `--color-d2emu-gold: #FBB13A` (95/767 ≈ 12% 亮色)<br>`--color-d2emu-gold-bright: #ffffff` | 高亮 / 烫金 KPI / 标题字 |
| 链接 | `--color-d2emu-link: #c7b377` | 内联链接 |
| 状态 | `--color-d2emu-red: #800000`<br>`--color-d2emu-blue: #4f83c7`<br>`--color-d2emu-orange: #b87020`<br>`--color-d2emu-bad: #ef5350`<br>`--color-d2emu-good: #4caf50` | 错误 / 信息 / 提醒 / 成功 |
| Gothic | `--c-blood #8b0a0a`<br>`--c-vellum #1a1612`<br>`--c-gold-leaf #b8923a`<br>`--c-arcane-glow rgba(201,163,74,0.18)` | 角色 Tab / portrait 装饰 |

**使用规则**：
- **绝不新建颜色 token**（除非 §A.4 列出且设计稿校验）
- **不混用 d2-* 和 d2emu-***（字符上下文用 d2-，扩展仓 / D2R 助手功能用 d2emu-）
- **WCAG AA 对比度 >= 4.5:1**（已校验 muted 4.2→6.5 升级）

### §0.2 间距 / 半径 / 过渡

| Token | 值 | 用途 |
|-------|---|------|
| `--space-xs/sm/md/lg/xl/2xl` | 4/8/12/16/24/32 | 统一间距（任何 padding/gap 必选其一） |
| `--radius-sm/md/lg` | 3/6/8 | 卡片 / 按钮 / 大块装饰 |
| `--ease-d2: 120ms ease` | 120ms | 微交互 |
| Tab fade | 160ms | TabBar 切换 |
| Page turn | 160ms | Page-level 切换 |
| Toast in/out | 250/200ms | `.d2emu-alert-in/out` |

### §0.3 字体（已自托管，无 CDN 依赖）

```
Cinzel 400/600/700/900       --font-serif / --font-d2emu-title
Crimson Pro 400/600 (+italic) --font-body / IM Fell English SC
Source Sans 3 300/400/600/700 --font-d2emu-ui
JetBrains Mono              --font-mono-num
IM Fell English SC          --font-arcane-label
```

### §0.4 复用组件（已有，无需新建）

| 组件 | 用途 | 复用点 |
|------|------|--------|
| `D2EmuCard` | 卡片容器（kicker/title/lede/tags/actions/children） | 仓库 / 共享仓 / 装备详情 |
| `KpiRow` | 4 列 KPI 摘要 | Hero 区下方 |
| `TabBar` (variant main/sub) | 主导航 / 子 tab | 角色 8 tab / stash 类目 |
| `StatusWidget` | 锁屏 / 警示横幅 | 魔改存档提示 |
| `D2ConfirmModal` | 二次确认 | 存入仓库 / 删除 |
| `Toast` (含 `.d2emu-alert-*`) | 顶部提示 | 全局 |
| `EmptyState` | 空态 | 仓库 / 共享仓 / 角色无选择 |
| `D2EmuLoading` | 加载态 | stash / warehouse |
| `ItemTooltip` | 物品悬浮提示 | 物品卡 |
| `EquipmentPanel` | 12 槽装备人型 | 角色装备 Tab |
| `InventoryView` | 背包三段布局 | 角色背包 Tab |
| `SkillTree` | 3 系技能树 | 角色技能 Tab |
| `SellModal` | 上架 | 共享仓 |
| `EquipmentDetailModal` | 装备详情 | 装备 Tab |
| `BuyModal` | 购买 | 市场 |

---

## §1 · 前端缓存架构（核心）

> 设计目标：**让数据"够用就行"**——4 级缓存，避免每次切 tab 都重新 invoke。

### §1.1 4 级缓存层级

```
Level 0 · Static Build-in          (code / constant / i18n / runewordMeta.json)
Level 1 · Module Singleton         (in-memory, 全 app 单例, React hook)
Level 2 · Cross-tab Pagination     (localStorage, 跨刷新存活)
Level 3 · Server-of-record         (Tauri/Rust, 磁盘 .d2s/.d2i/SQLite)
```

| 级 | 内容 | 写入 | 失效 |
|---|------|------|------|
| L0 | `ITEM_NAMES`, `runewordMeta`, `RUNE_NAMES` | 编入 bundle | n/a |
| L1 | 角色类缓存、stash 缓存、符文之语结果缓存 | React state + module Map | on `char:stage1/3` / `forceRefresh` / 切角色 |
| L2 | `localStorage` `d2r-char-full-<name>`, `d2r-char-stash-<name>`, `runeword-context-cache` 等 | 自动 on `dataLoaded` | 比较 `imported_at`，> 60s 自动 forceRefresh |
| L3 | Rust 读 .d2s / .d2i / SQLite（写回） | IPC `tauriInvoke` | 用户操作 |

### §1.2 Module Singleton 缓存层（新增 `web/src/cache/`）

> 所有 4 个一级菜单的"档案级"数据走统一 singleton，避免 props drilling + tab 间状态丢失。

```
web/src/cache/
├── ClientCache.ts       — 总体 type-safe LRU + TTL Map
├── characters.ts        — CharacterStore
├── stash.ts             — StashStore
├── warehouse.ts         — WarehouseStore
├── runewords.ts         — RuneWordStore
├── runes.ts             — 用户符文 holdings 跨页共享
├── useCache.ts          — React hook 订阅 + re-render
└── events.ts            — window CustomEvent 总线（平衡 + 编辑通知）
```

#### §1.2.1 `ClientCache<T>` 设计

```ts
interface CacheEntry<T> {
  data: T
  imported_at: number      // ms timestamp
  source: 'local' | 'ipc' | 'hybrid'
}

class ClientCache<T> {
  private map = new Map<string, CacheEntry<T>>()
  private listeners = new Set<() => void>()

  get(key: string, opts?: { maxAgeMs?: number, force?: boolean }): T | null
  set(key: string, data: T, source?: 'local' | 'ipc' | 'hybrid'): void
  invalidate(pattern: string | RegExp): void
  subscribe(fn: () => void): () => void
}
```

**统一约定**：
- `key = "<domain>:<name>"`，如 `character:EchoingStrike`、`stash:full`、`runeword:all-33`
- 默认 `maxAgeMs = 60_000`，可在 hook 调用处覆盖
- `invalidate` 支持 `startsWith('character:')` 或 `/^stash:/` 模式
- `subscribe` 触发 `useState({})` 重新渲染

#### §1.2.2 各 Store 的接口形态

```ts
// characters.ts
export const characterStore = {
  /** 角色列表（轻量），key 不带 name */
  listKey: 'characters:list',
  getList(): Promise<CharacterBriefInfo[]>

  /** 完整 CharacterInfo，key 带 name */
  fullKey: (name: string) => `character:${name}`,
  getFull(name: string, opts?): Promise<CharacterInfo>

  /** 切换:清掉所有 character:* 但保留 d2r-last-character */
  onSwitchCharacter(from: string, to: string): void
}
```

```ts
// stash.ts
export const stashStore = {
  fullKey: (name: string) => `stash:${name}`,
  get(name: string, opts?): Promise<StashResult>

  /** 跨页共享缓存 — 必须等于 d2r-char-stash-<name> 写入逻辑 */
  bindLocalStorage(name: string): void

  /** 写成功回调（list_item / warehouse_deposit 后台调用）*/
  onWriteSuccess(name: string): void
}
```

```ts
// warehouse.ts
export const warehouseStore = {
  searchKey: (filters) => `warehouse:${JSON.stringify(filters)}`,
  search(filters, opts?): Promise<WarehouseItem[]>

  /** 写成功回调 */
  onWriteSuccess(): void
}
```

```ts
// runewords.ts
export const runeWordStore = {
  resultsKey: (ownedRunes: string[]) => `runeword:${ownedRunes.sort().join(',')}`,
  compute(ownedRunes: string[]): Promise<any[]>

  contextKey: 'runeword:context',
  getContext(): Promise<RunewordContextCache>
}
```

#### §1.2.3 React 订阅 hook

```ts
// useCache.ts
export function useCached<T>(
  loader: () => Promise<T>,
  opts: {
    key: string
    maxAgeMs?: number
    onInvalidate?: () => void
  },
): {
  data: T | null
  loading: boolean
  error: Error | null
  refresh: (force?: boolean) => Promise<void>
  isStale: boolean         // data 超过 maxAgeMs 但未失效
}
```

**示例**（Characters.tsx 改写）：

```tsx
const { data: character, loading, refresh, isStale } = useCached(
  () => tauriInvoke('load_character_background', { path }) as Promise<CharacterInfo>,
  { key: `character:${selectedChar}`, maxAgeMs: 60_000 },
)
```

> 注意：`load_character_background` 是异步三段事件，不适合走 `useCached` 的 Promise 形态。**它走自己独有的 event bus**（`char:stage1/3`），仅 L2 持久化复用 `d2r-char-full-<name>` 逻辑。

### §1.3 失效事件总线

| Triggering Action | 失效 Key | 实现 |
|------|------|------|
| 进入 / 切回 角色 Tab | `character:<name>` | useEffect 监听 |
| 切回共享仓库 Tab | `stash:<name>` | useEffect 监听 |
| d2s 写入成功 | `character:<name>`, `character-class:<name>` | 命令返回 ok 后分发 `cache:invalidate` |
| d2i 写入成功（取回） | `stash:<name>`, `character:<name>`（含仓库 tab） | 同上 |
| `extract_character_equipment` | `warehouse:`, `character:` | 同上 |
| 角色切换 | `runeword:context`（保留 ownedRunes, 但底材失效） | `clearRunewordContextCache()` 后分发 |
| 锈币/上架等市场操作 | `balance` | `window.dispatchEvent('balance-update')` 已有 |
| 用户点 "立即刷新" 按钮 | 当前页 cache | 主动 `invalidate` |

**实现**：复用 `window.dispatchEvent(new CustomEvent('cache:invalidate', { detail: pattern }))` 模式（已有 `balance-update` 雏形在 `useToast.ts:25`）。

### §1.4 缓存键总账（含命名/TTL 规范）

> 与 `main-menu-requirements.md` 附录 B 对齐，**新增"module singleton"层单独条目**：

| Key | Owner Store | TTL | 来源 | 失效事件 |
|-----|------------|-----|------|---------|
| `characters:list` | characterStore | 30s | `list_characters_brief` | `cache:invalidate:characters:` |
| `character:<name>` | characterStore | 60s | `char:stage3` event payload | `cache:invalidate:character:<name>` |
| `character-class:<name>` | characterStore | 60s | 同上 | 同上 |
| `stash:<name>` | stashStore | 60s | `read_stash` | `cache:invalidate:stash:<name>` |
| `warehouse:<json>` | warehouseStore | 30s | `warehouse_search` | `cache:invalidate:warehouse:` |
| `warehouse:item:<id>` | warehouseStore | 120s | `warehouse_search` 命中 | `cache:invalidate:warehouse:` |
| `runeword:<sorted-runes>` | runeWordStore | 10min | `find_runewords` | `cache:invalidate:runeword:` |
| `runeword:context` | runeWordStore | 60s | `get_runeword_context` | `cache:invalidate:runeword:context` |
| `user:runes:<name>` | runesStore | 60s | L2 `d2r-char-full-<name>` 派生 | `cache:invalidate:character:<name>` |

**L2 → L1 同步**：`characters.tsx` 启动时读 `d2r-char-full-<name>` 立即填充 L1（fallback），后续 IPC 刷新时同时 `set` L1 + 写 L2。

### §1.5 跨页状态机的统一 hook

> 现在 4 个页面各自维护 localStorage 副本 → 缓存策略层叠在 Store 之上，**最关键的是** `useCached` 取代散落的 `useEffect + fetch`。

**对比示意**：

```tsx
// 现在 (v1) — 散在每个页面
const [items, setItems] = useState([])
useEffect(() => { tauriInvoke('warehouse_search', f).then(setItems) }, [f])

// 改后 (v2) — 走 store + hook
const { data: items, isStale, refresh } = useCached(
  () => warehouseStore.search(filters),
  { key: `warehouse:${JSON.stringify(filters)}`, maxAgeMs: 30_000 },
)
```

> 收益：组件 unmount 不丢数据（缓存仍在 L1），再次 mount 立即返回；切换 store 不重发请求；过期数据自带 `isStale` 提示。

---

## §2 · 跨菜单数据流图（基于 §1 cache 层）

```
            ┌──────────────────────────────────────────────────┐
            │   L3 (Server-of-record: Rust / disk / SQLite)   │
            │   .d2s / .d2i / SQLite / 价格参考                 │
            └──────────────────────────────────────────────────┘
                       ▲                                │
                       │ IPC                            │
              ┌────────┴────────────────┐               │
              │                         │               │
              │   Tauri command bridge  │               │
              │   (tauriInvoke / events)│               │
              │                         ▼               │
              │              ┌──────────────────┐       │
              │              │ Rust / src-tauri/ │       │
              │              │ commands/* /     │◀────write-back
              │              │ protocol/d2s/d2i │       │
              │              └──────────────────┘       │
              │                         │               │
              └────────┬────────────────┘               │
                       ▼                                │
            ┌──────────────────────────────────────────────────┐
            │   L2 (localStorage 持久层)                       │
            │   d2r-char-full-<n> / d2r-char-stash-<n>        │
            │   runeword-context-cache / ...                  │
            └──────────────────────────────────────────────────┘
                       │                                    │
                       ▼                                    │
            ┌──────────────────────────────────────────────────┐
            │   L1 (Module Singleton Store)                    │
            │   characterStore / stashStore / warehouseStore  │
            │   runeWordStore / runesStore                     │
            │                                                    │
            │   持有的副本: CharacterInfo / StashResult         │
            │              WarehouseItem[] / RunewordResult[]   │
            └──────────────────────────────────────────────────┘
                       │
                       ▼
            ┌──────────────────────────────────────────────────┐
            │   L0 (React state per component)                │
            │   useState + useCached 订阅                      │
            └──────────────────────────────────────────────────┘
                       │
                       ▼
            ┌──────────────────────────────────────────────────┐
            │   UI Components                                   │
            │   ── Characters.tsx / Warehouse.tsx              │
            │   ── Inventory.tsx / RunewordCalc.tsx            │
            │   ── CharacterPanel / EquipmentPanel / ...      │
            └──────────────────────────────────────────────────┘
```

**关键不变量**：
1. **L1 是 single source of UI**：组件只从 `useCached` 读，不再自己 `tauriInvoke`
2. **L2 是冷启动 fallback**：L1 命中直接返回；L1 miss 但 L2 在，hydrate 后再 IPC；L2 miss 必须 IPC
3. **写命令 return `ok=true` 必 invalidate**：写后一致
4. **过期数据有 `isStale` 提示**：UI 可显示 "T+45s 数据" 之类的角标

### §2.1 例：切到"仓库"页面时的数据旅程

```
用户点 /warehouse
↓
Warehouse.tsx mount → useCached 启动
↓
characterStore.search(filters)
  ├─ L1 命中 → 返回（instant）
  ├─ L1 miss, L2 命中 → hydrate + 后台 IPC 刷新
  └─ 双 miss → IPC → set L1 + L2
↓
返回 React UI 渲染
```

### §2.2 例：用户在 共享仓 "上架" 后的数据旅程

```
SellModal.onListed()
↓ tauriInvoke('list_item', ...)
↓
后端 ok → window.dispatchEvent('cache:invalidate', { detail: 'stash:<name>' })
↓
charactersStore / stashStore 收到 → invalidate matching keys
↓
React 订阅者 re-render：共享仓列表减少 + balance 上新
（无需轮询 / refetch）
```

---

## §3 · 4 个主菜单 UI 改造规格（按 P0 排）

### §3.1 角色 (`/characters`) — 新增组件 4 件 + 改造 1 处

#### §3.1.1 角色选择卡（CharacterPicker）— **新增**

> 解决"刷新后旧存档找不到 / 没有按职业过滤 / 没有角色卡片选中视觉"问题。

```
┌─────────────────────────────────────────────────────┐
│ Hero (现有)                                          │
│   "角色" kicker + "角色" title + lede                │
│   [+ 刷新] [+ 存入仓库]   角色总数 chip              │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────┐
│ D2EmuCard kicker="角色" actions=[filter chips]      │
│                                                     │
│  ┌──────┬──────┬──────┬──────┐                       │
│  │ Amzn │ Sorc │ Necr │ Pala │   (8 职业点阵 chip)   │
│  └──────┴──────┴──────┴──────┘                       │
│  [全部/资料片/经典] [生/死] [全部/Hc/Nec]            │
│  ───────────────────────────────────────────         │
│  ┌────────────────────────────────────────────┐      │
│  │ ⚒ Amazon   "EchoingStrike"   Lv.92 资料片 │ 选中│  │
│  ├────────────────────────────────────────────┤      │
│  │ ⚒ Necromancer "happy_librarian"  Lv.84 死档│      │
│  ├────────────────────────────────────────────┤      │
│  │ ⚒ Warlock    "术士_新手"   Lv.31           │ 改动│  │
│  └────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────┘
```

**规格**：
- 用现有 `d2emu-tab` 风格 chip（已存在 `TabBar.tsx` `variant='sub'`）
- 列表行复用 `CharacterPanel.tsx:166 CharListItem`，但把 `<div role="button">` 换成 `<button>` 配合 d2emu 行样式
- 文件 hash 变更 chip 用 `d2emu-tag-active` + 金色
- 文件路径：新建 `web/src/components/CharacterPicker.tsx`
- **缓存**：`characterStore.list` (L1) + `d2r-character-names` (L2)

#### §3.1.2 切换主副手（HandSwapButton） — **新增按钮**

```
        ┌────────────────────────┐
        │ ⚒ 主武 [并排] 盾       │
        │   ↕ 一键切换           │  ← d2emu-btn-sm
        │ ⚒ 副武  盾      副手   │
        └────────────────────────┘
```

**规格**：
- `EquipmentPanel` 下方加一条 `.d2emu-row.justify-center`，按钮 `d2emu-btn d2emu-btn-ghost`
- 点击弹 `DConfirmModal`："主手 ↔ 副手 互换？此操作会写回 d2s"
- 写成功 → invalidate `character:<name>`
- 锚点：`EquipmentPanel.tsx:160-165` 加 `<HandSwapButton character={characterName} onSwap={...}/>` 占位

#### §3.1.3 一键替换（EquipReplaceMenu） — **新增菜单**

```
  [装备 X 槽 ▾]
       ├─ 从共享仓库选择…
       ├─ 从扩展仓库选择…
       └─ 卸下
```

**规格**：
- 行内 `<select>` 或 dropdown menu，触发弹二级 modal：`QuickReplaceModal`
- 该 modal 三 tab：装备 Tab / 共享仓 / 扩展仓 → 选中 → 调 `swap_equipment` 命令（v3 新增）
- 缓存：`stash:<name>` / `warehouse:filtered-equip`

#### §3.1.4 装备技能加层聚合（EquipmentBonusTome） — **新增组件**

```
┌─ 装备 · 加层 ──────────────┐
│ Skill Tabs                  │
│  Tab #0 (Amazon Bow)    +3  │
│  Tab #2 (Amazon Spear)  +1  │
│ Chance to Cast              │
│  Strafe (skill 24)   12% Lv5│
│ Charges                     │
│  Strafe           3/15  Lv10│
└─────────────────────────────┘
```

**规格**：
- 新建 `web/src/components/EquipmentBonusTome.tsx`
- 走 `classifySkillBonuses(equipmentSkillBonuses)` 已经存在（参见 `SkillDetailsPanel.tsx:61-88`）
- 标签 `<h5>` 沿用 `d2emu-section-title-arcane` 装饰

#### §3.1.5 角色 dead 状态横幅（DeadBanner）— **新增**

> 性格信息：`is_dead` 已被 `CharacterBriefInfo` 类型定义但 UI 没暴露。

```
┌─────────────────────────────────────────────────────────┐
│ ⚠ 你的角色 "EchoingStrike" 已死亡。所有操作只读。          │
│   [进入归档视图]  [保留备份]   09:34:23 死亡                │
└─────────────────────────────────────────────────────────┘
```

**规格**：
- 沿用 `index.css::.d2emu-lock-banner`（已存在）
- 锚点：在 `CharacterPanel.tsx:367` 顶部增加 `<DeadBanner character={character} />`

#### §3.1.6 8 大 Tab 排序调整（UX 改良）

> 现在顺序是 **装备/背包/仓库/技能/小站/任务/奖励/佣兵**。但日常高频：**装备 / 仓库 / 技能 / 背包 / 佣兵 / 小站 / 任务 / 奖励**。

**规格**：
- `TABS` 数组交换：`['overview', 'warehouse', 'skills', 'storage', 'merc', 'waypoints', 'quests', 'rewards']`
- 或保留原顺序但加 `pin` 标记用户最常用的到第一位
- 用 sub-tab pill 风格：`d2emu-tab-ribbon` (gothic ribbon) — 已存在

---

### §3.2 仓库 (`/warehouse`) — 主要改造：取回弹选位 / 错误 toast / 图标

#### §3.2.1 取回选位弹窗（WithdrawPositionModal）— **新增 modal**

```
┌─────── 取出 [物品名 × N] ───────┐
│ 目标共享仓库页：                  │
│   ◉ 第 1 页 · 高级页·堆叠  (3 个空位)
│   ○ 第 2 页 · 装备页            (空)
│                                  │
│ 目标坐标：                       │
│   X [8]  Y [4]  [+] [-]          │
│   缩略图 → 显示 8,4 的空位     │
│                                  │
│ ⚠ 取出后扩展仓库减 N 件          │
│   d2i 文件将立即落盘 + 备份       │
│                                  │
│  [取回]      [取消]              │
└──────────────────────────────────┘
```

**规格**：
- 新建 `web/src/components/WithdrawPositionModal.tsx`
- 调用 `get_stash_empty_slots(stashPath, pageIndex)` → 返回 `{ page: index, x, y, w, h, itemWidth, itemHeight }[]`
- 用户选位 → 写命令 `warehouse_withdraw({ itemId, stashPath, pageIndex, positionX, positionY })`
- 失败 toast + 缓存不更新

#### §3.2.2 错误 toast 升级

```ts
// Warehouse.tsx:85 当前
} catch (e: unknown) {
  showToast(e instanceof Error ? e.message : '加载仓库失败', 'warning')
}

// 改后 (v2)
} catch (e: unknown) {
  const msg = e instanceof Error ? e.message : '加载仓库失败'
  showToast(msg, 'error', { position: 'top' })  // warning → error
}
```

#### §3.2.3 仓库行项图标

```tsx
// Warehouse.tsx:312 当前
<img src="data:image/svg+xml,..." alt="" ... />
// fallback 始终是 '?'

// 改后（v2）
<img src={resolveItemIcon(item)} alt={item.item_name} className="w-5 h-5 object-contain"
  onError={handleImgError} />
```

`resolveItemIcon` 已是 `utils/itemImages.ts` 函数，`WarehouseItem` 需要扩字段：
```ts
// types.ts
export interface WarehouseItem {
  // ... 现有字段
  icon?: string   // 新增
  name_en?: string  // 新增
}
```

#### §3.2.4 收藏页 datalist

```tsx
// Warehouse.tsx:349 当前
<input value={warehouseDraft.page_name}
  onChange={...}
  placeholder="收藏页名称" />

// 改后（v2）
<input list="datalist-page-name" ... />
<datalist id="datalist-page-name">
  {Array.from(new Set(warehouse.map(w => w.page_name))).map(p => (
    <option key={p} value={p} />
  ))}
</datalist>
```

#### §3.2.5 视觉规范

- 行低耐久用 `index.css::is-low-dur` 红底（已存在）+ `.dur-pulse` 1.6s 动画（已存在）
- 行 hover 浮起沿用 `.d2emu-card-hoverable:hover { transform: translateY(-2px) }`
- 整理 Modal 沿用 `.d2emu-modal` (max-w 480px)

---

### §3.3 共享仓库 (`/stash`) — 主要改造：批量 / 过滤 / 移动端

#### §3.3.1 批量选择 + 批量动作条

```
┌──────────────────────────────────────────────┐
│ 🗂 共享仓库 / 第 2 页                        │
│   [全选] [反选]                              │
│   ─────────────────────────────────────     │
│   ✔ Stacked Rune × 5   unique    上架        │ ← 行项前 checkbox
│   ✔ Topaz × 12          normal   入仓        │
│   ─────────────────────────────────────     │
│ ⚡ [批量上架 2 件]  [批量入仓]  [取消选择]   │ ← 固定底部条
└──────────────────────────────────────────────┘
```

**规格**：
- 行项 `<article>` 前加 `<input type="checkbox" className="d2emu-checkbox">` (新 class)
- 选中数 > 0 显示 `.d2emu-batch-action-bar`，fixed bottom
- 批量上架：弹 `BatchSellModal`（多件聚合数据）
- 批量入仓：弹 `BatchDepositModal`（带 1 个 confirm 汇总）
- 缓存：成功后 invalidate `stash:<name>` + `warehouse:`
- 触屏 `<800px` 时 checkbox 区放大到 36×36
- Class CSS：在 `index.css` 加：
```css
.d2emu-checkbox { width: 16px; height: 16px; accent-color: var(--color-d2emu-gold); cursor: pointer; }
.d2emu-batch-action-bar { position: sticky; bottom: 12px; ... }
```

#### §3.3.2 类目过滤器二级筛选

```
[全部] [符文] [宝石] [...] [装备*]
  └─ 装备激活时展开二级:
     [品质▾] [部位▾] [底材▾]
```

**规格**：
- 复用现有 `CATEGORY_TABS` 增加 `expanded: boolean` 状态
- 二级筛选 row 沿用 `.d2emu-subtab` 风格
- 筛选条件走 `useCached(() => warehouseStore.search(filters), ...)` 模式

#### §3.3.3 触屏固定底栏

```
┌──────────────────────┐
│ 物品卡 (紧凑)         │
│ ┌────┐ ┌────┐       │
│ │ R  │ │ G  │       │
│ └────┘ └────┘       │
├──────────────────────┤
│ [上架]  [入仓]  ← 触屏固定
└──────────────────────┘
```

**规格**：
- `@media (max-width: 800px) { .d2emu-card-bulk-actions { position: fixed; bottom: 0; ... } }`
- 沿用 `d2emu-btn d2emu-btn-sm`

#### §3.3.4 SellModal 单价上限提示

```tsx
// SellModal.tsx 当前
<input id="sell-price" type="number" min={1} value={price} ... />
<button onClick={handlePriceSuggestion} ...>参考建议售价</button>

// 改后（v2）
<input id="sell-price" type="number" min={1} max={suggestion?.max_price ?? undefined} value={price} ... />
{price > (suggestion?.max_price ?? Infinity) * 1.5 && (
  <small className="d2emu-warning-chip">超出建议 1.5x</small>
)}
```

#### §3.3.5 视觉规范

- 类目 chip 沿用 `d2emu-tag` + `.d2emu-tag-active`
- 物品卡沿用 `d2emu-card-quiet`，hover 沿用 `.d2emu-card-hover-item`
- 品质色来自现有 `QualityLegend.tsx`
- 数量徽标沿用现有 `×{item.quantity}` 绝对定位写法

---

### §3.4 符文计算 (`/runeword`) — 主要改造：符文消耗统计 / 底材 quality / 位置追溯

#### §3.4.1 符文消耗统计条（StatBanner）— **新增组件**

```
┌───────── 你的 [N] 个符文 ────────────────┐
│ 可制作: 18 个符文之语                    │
│ 练级覆盖: 开荒 10 / 后期 8               │
│ 缺 1 个: Spirit (需要 r23 + r24)         │  ← 玩家声音最高
│ 缺 1 个: Enigma (需要 31 + 32)  ⚠ 价格高 │
└──────────────────────────────────────────┘
```

**规格**：
- 新建 `web/src/components/RunewordStatBanner.tsx`
- 数据：从 `selected` + `filteredResults` 派生
  - `covered = filteredResults.filter(rw => rw.runes.every(r => selected.has(r))).length`
  - `oneMissing = filteredResults.filter(rw => rw.runes.filter(r => selected.has(r)).length === rw.runes.length - 1).map(rw => ({ rw, missing: rw.runes.find(r => !selected.has(r))! }))`
- 用 `d2emu-card` + `.d2emu-lede` 风格

#### §3.4.2 底材 quality 过滤（BaseQualityFilter）— **新增 chip 组**

```
底材 [任意 ▾]
   └─ 品质 [任意 / 白板 / 优秀 / 无形]
```

**规格**：
- 新建 `web/src/components/BaseQualityFilter.tsx`
- 复用现有 `BaseTypeFilter` 风格（sub-tab chip）
- 底材品质数据从 `runewordMeta.json` 读（新加字段）
- 状态：`baseQualityFilter: Set<'normal'|'superior'|'ethereal'>`

#### §3.4.3 符文位置追溯（RuneLocationDrawer）— **新增组件（P1）**

```
       ┌─── [r31: Jah] 位置 ──────────┐
       │ ✒ 角色 EchoingStrike 背包    │
       │    4×4 网格 (5,1)            │
       │ ✒ 共享仓库 个人页            │
       │    16×16 网格 (10,7)        │
       │ ✒ "武器 [Chaos]" 的 socketed │
       │    [物品: 死亡呼吸]  [x]     │
       └─────────────────────────────┘
```

**规格**：
- 触发：点击未拥有的 rune 按钮
- 数据：从 L1 `characterStore` + `stashStore` 派生，加上装备 socketed 扫描
- 跳转：点击条目 → 跳转对应页面（`useNavigate`）

#### §3.4.4 Spirit 例外 + stars≥4 浮标修正

```tsx
// 现在 (v1)
{isGold && ...}   // 纯 stars ≥ 4 浮标

// 改后 (v2)
{(isGold || isBlue || isGreen) && meta?.stars && meta.stars >= 4 && (...)}
+ // Spirit 例外徽标
{meta?.rec_best?.includes('Spirit') && (
  <small className="d2emu-recommend-chip">★ 实际推荐</small>
)}
```

> 评级：`recommend_rank` 字段需要在 `runewordMeta.json` 注入。

#### §3.4.5 多语种 fallback

```ts
// RunewordCalc.tsx:742 改后
let affixes: string[] | null
if (language === 'zhCN' || language === 'zhTW') {
  affixes = meta?.affixes_zh?.map(translateAffixZh)
}
if (!affixes) affixes = meta?.affixes ?? []   // fallback 到英文
```

#### §3.4.6 视觉规范

- 4 色背景（🟡/🔵/🟢/⚫）保持现有
- tier 排序修正：sort 阶段数据从 derived 计算而非依赖 stars 排序
- 卡片宽度 300px 间距 12px，auto-fill 自适应

---

## §4 · 通用 UI 风格复用（避免重建）

| 场景 | 复用 token / class | 位置 |
|------|------------------|------|
| 卡片头部 | `.d2emu-card` + `.d2emu-kicker` + `.d2emu-title` + `.d2emu-lede` | `D2EmuCard.tsx` |
| 二次确认 | `.d2emu-modal` + `DConfirmModal` | `DConfirmModal.tsx` |
| 顶部 toast | `.d2emu-alert` + `Toast` 组件 | `Toast.tsx` |
| KPI | `.d2emu-kpi` 含金边 + 烫金大数字 | `KpiRow.tsx` |
| Loading | `.d2emu-loading` + 旋转符文 | `D2EmuLoading.tsx` |
| Empty | `EmptyState` + icon + lede + hint | `EmptyState.tsx` |
| 装备人型 | `.d2emu-equipment-grid` + `.d2emu-item-slot-arcane` | `EquipmentPanel.tsx` |
| Tab 切换 | `.d2emu-tabbar-ribbon` (gothic ribbon) | `index.css:1517` |
| 装备图标 | `.d2emu-wax-seal` 蜡封 5 色 | `QualityLegend.tsx` |
| 角色肖像 | `.d2emu-portrait-arcane` + corner accents | `index.css:1244` |
| Stat block | `.d2emu-statblock-tome` 手稿体 + `❦` 装饰 | `index.css:1427` |
| Lock 横幅 | `.d2emu-lock-banner` | `index.css:1375` |
| Drop zone | `.d2emu-drop` + pulse | `index.css:1273` |

---

## §5 · 响应式断点规范（沿用既有，避免新建）

```
base          < 700px   → KpiRow 1 列 / modal 满屏 / equipment-cell 56-80px
              800-899   → KpiRow 2 列 / TabBar wrapped
              900-1099  → equipment-cell 80-110px / drop into single column
              1100-1799 → KpiRow 4 列 / RunewordCalc 双列 / cell 96-128px
default       > 1100px  → html font-size 110%
              > 1800px  → html font-size 115%
```

**新增组件必须遵守**：mobile-first，根 token 用现有断点（不引入新断点）。

---

## §6 · 可访问性规范

| 字段 | 要求 |
|------|------|
| `aria-label` | 全部图标按钮必须有 `aria-label`（已部分实现） |
| `role="alert"` | toast 错误/警告 / DConfirmModal 错误文案 |
| `aria-live` | 已实现于 `SellModal` validation |
| 键盘焦点 | Modal 用 `useFocusTrap`（已实现） |
| Tab 顺序 | TabBar 已实现 ArrowLeft/Right/Home/End |
| 对比度 | >= 4.5:1（已校验）|
| Reduced motion | `@media (prefers-reduced-motion: reduce) { ... }` 已部分实现，加全面 |

**新增组件必备**：

```tsx
<input aria-label="..." aria-invalid={!!error} aria-describedby="error-id" />
<button aria-busy={loading} aria-live={loading ? 'polite' : undefined}>...</button>
```

---

## §7 · 实施清单（按周 sprint 排）

### Week 1 · 缓存层基础设施
- [ ] 新建 `web/src/cache/ClientCache.ts` + 5 个 store
- [ ] 新建 `web/src/cache/useCache.ts`
- [ ] 迁移 `Characters.tsx` `useCached` 化
- [ ] 迁移 `Warehouse.tsx` `useCached` 化
- [ ] 迁移 `Inventory.tsx` `useCached` 化
- [ ] 迁移 `RunewordCalc.tsx` `useCached` 化
- [ ] 配 `invalidate` 总线 + 7 个写命令 success hook
- [ ] 测试：`store-cache.spec.ts` 覆盖 5 个 store

### Week 2 · 4 个菜单 P0 改造
- [ ] 角色 · CharacterPicker 组件 + 8 tab reorder
- [ ] 角色 · DeadBanner 组件 + is_dead 联动
- [ ] 角色 · EquipmentBonusTome 组件
- [ ] 角色 · HandSwapButton + EquipReplaceMenu（v3 swap_equipment 命令不在本 sprint 范围）
- [ ] 仓库 · WithdrawPositionModal
- [ ] 仓库 · 错误 toast 升级
- [ ] 仓库 · 行项图标 resolveItemIcon
- [ ] 共享仓 · 批量选择 + 批量动作条
- [ ] 共享仓 · 类目二级筛选（装备）
- [ ] 共享仓 · 触屏固定栏
- [ ] 符文 · RunewordStatBanner
- [ ] 符文 · BaseQualityFilter + runewordMeta.json 扩字段
- [ ] 符文 · 位置追溯 (P1, 视时间)

### Week 3 · 跨菜单一致性 + 测试
- [ ] 抽 11 个 batch action 类样式到 `index.css`
- [ ] 抽 `Drawer` / `Modal` 通用属性
- [ ] vitest 测试覆盖率 > 70%
- [ ] playwright e2e: 4 主菜单常用路径
- [ ] a11y 审计（axe-core）
- [ ] 性能：d2i 5000 件虚拟滚动（如有余量）

---

## §8 · 风险与已识别的卡点

| 风险 | 缓解 |
|------|------|
| `useCached` 与现有 React Query 等冲突 | 现有项目无 React Query，约定只引入 `useCached` 一种缓存 hook |
| `warehouse_swap_equipment` 命令尚未存在 | Week 2 列入"v3 范围外"，按钮 disabled + toast "即将推出" |
| ECS batch 调用可能阻塞 UI | 用 `Promise.all` 但每个 `wrap` 自己 catch |
| D2R Warlock 全身像资源 | 走 §0.1 三档路径（用户截图 + SVG fallback） |
| localStorage 跨页 stale | TTL + invalidate 事件 + "立即刷新" 按钮 |
| 测试：仓库搜索过滤 5 维组合 | vitest snapshot + parameterized cases |
| `useCached` 错误重试次数 | 默认 1 次，UI 显 "重试" 按钮 |

---

## §9 · 引用文件清单（实现时按需 navigate）

### 新增
- `web/src/cache/ClientCache.ts`
- `web/src/cache/characters.ts`
- `web/src/cache/stash.ts`
- `web/src/cache/warehouse.ts`
- `web/src/cache/runewords.ts`
- `web/src/cache/runes.ts`
- `web/src/cache/useCache.ts`
- `web/src/cache/events.ts`
- `web/src/components/CharacterPicker.tsx`
- `web/src/components/WithdrawPositionModal.tsx`
- `web/src/components/RunewordStatBanner.tsx`
- `web/src/components/BaseQualityFilter.tsx`
- `web/src/components/RuneLocationDrawer.tsx`
- `web/src/components/EquipmentBonusTome.tsx`
- `web/src/components/DeadBanner.tsx`
- `web/src/components/HandSwapButton.tsx`
- `web/src/components/EquipReplaceMenu.tsx`
- `web/src/components/BatchSellModal.tsx`
- `web/src/components/BatchDepositModal.tsx`

### 修改
- `web/src/index.css`（新增 `.d2emu-checkbox`, `.d2emu-batch-action-bar`, `.d2emu-drawer`, etc.）
- `web/src/pages/Characters.tsx`（改 useCached）
- `web/src/pages/Warehouse.tsx`（改 useCached + 取回 modal + 图标 + datalist）
- `web/src/pages/Inventory.tsx`（改 useCached + 批量）
- `web/src/pages/RunewordCalc.tsx`（改 useCached + StatBanner + QualityFilter）
- `web/src/components/CharacterPanel.tsx`（DeadBanner + EquipmentBonusTome + tab reorder）
- `web/src/components/EquipmentPanel.tsx`（HandSwapButton + EquipReplaceMenu 钩子）
- `web/src/components/SellModal.tsx`（max_price 提示）
- `web/src/types.ts`（WarehouseItem 加 `icon`, `name_en`；CharacterInfo 透传 `is_dead`）
- `web/src/data/runewordMeta.json`（加 `base_quality`, `recommend_rank`）

### 测试新增
- `web/tests/unit/cache/ClientCache.spec.ts`
- `web/tests/unit/cache/characters.spec.ts`
- `web/tests/unit/cache/stash.spec.ts`
- `web/tests/unit/cache/warehouse.spec.ts`
- `web/tests/unit/cache/runewords.spec.ts`
- `web/tests/unit/cache/useCache.spec.tsx`
- `web/tests/components/CharacterPicker.spec.tsx`
- `web/tests/components/WithdrawPositionModal.spec.tsx`
- `web/tests/components/RunewordStatBanner.spec.tsx`
- `web/tests/components/BaseQualityFilter.spec.tsx`
- `web/tests/e2e/characters-tab-flow.spec.ts`
- `web/tests/e2e/warehouse-withdraw-flow.spec.ts`
- `web/tests/e2e/stash-batch-flow.spec.ts`
- `web/tests/e2e/runeword-consumption-flow.spec.ts`

---

## 版本与变更日志

| 版本 | 日期 | 变更 |
|------|------|------|
| v1 | 2026-07-20 | 初版：4 级缓存架构 + 4 主菜单 UI 改造规格 + 实施 3 周 sprint |
