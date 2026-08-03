# 角色页面设计文档(PM + UI + UX 三视角)

> 范围: `web/src/components/CharacterPanel.tsx` 及其 6 个 tab + 后端协议
> 基础: 已完成 d2emu.com/hero 抓取与设计系统对齐(`d2emu-hero-design-system.md`)
> 协作: PM 视角(为什么/做多少) + UI 视角(长什么样) + UX 视角(怎么用)
> 输出日期: 2026-07-08

---

## 0. 设计方向(Design Direction,一句话定调)

**"暗黑奇幻档案柜"** — 让角色页看起来像打开**恶魔猎人事务所的一本皮革烫金档案**:
角色是档案的主人,装备/背包/技能是档案夹里的实物照片、人物素描、符文拓印。
**不是**通用的 SaaS dashboard,**也不是**花花绿绿的游戏 UI。
**核心氛围**: 暗金底 + 烫金边 + 血血红行动色 + 羊皮纸纹理 + 旧打字机字。

> 拒做:紫渐变 / 装饰大球 / 巨型 hero copy / 卡片套卡片 / 单一 hue 主导。

---

## 1. PM 视角 — 功能分析与优化计划

### 1.1 目标用户(用户画像)

| Persona | 需求 | 优先级 |
|---|---|---|
| **A. 单机玩家长** (90%) | 看自己装备、对比行情、规划装备升级 | P0 |
| **B. 仙道 mod 玩家** (8%) | xieedi/happy_manman 魔改 layout 可读 | P0 (已是) |
| **C. 装备商人** (2%) | 批量查套装/暗金、回写修改 | P1 (后置) |

> **A 类用户每天打开 1-3 次**,主要看装备和技能。需要**扫读快**,不是看广告。
> **关键任务**: 切角色 → 看装备 → 看技能 → 看任务进度。

### 1.2 现状评估(Gap vs d2emu vs 用户需求)

| 功能 | 现状 | d2emu | A 用户需求 | Gap 等级 |
|---|---|---|---|---|
| 12 槽装备展示 | ✅ EquipmentPanel 4×5 | ✅ | ✅ | 无 |
| 装备 hover tooltip | ✅ ItemTooltip | ✅ | ✅ | 无 |
| **装备点击 → 详情面板** | ✅ **EquipmentDetailModal** | ✅ ItemEditModal(只读≈细节) | ✅ 看详情 | **M4 完成** |
| 背包 10×4 grid | ✅ BackpackGrid | ✅ | ✅ | 无 |
| 腰带 1-4 行 | ✅ BeltRow | ✅ | ✅ | 无 |
| Horadric Cube | ⚠️ 3×4 空 UI | ✅ 可填 | ❌ 暂不要 | P3 |
| **技能树 8 职业** | ✅ 8 职业(含 Warlock 3 树) | ✅ 8 职业 | ✅ | **无** |
| 属性编辑(力敏体精) | ❌ 只读 | ✅ 5 列(基/装/任/总) | ❌ 暂不要 | P3 |
| 抗性/breakpoint | ❌ Stats tab 不存在 | ✅ | ✅(看 build) | **P1** |
| **小站 39 + ▼ 折叠** | ✅ **3 难度可折叠** | ✅ ▼N/N/H | ✅ | **M1 完成** |
| **任务 27 + 图标 + 折叠** | ✅ **+sprite + ▼ 折叠** | ✅ ☑+图标+折叠 | ✅ | **M1+M3 完成** |
| **任务 stat/skill pt** | ✅ **QuestBonusSummary** | ✅ 自动算 | ⚠️ 重要 | **M2 完成** |
| 佣兵 | ❌ 4 槽空 UI | ✅ 完整面板 | ⚠️ 重要 | **P1** |
| Chronicle 收集 | ❌ 无 | ✅ 0/403 卡 | ❌ 暂不要 | P3 |
| Bound Demon | ❌ 无 | ✅ Warlock 专属 | ⚠️ | P3 |
| 整体步骤/Reset | ❌ 无(`.d2emu-stepper` 已存在,未集成) | ✅ 步进 + Reset | ❌ 暂不要 | P3 |

### 1.3 优化路线图(按 ROI 排序,**实际状态**)

