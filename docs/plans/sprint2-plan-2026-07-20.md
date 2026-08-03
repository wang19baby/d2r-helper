# Sprint 2 计划 · 2026-07-20

> **来源**: `docs/handoffs/sprint1-review-pm-ppm-2026-07-20.md` 双角色 review 通过的 backlog
> **范围**: Sprint 1 收尾后未完成的 P0/P1/P2 任务
> **节奏**: 3 周,Week 1 必修 / Week 2 必修 / Week 3 质量

---

## 0 · 入口上下文

### 0.1 Sprint 1 状态
- 11/13 P0 落地(85%),2 defer(等 backend `swap_equipment`,v3 范围)
- L1 缓存基础设施已就绪(9 文件 + 38 测试)
- 验证基线:`tsc --noEmit` 0 errors · `npm test` 38/38 pass

### 0.2 Sprint 2 三大入口(从 review 倒推)
1. 🔴 **Call-site migration** — L1 价值未兑现
2. 🔴 **装备上平台二级筛选** — mod stash 玩家核心痛点
3. 🔴 **Characters 三段事件 + useCached 兼容** — cache 完整覆盖

### 0.3 风险预警
- cache 层单测是孤岛,Sprint 2 call-site migration 没有回归测试网保护
- 装备上平台涉及 `warehouseStore` 重构,需提前对接 spec §3.7
- Characters 三段事件不强行包装 useCached,需要扩展 hook 形态

---

## 1 · Week 1 · Cache 落地 + 触发器收口(P0 必修)

### 1.1 目标
让 L1 缓存基础设施真正进入页面,消除"孤岛代码"状态。

### 1.2 任务清单

#### T1.1 · Warehouse.tsx call-site migration
- **当前**: `Warehouse.tsx:80` 直接 `tauriInvoke('warehouse_search')`
- **目标**: 走 `warehouseStore.search(filters)`,5 维签名 searchKey
- **验收**:
  - `useCached(() => warehouseStore.search(filters), { key: warehouseStore.searchKey(filters), maxAgeMs: 30_000 })`
  - 写成功(`warehouse_deposit` / `warehouse_withdraw` / `warehouse_remove` / `warehouse_update_meta`)后 invalidate `warehouse:` 命名空间
- **测试**: `tests/integration/warehouse-store.spec.tsx` mock tauriInvoke,验证搜索/失效
- **锚点**: `web/src/pages/Warehouse.tsx:75-87, 312-318, 352-360`

#### T1.2 · Inventory.tsx call-site migration
- **当前**: `Inventory.tsx:58` 直接 `tauriInvoke('read_stash')`
- **目标**: 走 `stashStore.fetch(name)`,L2 绑定 `d2r-char-stash-<name>`
- **验收**:
  - `useCached(() => stashStore.fetch(activeName), { key: stashStore.fullKey(activeName), maxAgeMs: 60_000 })`
  - 写成功(`list_item` / `warehouse_deposit` / `warehouse_withdraw`)后 invalidate `stash:<name>`
- **测试**: `tests/integration/stash-store.spec.tsx`
- **锚点**: `web/src/pages/Inventory.tsx:53-58, 120-130, 313`

#### T1.3 · RunewordCalc.tsx call-site migration
- **当前**: 直接 `tauriInvoke('find_runewords')` + 自管 localStorage
- **目标**: 走 `runeWordStore.compute(ownedRunes)`,L2 绑定 `runeword-cache-all` + `runeword-context-cache`
- **验收**:
  - `useCached(() => runeWordStore.compute(owned), { key: runeWordStore.resultsKey(owned), maxAgeMs: 600_000 })`
  - `runeWordStore.getContext()` 同样走 hook
- **测试**: `tests/integration/runeword-store.spec.tsx`
- **锚点**: `web/src/pages/RunewordCalc.tsx:268-285, 419-455`

#### T1.4 · Characters.tsx 三段事件 + useCached 兼容方案
- **当前**: 走 `char:stage1` / `char:stage3` / `char:error` event bus,不适合 `useCached` 的 Promise 形态(见交接 §7.6)
- **目标**: 扩展 `useCache.ts` 支持 event-based loader,或在 Characters 内包一层 event→Promise adapter
- **验收**:
  - 选定方案(扩展 hook vs adapter),记录决策
  - `characterStore.list` 走 `useCached`(L2 `d2r-character-names`)
  - `characterStore.full(name)` 走 event-based wrapper 或 hybrid
- **测试**: `tests/integration/characters-store.spec.tsx` 覆盖 stage1/stage3/event abort
- **锚点**: `web/src/pages/Characters.tsx:174-242`, `web/src/cache/characters.ts`

#### T1.5 · Warlock 8 职业 chip 完整性审计
- **目标**: 确认 CharacterPicker 的 8 职业 chip 包含 Warlock(2026-07 添加的职业)
- **验收**: 测试覆盖所有 8 个职业的 chip 渲染
- **测试**: `tests/components/CharacterPicker.spec.tsx` 加 Warlock case
- **锚点**: `web/src/pages/Characters.tsx:64-103`

### 1.3 Week 1 验收标准

- 4 page 全部走 cache store,无散落 `tauriInvoke` 调用
- `useCached` 失效链路(写命令 → invalidate → UI 自动更新)验证通过
- `tsc --noEmit` 0 errors · `npm test` 通过(应 > 38)
- 集成测试新增 ≥ 4 个

---

## 2 · Week 2 · 装备上平台 + 角色过滤(P0)

### 2.1 目标
补完 spec §3.7 P0(装备上平台二级筛选),同时落实 §1.2 角色 chip 过滤接 characterStore。

### 2.2 任务清单

#### T2.1 · 装备上平台二级筛选组件 `EquipmentFilterChips`
- **当前**: Inventory 类目仅 11 大类,装备类目无二级
- **目标**: 装备类目激活时展开二级 chip:**品质** / **部位** / **底材**
- **验收**:
  - 复用 spec §3.3.2 已描述的 `expanded: boolean` 状态
  - 二级筛选 row 沿用 `.d2emu-subtab` 风格
  - 筛选条件走 `useCached(() => warehouseStore.search(filters), ...)` 模式(由 T1.1 接入)
- **测试**: `tests/components/EquipmentFilterChips.spec.tsx` + `tests/integration/warehouse-filter-equipment.spec.tsx`
- **锚点**: `web/src/pages/Inventory.tsx:184-201`, `web/src/components/EquipmentFilterChips.tsx`(新建)

#### T2.2 · Characters chip 过滤接 characterStore
- **当前**: chip state 已在 Characters 页,但未接 store(交接 §7.6 决策先不上)
- **目标**: chip 过滤走 `characterStore.list` + 客户端 memo filter
- **验收**:
  - 切换职业 chip → 角色列表实时过滤
  - chip state 持久化到 L2(`d2r-character-names` 关联 metadata)
- **测试**: `tests/components/Characters-filter.spec.tsx`
- **锚点**: `web/src/pages/Characters.tsx:64-103, 287-330`

#### T2.3 · Characters stage1 abort + 错误 toast
- **当前**: `loadingCharRef` 仅做"晚到的 stage3 拦截",没有真正 abort
- **目标**: 真正 abort(AbortController 或版本号)+ stage1 err toast
- **验收**:
  - 连点两个角色,第一个的 stage1/3 结果全部丢弃
  - stage1 err → 顶部 toast + 自动重试按钮(最多 1 次)
- **测试**: `tests/components/Characters-abort.spec.tsx` + 注入 stage1 err
- **锚点**: `web/src/pages/Characters.tsx:174-242`

#### T2.4 · HandSwapButton 占位(等 backend v3)
- **当前**: 缺
- **目标**: 装备面板加"主手 ↔ 副手"按钮,点击 toast "即将推出",等 `swap_equipment` backend
- **验收**:
  - 按钮 disabled + toast "即将推出 - 需 swap_equipment 后端命令"
  - 等 backend 落地后直接接通
- **测试**: 暂不需要(纯占位)
- **锚点**: `web/src/components/EquipmentPanel.tsx:160-165` 加占位

### 2.3 Week 2 验收标准

- Inventory 装备类目下二级筛选可用(mod stash 100+ 件装备不再"几乎不可用")
- Characters chip 过滤实时生效
- stage1/3 abort + error toast 完整
- `tsc --noEmit` 0 errors · `npm test` 通过(应 > 50)
- 集成测试新增 ≥ 3 个

---

## 3 · Week 3 · 测试 + a11y + 触屏

### 3.1 目标
补齐验收"三件套"覆盖率(目前 0% UI 测试),降低 Sprint 2 call-site migration 引入的回归风险。

### 3.2 任务清单

#### T3.1 · UI/state flow 集成测试
- **目标**: `tests/integration/` 目录 mock tauriInvoke,测 4 page + cache store 调用路径
- **范围**:
  - `warehouse-search-integration.spec.tsx`(过滤组合)
  - `stash-fetch-integration.spec.tsx`(切角色 / 写后失效)
  - `runeword-compute-integration.spec.tsx`(owned runes 变化)
  - `characters-event-integration.spec.tsx`(stage1/3/abort)
- **验收**: 4 个 integration 文件,每个 ≥ 3 个 case
- **锚点**: `web/tests/integration/`

#### T3.2 · E2E (playwright) 4 主菜单 smoke
- **目标**: 4 主菜单(角色/仓库/共享仓库/符文计算)的核心 flow 跑通
- **范围**:
  - `characters-tab-flow.spec.ts`(选角色 → 切 tab → 死亡 banner)
  - `warehouse-withdraw-flow.spec.ts`(搜索 → 取回选位 → 落盘)
  - `stash-batch-flow.spec.ts`(批量选 → 批量上架/入仓)
  - `runeword-consumption-flow.spec.ts`(选符文 → StatBanner → BaseQualityFilter)
- **验收**: 4 个 e2e 文件,可在 CI 跑通
- **锚点**: `web/tests/e2e/`

#### T3.3 · a11y 审计(axe-core)
- **目标**: 跑 axe-core,修复 ≥ 80% 严重项
- **范围**:
  - 11 个新组件全部审计
  - 4 page 主页审计
  - Modal 焦点陷阱验证