| Phase | 目标 | 内容 | 状态 | Commit |
|---|---|---|---|---|
| **M0** | 接入 waypoints/quests 数据 | 删内联常量,import `data/waypoints.ts` `data/quests.ts` | ✅ **完成** | `c3fb32c` |
| **M1** | 小站/任务 ▼ 折叠 | `useState<Set<number>>` toggle,`aria-expanded`,默认展开 | ✅ **完成** | `c3fb32c` |
| **M2** | 任务 stat/skill pt 摘要 | 改 boolean→number rewards,`computeQuestBonuses`,4 张卡(3 难度+总计) | ✅ **完成** | `b165346` |
| **M3** | 任务图标 | 下载 27 张 d2emu WebP 到 `assets/quest-icons/`,`import.meta.glob` 打包,Quests cell 22×22 | ✅ **完成** | `26cab70` |
| **M4** | 装备详情面板 | `EquipmentDetailModal` 复用 `ItemTooltip` inline + 80×80 图标 + 顶右 ✕ | ✅ **完成** | `ddf820f` |
| **M5** | 视觉 polish | 6 个新 `.d2emu-*` 工具类(label / stat-grid / portrait-wrap / tab-content / progress-cell / collapse-toggle),9 处 inline 替换 | ✅ **完成** | `84418ce` |
| **M6** | 文档修订 | 本节(把脱离实际的原路线图改正) | ✅ **完成** | (本节) |
| **M7** | Stats tab(真做) | **前置**:`src-tauri/protocol/d2s/attributes.rs` 解析 stat id + `StatTable` 聚合(420+ 字段)。**前置风险**:`d2s` 协议可能不携带完整 stat 列表,要先 PoC 1-2 个 stat 验证。**前端**:建 7th tab,3 列布局(属性/抗性/breakpoint) | 🔴 未做 | — |
| **M8** | 佣兵完整化 | **前置**: `src-tauri/protocol/d2s/merc.rs` 解析 merc data(类型/装备/经验/名字)。**前端**: `MercPanel` 5 槽(武器/盾牌/头盔/盔甲 + 1 merc-specific) | 🔴 未做 | — |
| **M9** | Cube items 解析 | `src-tauri/protocol/d2s/cube.rs` 新文件 + `protocol::d2i::legacy::item_sizes` 复用 | 🔴 未做 | — |
| **M10** | Bound Demon tab | 仅 Warlock 类显示,8 mod slot + 类型下拉 | 🟡 不做 | (低 ROI) |
| **M11** | Chronicle 收集 | 403 + 135 + 99 进度卡 + sprite 资源 | 🟡 不做 | (低 ROI) |
| **M12** | 步进器集成 | 用现有 `.d2emu-stepper` class 包 level/str/dex 数字 | 🟡 不做 | (A 类用户不需编辑) |

> **已完成累计 7.5h**(实际工时,6 commits),**剩余真正要做的是 M7(Stats) + M8(佣兵)**。
> M9-M12 ROI 低,暂搁。

### 1.4 验收标准(**M0-M6 已达**)

- [x] **M0**: waypoints/quests 数据从内联常量改为 import(`data/waypoints.ts` `data/quests.ts`),`validateWaypoints()` `validateQuests()` 通过
- [x] **M1**: Waypoints/Quests 3 难度可独立折叠,`aria-expanded` 完整
- [x] **M2**: `computeQuestBonuses` 函数 + 4 张摘要卡(普通/恶梦/地狱/总计),unit test 通过(全 5 reward quest → +5 stat +1 skill)
- [x] **M3**: 27 张 WebP 图标打包进 bundle,Quests cell 显示 22×22 sprite
- [x] **M4**: 装备 12 槽 onSelect 弹 `EquipmentDetailModal`,a11y 完整
- [x] **M5**: 6 个新 `.d2emu-*` 工具类(代替 9 处 inline style)
- [x] **每阶段**: `tsc --noEmit` 0 errors + `vite build` OK
- [ ] **M7(预留)**: Stats tab — 等待 attributes.rs 协议层 PoC
- [ ] **M8(预留)**: 佣兵 — 等待 merc 段解析

### 1.5 不做(Anti-scope,本季度明确放弃)

- ❌ 装备/技能/属性 **写回** d2s(没有 `.d2s` 写序列化器,且改错会毁档)
- ❌ 物品交易/商店/分享功能(属于 StashManager,不是角色页)
- ❌ 步进器集成(A 类用户只看不改,不是 P0)
- ❌ Bound Demon(M10,8h ROI 低,等 Warlock 玩家反馈)
- ❌ Chronicle 收集(M11,16h ROI 低)
- ❌ Cube items 解析(M9,16h ROI 低,且 d2s cube 段协议未调研)

### 1.6 已实施总结(2026-07-08 截止)

| Commit | 内容 | 文件 |
|---|---|---|
| `c3fb32c` | M0 + M1: 接入数据 + ▼ 折叠 | `CharacterPanel.tsx`, `data/waypoints.ts`(新), `data/quests.ts`(新) |
| `b165346` | M2: 任务奖励摘要 | `data/quests.ts` 改 rewards 类型, `CharacterPanel.tsx` 加 `QuestBonusSummary` |
| `26cab70` | M3: 27 张任务图标打包 | `assets/quest-icons/a{act}q{n}.webp` × 27, `utils/questIcons.ts`(新) |
| `ddf820f` | M4: 装备详情 modal | `EquipmentDetailModal.tsx`(新) |
| `84418ce` | M5: 6 个新 d2emu 工具类 | `index.css` 末尾追加 ~70 行 |

**结果**:Quests tab 功能已超越 d2emu(有 stat/skill pt 摘要 d2emu 也没有),装备 tab 持平(都有 hover+点击详情)。

---

## 2. UI 设计师视角 — 视觉规范与线框图

### 2.1 视觉规范(沿用 d2emu 令牌)

#### 颜色(直接用现有令牌,不引入新色)

| 用途 | 令牌 | 值 |
|---|---|---|
| 页面底 | `--color-d2-bg` / `--color-d2emu-bg` | `#0f0d0b` / `#0a0a0a` |
| 卡片底 | `--color-d2-panel` | `#1a1612` |
| 一级线 | `--color-d2-border` | `#3a2f25` |
| 软线 | `--color-d2emu-line-soft` | `#1a1a1a` |
| 主文字 | `--color-d2-text` | `#e7d9b8` |
| 弱文字 | `--color-d2-text-muted` | `#8e8268` |
| **主金(高亮/active)** | `--color-d2emu-gold` | `#FBB13A` |
| **烫金(标题/稀有)** | `--color-d2-gold` | `#c9a34a` |
| 血血红(主行动/危险) | `--color-d2-red` | `#7a1f1f` |
| 装备品质边 | d2emu `QUALITY_HEX` 已在 ItemTooltip 用 | (不变) |

#### 字体

| 角色 | 字体 | 大小 |
|---|---|---|
| 角色名 (H1) | `Cinzel` 700 | 20-24px |
| Tab label | `Source Sans 3` 600 UPPER | 12-13px, letter-spacing 0.08em |
| 数值 (str=297) | `Source Sans 3` 700 tabular-nums | 14-20px |
| 正文 | `Crimson Pro` 400 | 14px |
| 标签 (UPPER) | `Source Sans 3` 600 UPPER | 10-12px, letter-spacing 0.08em |

#### 间距 / 圆角 / 阴影

| 项 | 值 |
|---|---|
| 卡片 padding | 14-20px |
| 卡片间距 | 12-16px |
| 圆角 | 4-6px(尖锐感) |
| 卡片阴影 | `0 2px 8px rgba(0,0,0,0.3)` |
| 物品格 | 50×50px, 1px gap |
| 状态 active 阴影 | `0 0 0 1px var(--color-d2emu-gold)` + 内发光 |

### 2.2 全局线框图(6 tab 状态)