- **验收**: axe-core 集成到 CI,新增严重项 0
- **锚点**: `web/tests/a11y/`, CI 配置

#### T3.4 · 触屏断点复查(EquipmentPanel + CharacterPanel)
- **当前**: Inventory 有 sticky bar,EquipmentPanel/CharacterPanel 未复查(交接 §7.6 + review §3.3)
- **目标**: `<800px` 断点下两个 panel 可用
- **验收**:
  - EquipmentPanel 触屏模式:装备槽放大到 56-80px,行可滑动
  - CharacterPanel 触屏模式:tab 切换可点击区域 ≥ 36×36
- **测试**: playwright mobile viewport 测试
- **锚点**: `web/src/components/EquipmentPanel.tsx`, `web/src/components/CharacterPanel.tsx`

#### T3.5 · 性能:d2i 5000 件虚拟滚动(可选)
- **目标**: d2i 多 page 大量物品下不卡(交接 §4 + spec §3.7)
- **验收**: 5000 件首屏渲染 < 200ms,使用 react-window 或自实现
- **测试**: playwright `stash-large-perf.spec.ts`
- **锚点**: `web/src/pages/Inventory.tsx`

### 3.3 Week 3 验收标准

- integration 测试 ≥ 12 个 case
- e2e 测试 4 个主菜单 smoke 全过
- axe-core 严重项 0
- 触屏断点(EquipmentPanel/CharacterPanel)可用
- `tsc --noEmit` 0 errors · `npm test` 通过(应 > 60)
- 性能:d2i 5000 件首屏 < 200ms(如有余量)

---

## 4 · Sprint 2 总览

| Week | 主题 | 关键产出 | 测试增量 |
|---|---|---|---|
| W1 | Cache 落地 + 触发器收口 | 4 page call-site 接入 + Characters 三段事件兼容 | +4 integration(≥ 12 case) |
| W2 | 装备上平台 + 角色过滤 | EquipmentFilterChips + Characters chip 接 store + abort + HandSwapButton 占位 | +3 integration(≥ 9 case) |
| W3 | 测试 + a11y + 触屏 | e2e 4 menu + axe-core + 触屏 + 虚拟滚动(可选) | +4 e2e + a11y 全过 |

### 4.1 Sprint 2 验证门槛

```bash
# 必跑
cd web
./node_modules/.bin/tsc --noEmit                # 0 errors
npm test                                         # 38 + ≥ 25 case = ≥ 63 通过
npm run test:e2e                                 # 4 menu smoke 全过
npm run test:a11y                                # axe-core 0 严重
```

### 4.2 Sprint 2 不做(留给 v3)

| 任务 | 原因 |
|---|---|
| 切换主副手(真接通) | 需 backend `swap_equipment` 命令 |
| 一键替换(真接通) | 需 backend `swap_equipment` 命令 |
| 撤销上架通道 | 需后端 cancel_listing 改进 + UI undo |
| d2emu hero editor 集成 | 跨项目集成,Sprint 3+ 范围 |
| Warlock 全身像 CC0 替代 | 资源策略,需用户协调 |

---

## 5 · 风险登记册(Sprint 2 入口)

| 风险 | 严重度 | 缓解 |
|---|---|---|
| cache call-site migration 引入 UI 回归 | 🟡 中 | Week 3 前先扩 cache 单测覆盖 |
| Characters 三段事件 useCached 兼容设计有偏差 | 🟡 中 | T1.4 先 POC 两个方案,选小步前进 |
| EquipmentFilterChips 触屏复杂度溢出 | 🟢 低 | 与 T3.4 触屏复查合并做 |
| a11y axe-core 在 CI 失败阻塞 PR | 🟡 中 | 阈值设宽松,严重项先修,warn 项待办 |

---

## 6 · 与 Sprint 1 的连续性

### 6.1 沿用基础设施

| 资产 | 路径 | Sprint 2 用途 |
|---|---|---|
| `ClientCache.ts` | `web/src/cache/` | T1.1-T1.3 call-site 迁移直接用 |
| `useCache.ts` | `web/src/cache/` | T1.4 扩展 event-based loader |
| `events.ts` | `web/src/cache/` | T1.4 + W3 失效链路 |
| `BaseQualityFilter.tsx` | `web/src/components/` | T2.1 装备上平台二级 chip 参考样式 |
| `BatchSellModal / BatchDepositModal` | `web/src/components/` | T2.1 复用批量模式 |

### 6.2 复用 spec 与决策

- spec §0.1 资源边界:Sprint 2 沿用(Warlock 仍是 SVG fallback)
- spec §1.2-1.5 缓存设计:Sprint 2 W1 直接落地
- 交接 §7.6 Characters 三段事件决策:Sprint 2 T1.4 重新审视

---

## 7 · 版本与变更日志

| 版本 | 日期 | 变更摘要 |
|---|---|---|
| v1 | 2026-07-20 | 初版:从 review 倒推的 Sprint 2 三周排期,W1 cache / W2 装备 + 角色 / W3 测试 + a11y |