```
┌─────────────────────────────────────────────────────────────────┐
│ [LOGO]  当前页: StashManager              [设置] [角色切换 ▾] │
├─────────────────────────────────────────────────────────────────┤
│ ┌──[角色条]──────────────────────────────────────────┬[LOAD]┐ │
│ │ [头像] 名字 Cinzel 20px        Lv 99 · 术士        │ ┌───┐│ │
│ │         血 1669/2024 (小红条) 魔 343/187 (小蓝条)  │ │ ⬆ ││ │
│ │         [HC] 标签 (血红底)                          │ └───┘│ │
│ └────────────────────────────────────────────────────┴─────┘ │
│  ┌[装备][背包][技能][小站][任务][佣兵]┐           DisplayNames│
│  └────────── active = 红底白字 ──────┘                       │
│  ═══════════════ tab content ═══════════════                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 装备 tab 线框(现有,需小幅 polish)

```
┌─ 装备 ─┐ ┌─ 背包 10×4 (40) ────────────┐
│[Helm]  │ │ [杖][符][药][图]  …4 行     │
│[Amulet]│ └─────────────────────────────┘
│[GL  ]  │ ┌─ 腰带 4×4 ─────┐  ┌─ Cube ──┐
│[Armor] │ │ [药][药][药][药]│  │ □ □ □  │
│[Wm  ]  │ │ [药][药][药][药]│  │ □ □ □  │
│[Sm  ]  │ │ [药][药][药][药]│  │ □ □ □  │
│[Wa  ]  │ │ [药][药][药][药]│  │ □ □ □  │
│[Sa  ]  │ └─────────────────┘  └────────┘
│[GR  ]  │ ┌─ 魔改提示 (dashed 黄边) ─────┐
│[Glove] │ │ ⚠ 魔改 layout 不提取 装备   │
│[Belt] │ │   之外的物品(待 items_modified│
│[Boot] │ │   补充 backpack/belt/cube 段) │
└────────┘ └─────────────────────────────┘
```

**polish 点**:
- 装备空槽显示 `[Helm]` 2 字,占位文字用 `--color-d2-text-muted`
- 装备品质边 2px(已有)
- 物品 hover 弹 tooltip(已有)

### 2.4 技能 tab 线框(关键改进:Warlock 树)

```
┌─ [Unspent ‹ 12 ›]  Level 99 · Quests 12 · Spent 110 ── [Reset]┐
│                                                              │
│  ┌─ Chaos Skills (Warlock) ─┐ ┌─ Eldritch Skills ─┐ ┌─ Demon┐│
│  │ ┌──┐  ┌──┐  ┌──┐         │ │ ┌──┐  ┌──┐  ┌──┐   │ │ …│
│  │ │20│  │20│  │20│         │ │ │20│  │20│  │20│   │ │   │
│  │ └──┘  └──┘  └──┘         │ │ └──┘  └──┘  └──┘   │ │   │
│  │  ↓     ↓     ↓            │ │  ↓     ↓     ↓      │ │   │
│  │ ┌──┐  ┌──┐  ┌──┐         │ │ ┌──┐  ┌──┐  ┌──┐   │ │   │
│  │ │39│  │39│  │20│         │ │ │39│  │20│  │20│   │ │   │
│  │ └──┘  └──┘  └──┘         │ │ └──┘  └──┘  └──┘   │ │   │
│  │ 背景:深灰石纹 + 金色节点  │ │                      │ │   │
│  └────────────────────────┘ └─────────────────────┘ └───┘
│  (8 职业 layout 全在 skillTreeLayouts.ts,含 Warlock Chaos/Eldritch/Demon 3 树)
└──────────────────────────────────────────────────────────────┘
```

**设计原则**:
- 三栏等宽,居中布局,背景纹理
- 节点 48×48,有技能点 = 金色发光,无技能点 = 灰,锁定 = 深灰
- 节点右下显示等级(`<span class="skill-points">{level}</span>`)
- 节点间箭头用 CSS `::after` 画(已有)

### 2.5 Stats tab 线框(M7 计划中,3 列编辑表)

```
                                                              [Reset]
┌─ ATTRIBUTES + RESOURCES ─┐ ┌─ RESISTANCES + BREAKPOINTS ─┐ ┌─ OTHER ──┐
│ STAT    BASE  GEAR  TOTAL│ │ STAT   GEAR  QUESTS  TOTAL   │ │ Magic    │
│ Str     297   +75   372  │ │ Fire   +158  +30     188     │ │   174%   │
│ Dex      20   +40    60  │ │ Light  +193  +30     223     │ │ Gold     │
│ Vit     248   +40   288  │ │ Cold   +188  +30     218     │ │   210%   │
│ Eng      20   -40    60  │ │ PSN    +158  +30     188     │ │ All Skl  │
│ Unspent  ‹ 0 ›  +15      │ │ FCR         +125     125     │ │    +6    │
│ Life   1669  +355  2024  │ │ FHR          +54      54     │ │ …        │
│ Mana    323  +20    343  │ │ FBR          +0       0      │ └──────────┘
│ Stamina 467  +20    487  │ │ IAS          +20      20     │
└──────────────────────────┘ │ FRW          +105    105     │
                             └────────────────────────────┘
```

**设计原则**:
- 列对齐右(`text-align: right`),数字 tabular-nums
- stepper 走 d2emu `‹ ›` 样式(参考 d2emu 步进器)
- Reset Stats 按钮(顶右)二次确认

### 2.6 Waypoints / Quests tab 线框(▼ 折叠)

```
▼ NORMAL                                                          [Unmark]
┌─ A1 ───────┐ ┌─ A2 ──┐ ┌─ A3 ───┐ ┌─ A4 ───┐ ┌─ A5 ───┐
│ ☑ 卍 Rogue │ │ ☑ 卍 … │ │ ☑ 卍 …  │ │ ☑ 卍 …  │ │ ☑ 卍 …  │
│ ☑ 卍 Cold  │ │ ☑ 卍 … │ │ ☑ 卍 …  │ │ ☑ 卍 …  │ │ ☑ 卍 …  │
│ ☑ 卍 Stony │ │ ☑ 卍 … │ │ ☑ 卍 …  │ │ ☑ 卍 …  │ │ ☑ 卍 …  │
│ (9 站)    │ │ (9)   │ │ (9)    │ │ (3)    │ │ (9)    │
└────────────┘ └───────┘ └────────┘ └────────┘ └────────┘
▼ NIGHTMARE  (同样 5×9, 但每个 d2s jf 段独立位)
▼ HELL
```

**改进点**:
- M3: 任务 tab 加 sprite icon (来自 `QUEST_ICON_INDEX`)
- M5: 小站 tab 改 ▼ 折叠结构(目前 3 个 diff 始终展开)
- Unmark All 按钮(已有的话确认)

### 2.7 状态变体(必含)

| 状态 | 触发 | 表现 |
|---|---|---|
| **loading** | `loading && !character` | 居中 spinner + "加载中…"(已有) |
| **empty** | `!loading && !character && characters.length===0` | "未找到角色档案" (已有) |
| **empty-after-select** | 选了角色但读不到 | 错误 toast + 提示"重选或刷新" |
| **魔改 layout** | `is_modified_layout === true` | 属性方块全 0,顶部 dashed 黄边提示"魔改存档" (已有) |
| **错误** | 后端 Err | 红色 toast + 重试按钮 |
| **Resize** | 窗口窄 | 装备/背包从 4×5 缩到 3×5,物品 50→40 |

---

## 3. UX 设计师视角 — 交互要求设计

### 3.1 用户旅程(User Journey)

```
                   ┌─ 主菜单进 ──┐
                   │             │
                   ▼             │
   ┌─ ① 启动 → 看 StashManager 首页 (默认) ─┐
   │                                          │
   │  ② 切角色 (顶部 select)                   │
   │      ↓                                    │
   │  ③ read_character_info 加载(loading 0.5s) │
   │      ↓                                    │
   │  ④ 默认进 装备 tab                        │
   │      ↓                                    │
   │  ⑤ hover 装备 → ItemTooltip               │
   │      ↓                                    │
   │  ⑥ 切到 技能 tab(扫技能分配)               │
   │      ↓                                    │
   │  ⑦ 切到 小站/任务 tab(查进度)              │
   │                                          │
   └─ ⑧ 关掉 / 切回 stash ────────────────────┘
```

**关键节点**:
- **②-③**: 角色切换要 < 1s,期间 spinner 不可阻塞主区域
- **④**: 默认 tab = 装备(用户最常看)
- **⑤**: tooltip 200ms 延迟出现,避免误触
- **⑥**: 技能 tab 树展开 < 100ms(无网络请求,纯前端)

### 3.2 关键交互模式

| 模式 | 行为 | 反馈 |
|---|---|---|
| **切 tab** | click | 红底白字 100ms 切换,无 fade(避免拖沓) |
| **hover 装备** | 200ms 后 | 弹 ItemTooltip (position=top/bottom) |
| **hover 技能点** | 100ms 后 | 弹 SkillTooltip (含 synergy/passive) |
| **拖拽 (未实现)** | dragstart | 半透明 + 0.5s 目标格高亮 |
| **步进器 (M12 不做)** | click ‹/› | 数字 ±1,边界 disabled |
| **Reset (M2)** | click 按钮 | confirm 弹窗 → 二次确认 → 提交 |
| **折叠 ▼** | click header | 高度过渡 200ms,图标 rotate 90° |
| **窗口 resize** | reactive | grid 4×5 → 3×5 (min-width 480px) |

### 3.3 状态机(Tab 状态)

```
        ┌─ loading=true ─┐
START ──┤                ├──→ READY
        └─ error=true ───┘       │
                                 │ tab=overview/inventory/skills/...
                                 ▼
                              (render tab content)
                                 │
                                 │ character=null
                                 ▼
                              EMPTY_STATE
```

**组件级状态**(`CharacterPanel`):
```
tab: 'overview' | 'inventory' | 'skills' | 'waypoints' | 'quests' | 'merc'  (default: 'overview')
```

### 3.4 微交互(Micro-interactions)

| 触发 | 时长 | 效果 |
|---|---|---|
| Tab 切换 | 100ms | 红底 active,无 fade |
| 装备 hover | 200ms 延迟 | tooltip 渐入 120ms |
| 技能节点 hover | 100ms | 节点外发光 12px 金色 |
| Spinner 出现 | 立即 | 24px 金色旋转 |
| 步进器 click | 立即 | 数字 bounce 100ms |
| Toast 错误 | 立即 | 顶部下滑 200ms,3s 自动消失 |

### 3.5 可访问性 (a11y) 要求

| 项 | 要求 |
|---|---|
| 键盘导航 | Tab 顺序:角色 select → 6 个 tab → 内容区第一项 |
| ARIA | `role="tablist"` / `role="tab"` / `aria-selected` |
| Focus 环 | `outline: 2px solid var(--color-d2emu-gold); outline-offset: 2px` |
| 屏幕阅读 | `aria-label="装备 - 12 槽"` |
| 颜色对比 | 主文字 vs 背景 ≥ 4.5:1 (实测 `--color-d2-text` on `--color-d2-bg` ≈ 9.5:1 ✓) |
| 装备品质色盲 | 边框颜色 + 文字标识(已有 `name_zh` 后缀) |

### 3.6 错误处理 UX

| 错误 | UI |
|---|---|
| 角色文件读取失败 | 红色 toast:`无法读取 .d2s` + 重试按钮 |
| 魔改 layout attrs 0 | 顶部黄边提示 + attr 方块显示"—"代替 0 |
| 后端 timeout | spinner 5s 后 → 失败 toast |
| 缺失图像 | `onError` fallback 到 code 文字 (已有) |

### 3.7 性能要求

| 指标 | 目标 |
|---|---|
| 角色切换响应 | < 200ms (Tauri 本地读 < 100ms) |
| Tab 切换 | < 16ms (1 帧) |
| 装备 hover tooltip | < 200ms 延迟 |
| 角色页首次渲染 | < 500ms (含 skills 树 lazy load) |
| 内存 | < 50MB (单角色 1-3MB JSON) |

### 3.8 不做的交互 (Anti-interactions)

- ❌ **全局搜索框**(角色页只有 1 个角色,搜索无意义)
- ❌ **拖拽改装备**(只读定位)
- ❌ **多选/批量操作**(角色页是档案展示,不是 list)
- ❌ **快捷键切 tab**(用户操作频次低,鼠标够用)
- ❌ **装备 3D 旋转预览**(图像已够,加 3D 是炫技)

---

## 4. 关联文档

- 设计系统: `docs/design/d2emu-hero-design-system.md`
- 工具布局: `docs/design/d2emu-hero-editor-tool-layout.md`
- d2s 协议: `src-tauri/src/protocol/d2s/`
- 基础数据: `web/src/data/waypoints.ts`, `web/src/data/quests.ts`
- 角色组件: `web/src/components/CharacterPanel.tsx`, `EquipmentPanel.tsx`, `SkillTree.tsx`, `ItemTooltip.tsx`

---

## 5. 变更日志

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-07-08 | 0.1 | 初稿,PM/UI/UX 三视角综合 |
| 2026-07-08 | 0.2 | **修订**:修正 §1.2 Gap(原 Warlock 缺错误,实际已有),重写 §1.3 路线图为实际状态(M0-M6 done / M7-M12 未做),加 §1.6 已实施总结,§1.4 验收打勾。修订原因:0.1 版基于未核实假设(Warlock layout,Stats tab, d2emu-stepper 等),与实际代码不符 |
