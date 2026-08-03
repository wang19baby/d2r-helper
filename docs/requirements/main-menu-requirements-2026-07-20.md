# D2R 助手 · 一级菜单需求文档 (v2)

> **版本**：v2 / 2026-07-20（基于 v1 + 多角色评审迭代）
> **评审来源**：PM / 架构 / QA / 设计 / 老玩家 / 合规 六个角色
> **使用方式**：本文档为"一级菜单需求的 source of truth"，所有 P0 配套 **验收 / 测试 / 锚点** 三件套，便于后续 PR 直接 attach。
>
> ### v1 → v2 关键变更（TL;DR）
> 1. **§0 新增三件套**：资源与版权边界 / 文档约定 / 优先级重排表
> 2. **§A/B/C 三大附录**：Tauri 命令登记表 / localStorage key 总账 / 数据流图占位
> 3. **每条 P0/P1 配 "验收 / 测试 / 锚点" 三件套**
> 4. **P0 重排**：按玩家真实使用频率，不再按工程师修复成本
> 5. **Warlock 图片策略**：三种路径（本地/CC0/SVG）按风险分层
>
> ### 引用
> - 路由：`web/src/App.tsx::NAV`
> - 角色全 tab 名：`web/src/components/CharacterPanel.tsx::TABS`
> - 类型：`web/src/types.ts`
> - Tauri 桥：`web/src/tauri.ts`

---

## §0 · 文档元信息

### §0.1 资源与版权边界（必读）

> ⚠️ 本项目为**开源仓库**，Blizzard 官方美术资产（含 Warlock 全身像、D2R 角色 select 帧）**禁止 commit 到任何 git 远程 / 禁止再分发**。

| 资源类型 | 合法用途 | 入仓 | 路径建议 |
|---------|---------|------|---------|
| **Blizzard 官方美术**（Warlock 全身像、官方截图、宣传图） | ✅ 个人本地、单机调试 | ❌ 禁止 | `web/public/assets/characters/*.png` + 加 `.gitignore` |
| **D2 fan-art / OpenGameArt CC0 / CC-BY** | ✅ 入仓 + 再分发 | ✅ 允许 | `web/public/assets/characters/`，附 `CREDITS.md` |
| **自绘 inline SVG / CSS pixel art** | ✅ 入仓 + 商用 | ✅ 允许 | 扩 `CharacterPanel.tsx` 内联 SVG 风格 |

#### 角色图片策略（v2 推荐执行路径）

1. **P0 占位（fallback）**：8 职业 inline SVG 全身剪影（配色 + 武器道具差异），agent 可代写。
2. **P0 玩家模式**：玩家自己从 D2R 游戏截图 → 放 `web/public/assets/characters/<class_en>.png` → **不入仓**（`.gitignore` 排除）→ 应用内 `import` 时优先使用本地图，缺图回落 SVG。
3. **P2 升级**：引入 OpenGameArt CC0 全身像作为离线默认 → 可入仓。

> 评审记录：合规视角"❌ 缺资源策略章节"——本节即为补齐。

### §0.2 文档约定

**emoji 统一**：

- ✅ = 已实现
- ⚠️ = 风险/待优化（优先实现）
- ❌ = 缺口（必须实现）

**每条 P0 / P1 需求都配套三件套**：

- **验收**：完成标准 + 可观察信号
- **测试**：单元 / 集成 / E2E 用例名（vitest / playwright）
- **锚点**：涉及文件 + 行号（PR 直接 attach）

### §0.3 优先级重排表（v2 vs v1）

| 需求 | v1 优先级 | v2 优先级 | 变更原因 |
|------|----|----|----|
| 角色 Tab · 切换主副手 | P2 | **P0** | 野蛮人 / 死灵 / 刺客 daily 刚需 |
| 装备 Tab · 一键替换 | P1 | **P0** | 切装高频刚需 |
| 符文计算 · 符文消耗统计 | P2 | **P0** | "防丢三连"高水平玩家最关心 |
| 符文计算 · 底材 quality 过滤（normal/superior/ethereal） | 未列 | **P0** | 对 build 影响巨大，目前 UI 完全缺失 |
| 符文计算 · 符文位置追溯 | 未列 | **P1** | 依赖 P0 符文消耗统计 |
| 符文计算 · stars ≥ 4 浮标 | 隐含 | **P1** | 修正：Spirit 2★ 但最有价值，单纯按 stars 不准 |
| 角色 stage1 错误 toast | P0 | **P1** | edge case，不算高频崩溃 |
| 共享仓库 · 装备筛选（部位/品质/底材） | P0 | **P0** | 保留 |
| 共享仓库 · 批量上架/入仓 | P0 | **P0** | 保留 |
| 仓库 · 取回时选位 | P0 | **P0** | 保留 |
| 仓库 · 错误改 error toast | P0 | **P0** | 保留 |
| 角色存档 hash 变更 toast | P1 | **P1** | 保留 |
| 多语种技能名 | P1 | **P1** | 保留 |

---

## 一、角色 (`/characters`)

### 1.1 Hero 区

- ✅ 刷新 / 单角色重新加载全链路打通。
- ⚠️ "存入仓库" 是全量覆盖式写入
  - **验收**：`extract_character_equipment` 调用成功后 UI toast 显示"已存入 N 件装备"，N = d2s 文件实际装备数（不含背包）。
  - **测试**：vitest `characters-extract-equipment.spec.ts`
  - **锚点**：`Characters.tsx:161-173`, `commands/character.rs::extract_character_equipment`
- ⚠️ Hero 区缺语言 chip 预览
  - **验收**：Hero 加 `zhCN / zhTW / enUS` chip，实时切换不影响持久化（设置页仍是默认）。
  - **测试**：vitest `characters-language-chip.spec.tsx`
  - **锚点**：`Characters.tsx:280-303`

### 1.2 角色列表（左侧列）

- ⚠️ `loadingCharRef` 仅做"晚到的 stage3 拦截"，没有真正 abort
  - **验收**：连点两个角色，第二个角色切到后第一个角色的所有 stage1/3 结果被丢弃（不写 state）。
  - **测试**：vitest `characters-abort-stale-loads.spec.ts`
  - **锚点**：`Characters.tsx:174-242`
- ⚠️ **P1** · stage1 失败没有 toast
  - **验收**：stage1 err → 顶部 toast "加载角色失败: \<err\>" + 自动重试按钮（最多 1 次）。
  - **测试**：vitest `characters-stage1-error.spec.tsx` + 注入 stage1 err
  - **锚点**：`Characters.tsx:189-210`
- ❌ 没有"按职业 / 状态"过滤
  - **验收**：左侧列表上方增加 chips：`全部 / 资料片 / 经典 / 专家 / 普通 / 死档`，行项实时过滤。
  - **测试**：vitest `characters-filter-chips.spec.tsx`
  - **锚点**：TBD（v3 实现）
- ❌ 没有角色重命名 / 复制 / 删除入口
  - **锚点**：TBD（v3 实现），UI 出现在行项右键菜单

### 1.3 8 大二级 Tab（按 `CharacterPanel.tsx::TABS`）

> TABS 顺序：**装备 / 背包 / 仓库 / 技能 / 小站 / 任务 / 奖励 / 佣兵**。
> 注：第 9 个 tab `warehouse` 在 `MicroTab` 类型里，渲染层复用 `read_stash`（与一级菜单"仓库/共享仓库"语义有重叠，见 §1.3.3）。

#### 1.3.1 装备 Tab (`overview`)

- ⚠️ `EquipmentPanel` low-dur 触发时 `durability_max === 0` 会零除
  - **验收**：`durability_max === 0`（无形武器、任务道具）不触发低耐久红框，不抛 NaN。
  - **测试**：vitest `equipment-panel-zero-div.spec.tsx`（输入 `durability_max=0`）
  - **锚点**：`EquipmentPanel.tsx:251-252`
- ⚠️ **P0 升** · 切换主副手
  - **验收**：装备面板设"主手 ↔ 副手" swap 按钮，点击后装备交换，d2s 文件落盘。野蛮人 / 死灵 / 刺客可玩。
  - **测试**：vitest `equipment-swap-hands.spec.ts`
  - **锚点**：TBD（v3）
- ⚠️ **P0 升** · 一键替换
  - **验收**：在共享仓库 browse 一件装备后，装备面板出现"装备到 X 槽"按钮，confirm 后 d2s + UI 同步更新。
  - **测试**：vitest `equipment-one-click-replace.spec.ts` + playwright e2e
  - **锚点**：TBD（v3）
- ⚠️ 没有 `equipmentSkillBonuses` 聚合 UI
  - **验收**：装备 Tab 顶部增加 "技能加层 (来自装备)" section，聚合所有 `+N 技能系 / X% 施法 / N/M 充能`，按 `kind` 分组。
  - **测试**：vitest `equipment-skill-bonuses-aggregated.spec.tsx`
  - **锚点**：`CharacterPanel.tsx:337-340`, `types.ts::SkillBonus`
- ⚠️ 图片资源无懒加载（1280 件装角色会触发 CDN 高频请求）
  - **验收**：滚动出视口的 img 元素不发起 `<img src>` 请求；回到视口后正常加载。
  - **测试**：playwright `equipment-img-lazy.spec.ts`（拦截 outbound request 计数）
  - **锚点**：`utils/itemImages.ts::resolveItemIcon`

#### 1.3.2 背包 Tab (`storage`)

- ⚠️ 18 装备槽渲染 + mod layout 多页未覆盖
  - **锚点**：跟踪 `InventoryView.tsx`, `protocol::d2i::parser.rs` —— 属 mod 解析范围
- ❌ 没有物品右键菜单（卖出 / 入仓 / 替换）
  - **验收**：物品格右键弹出菜单：① 上架到市场 ② 存入扩展仓库 ③ 装备到此槽（仅装备类） ④ 查看详情。
  - **测试**：vitest `inventory-right-click-menu.spec.tsx`
  - **锚点**：TBD（v3，`InventoryView.tsx` 增加 `onContextMenu`）
- ❌ 个人仓库格子 16×16 硬编码
  - **验收**：从 d2s JM 段 `page=5` 读实际网格 size，mod 拓展 stash 显示真实尺寸。
  - **测试**：vitest `inventory-personal-stash-dynamic-size.spec.tsx`
  - **锚点**：`InventoryView.tsx:60-65`, `protocol::d2s::attributes`

#### 1.3.3 仓库 Tab (`warehouse`，角色内) —— ⚠️ 命名重叠

- ⚠️ `d2r-char-stash-<name>` 跨页 stale
  - **验收**：玩家在共享仓库移除某 d2i 页 → 角色仓库 tab 切换回来时自动 `forceRefresh=true`。
  - **测试**：vitest `stash-cross-page-stale.spec.tsx`
  - **锚点**：`CharacterPanel.tsx:262-298` + 附录 B §跨页 stale 治理
- ❌ 缺"复制到角色个人仓库"动作
  - **验收**：在角色仓库 tab 选中物品 → "复制到我的个人仓库"按钮，确认后写入 d2s page=5。
  - **测试**：vitest `stash-copy-to-personal.spec.ts`
  - **锚点**：TBD（v3）

#### 1.3.4 技能 Tab (`skills`)

- ⚠️ Warlock 0↔2 索引交换硬编码于 `summary`
  - **验收**：`swapTreeIndices(class_en)` 抽到 `utils/skillPresentation.ts`，覆盖 `Warlock / Sorceress` 两条路径，单元测试断言两条互不交叉。
  - **测试**：vitest `skill-warlock-swap.spec.ts`
  - **锚点**：`SkillTree.tsx:290-299`
- ❌ 没有"+加 1 点"模拟器
  - **验收**：preview 模式显示"+1 点后 vs 当前" 的 stat diff；点击 pin 后退出预览不写 d2s。
  - **测试**：vitest `skill-preview-planner.spec.tsx` + playwright e2e
  - **锚点**：TBD（v3）
- ⚠️ 多语种只 `zhCN/zhTW`，无 enUS/koKR fallback
  - **验收**：`get_localized_skill_texts` 加载失败或为空时 fallback 到 `skillName(class_en, id)` 硬编码英文；不报 undefined。
  - **测试**：vitest `skill-i18n-fallback.spec.tsx`
  - **锚点**：`SkillTree.tsx:302-310`, `commands/skill.rs::get_localized_skill_texts`

#### 1.3.5 小站 Tab (`waypoints`)

- ✅ 数据通路完成（`waypoints` 字段在 `CharacterInfo`）。
- ❌ UI 仅渲染列表，缺"传送计算 / 路线推荐"
  - **验收**：点击任一小站显示"从当前位置到目标"的最优路径（基于已通过 Act 的连通性）。
  - **测试**：vitest `waypoints-route-planner.spec.tsx`
  - **锚点**：`pages/characters/CharacterDetails.tsx::CharacterWaypoints`

#### 1.3.6 任务 Tab (`quests`)

- ⚠️ `quest_id → 中文名` 硬编码表，扩展性差
  - **验收**：quest_id 映射走 `pages/characters/data/quests.json`，按当前语言 fallback en/zhCN/zhTW。
  - **测试**：vitest `quests-i18n.spec.tsx`
  - **锚点**：`pages/characters/CharacterDetails.tsx::CharacterQuests`
- ❌ 没有"未完成任务清单"快捷筛选
  - **验收**：TabBar chip `全部 / 已完成 / 未完成`，未完成项按 Act 排序。
  - **测试**：vitest `quests-filter.spec.tsx`
  - **锚点**：TBD

#### 1.3.7 奖励 Tab (`rewards`)

- ❌ w4 奖励消费状态没有历史趋势
  - **验收**：展示每个 NPC 的奖励数 + 跨会话持久化历史（SQLite）。
  - **测试**：vitest `rewards-history.spec.tsx`
  - **锚点**：`pages/characters/CharacterDetails.tsx::CharacterRewards`
- ❌ 没有"未消耗奖励"提示
  - **验收**：未消耗奖励项目右上角红点 chip，提醒"还有 N 项未领取"。
  - **测试**：vitest `rewards-unused-indicator.spec.tsx`
  - **锚点**：TBD

#### 1.3.8 佣兵 Tab (`merc`)

- ⚠️ 佣兵类型 / 雇佣时间 / 死亡状态未显示
  - **验收**：页面顶部增加 "Mercenary Type: <类型> · Hired <时间> · 状态: 存活/死亡"。
  - **测试**：vitest `merc-meta.spec.tsx`
  - **锚点**：`CharacterPanel.tsx:609-619`, `types.ts::CharacterInfo::merc_equipment`
- ❌ 没有"复活 / 重新雇佣"按钮
  - **验收**：死亡佣兵展示"已死亡"，按钮 "重新雇佣" 弹 confirm → 调 `resurrect_merc` 命令。
  - **测试**：vitest `merc-resurrect.spec.ts` + e2e
  - **锚点**：TBD（v3）

---

## 二、仓库 (`/warehouse`)

> 范围：**扩展仓库（`warehouse` collection in SQLite）**。从共享仓库存入、取回、删除、整理元数据。

### 2.1 Hero 区 + KPI

- 标签："仓库"
- KPI：`总库存 / 仓库件数（gold 强调）`

### 2.2 筛选条

5 个并列筛选项（横向 flex-wrap）：

- **搜索**（名称关键字）
- **角色**（来源角色，来自 `list_characters`）
- **部位**（12 装备槽位名复用 `SLOT_LABEL`）
- **类型**：`rune / gem / jewel / essence / key / armor / weapon / shield / misc`（中文 KIND_LABEL）
- **品质**：`unique / set / rare / magic / superior / normal`
- 有任意筛选条件时显示"清除过滤"按钮。

### 2.3 分组列表

- 按 `item_kind` 分组，按固定 `KIND_ORDER` 排；其余类型追加在尾。
- 同组按品质降序：`unique > set > rare > magic > superior > normal`。
- 行项：图标 + 名称 + 品质 + 收藏页名 + 标签 + low-dur 红底。
- 行操作按钮（hover 时全显示）：
  - **整理**：唤出元数据编辑表单（收藏页名/标签/备注），保存 `warehouse_update_meta`。
  - **取回**：调 `warehouse_withdraw`，弹"未找到游戏仓库文件"错误提示。
  - **删除**：弹 `DConfirmModal` 二次确认 + danger 红字"不可撤销"。

### 2.4 数据命令（详见附录 A）

`warehouse_search` / `warehouse_update_meta` / `warehouse_withdraw` / `warehouse_remove`

### 2.5 需求点 / 缺口

- ⚠️ **P0** · 取回时未支持选页/选位（强制 `pageIndex=0, positionX=0, positionY=0`）
  - **验收**：
    1. 取回前弹出"目标页 + 坐标"选择器
    2. 默认选中"目标 d2i 第一个空位"，并提示"X 件物品将放到 page N (x,y)"
    3. 用户确认后写入 d2i，UI 立即清掉扩展仓库行
  - **测试**：vitest `warehouse-withdraw-position.spec.tsx`（mock d2i 的空位算法）
  - **锚点**：`Warehouse.tsx:119-132`, `commands/warehouse.rs::warehouse_withdraw`
- ⚠️ **P0** · `loadWh` 静默 catch + warning toast
  - **验收**：`warehouse_search` 失败时升级为 `error` toast + 行项保持空态 + 不阻塞其他模块。
  - **测试**：vitest `warehouse-error-toast.spec.tsx`（mock tauriInvoke reject）
  - **锚点**：`Warehouse.tsx:75-87`
- ⚠️ 行项图标硬编码为 SVG `?`，没有走 `resolveItemIcon`
  - **验收**：所有 warehouse 行图标用 `resolveItemIcon(item)`，onerror 时回落 SVG `?`。
  - **测试**：vitest `warehouse-item-icon.spec.tsx`
  - **锚点**：`Warehouse.tsx:307-320`, `utils/itemImages.ts`
- ⚠️ `Warehouse.tsx:285-296` low-dur 触发需补 `durability_max=0` 零除防御
  - **验收**：max=0 不抛 NaN；max>0 时 `(cur/max) < 0.1` 才标红。
  - **测试**：vitest `warehouse-dur-zerodiv.spec.tsx`
  - **锚点**：`Warehouse.tsx:294-296`
- ❌ 未实现"批量取回"
  - **验收**：行项前加 checkbox，多选后顶部出现"批量取回 N 件"按钮，弹出汇总 confirm。
  - **测试**：vitest `warehouse-batch-withdraw.spec.tsx`
  - **锚点**：TBD
- ❌ 未实现"按日期排序"
  - **验收**：列表 header 增加 `入库时间` 列，可点击 toggle 升/降序，URL query 可持久化。
  - **测试**：vitest `warehouse-sort-date.spec.tsx`
  - **锚点**：TBD
- ❌ 收藏页名无下拉（手动输入易拼错）
  - **验收**：编辑表单 `page_name` 字段为 datalist，从已有 `page_name` 聚合成选项。
  - **测试**：vitest `warehouse-page-datalist.spec.tsx`
  - **锚点**：`Warehouse.tsx:349-352`
- ❌ 导出/导入收藏（`.json` / `.csv`）缺失
  - **验收**："导出"下载全部 WarehouseItem JSON；"导入"校验 schema 失败回退 + 显示差异条目。
  - **测试**：vitest `warehouse-export-import.spec.tsx`
  - **锚点**：TBD
- ❌ 持久化"低耐久提醒"缺失
  - **验收**：低耐久物品 SQLite flag `notified_low_dur=true`，下一次启动不再重复 toast。
  - **测试**：vitest `warehouse-low-dur-persist.spec.ts`
  - **锚点**：TBD

---

## 三、共享仓库 (`/stash`)

> 范围：**d2i 文件（共享仓库）只读视图 + 切页 + 类目过滤 + 上架/入仓入口**。
> 数据源：`read_stash` Tauri 命令 → `StashResult { pages[], items[] }`。

### 3.1 Hero 区

- 标签：共享仓库
- 提示：仅支持可堆叠物品。
- KPI 4 项：物品总数 / 已上架 / 当前页 / 可上架类目。
- 文件名小 chip：`stash_file.split(/[\\/]/).pop()`。

### 3.2 页签切换

- 当 `pages.length > 0` 显示 `TabBar`（页标签 + count + 类型提示"高级页·堆叠物品 / 装备页"）。
- 仅 ≥ 2 页时切页有意义。

### 3.3 类目过滤

11 个 sub-tab：`全部 / 符文 / 宝石 / 药水 / 钥匙 / 精华 / 碎片 / 护符 / 珠宝 / 装备 / 杂项`。
分类规则在 `getItemCategory(item)`（按 `code` 前缀 + `kind`）。
类目旁显示数量徽标 `(N)`。

### 3.4 物品卡片网格

- 自适应 2/3/4 列（`grid-cols-2 sm:3 md:4`）。
- 卡内：图标（40×40 fallback）× 品质色 × 数量徽标 × name + quality · code。
- 双按钮：
  - **上架** → 弹 `SellModal`
  - **存入仓库** → 弹 `DConfirmModal` 二次确认
- Hover `ItemTooltip`（含 `socketed_items`）。

### 3.5 上架 Modal (`SellModal`)

- 字段：数量（min 1 / max item.quantity）、单价（min 1）、参考建议售价按钮 → `get_price_suggestion`。
- 校验：整数 + 范围 + 价格有效 → `validationError` 红字 + aria-live。
- 提交 → `list_item(stashFile, itemName, itemCode, itemKind, quantity, unitPrice)`。
- focus-trap + `useFocusTrap`。

### 3.6 存入仓库 Modal (`DConfirmModal`)

- 物品 ×数量 + 提示"物品将从共享仓库转移到扩展仓"。
- 确认 → `warehouse_deposit({ stashPath, itemCode, pageIndex, quantity })`。

### 3.7 需求点 / 缺口

- ⚠️ **P0** · 没有装备上平台：mod stash 100+ 件装备没有二级筛选
  - **验收**：装备类目下增加二级筛选 chip：`品质 / 部位 / 底材类型`，实时过滤。
  - **测试**：vitest `stash-equipment-filter.spec.tsx`
  - **锚点**：`Inventory.tsx:184-201`
- ⚠️ **P0** · 没有"批量上架/入仓"
  - **验收**：行项前加 checkbox，多选后顶部出现"批量上架 N 件" + "批量入仓 N 件" 按钮，弹汇总 confirm。
  - **测试**：vitest `stash-batch-actions.spec.tsx`
  - **锚点**：TBD（v3）
- ⚠️ `SellModal` 单价上限没有参考
  - **验收**：单价 input 旁显示 `get_price_suggestion.max_price`，超过则黄字提示"超过建议 1.5x"。
  - **测试**：vitest `sell-modal-max-price.spec.tsx`
  - **锚点**：`SellModal.tsx:60-73`, `types.ts::PriceSuggestion`
- ⚠️ `Inventory.tsx` hooks 命名仍用 `useToast`/`refreshBalance`，与"共享仓库"语义模糊
  - **验证**：rename file-level variables/deps 不改外部。
  - **锚点**：`Inventory.tsx:53`
- ❌ 没有"按 code / 属性搜索"
  - **验收**：顶部 search input 支持 code 精确匹配 + 词缀关键字模糊匹配（如 `搜索 +life`）。
  - **测试**：vitest `stash-search.spec.tsx`
  - **锚点**：TBD
- ❌ 没有"按 socketed 物品过滤"
  - **验收**：filter chip 含 `镶嵌符文` / `镶嵌宝石` / `镶嵌珠宝`，多选。
  - **测试**：vitest `stash-socketed-filter.spec.tsx`
  - **锚点**：TBD
- ❌ 没有"按 modifier 词缀过滤"
  - **验收**：词缀字典下拉选择（如 `+1 暴风雪`），匹配装备带此词缀的展示。
  - **测试**：vitest `stash-affix-filter.spec.tsx`
  - **锚点**：TBD
- ❌ 没有"自动下架"入口
  - **验收**：行项右键菜单增加"下架"按钮，调用 `cancel_listing(itemId)`。
  - **测试**：vitest `stash-cancel-listing.spec.ts`
  - **锚点**：TBD
- ❌ 没有"我已上架物品"过滤视图
  - **验收**：顶部 toggle `全部 / 已上架`，已上架物品灰色 badge `已上架 x 件`。
  - **测试**：vitest `stash-listed-filter.spec.tsx`
  - **锚点**：TBD
- ❌ d2i 多 page 大量物品下没有虚拟滚动
  - **验收**：>500 物品时启用虚拟滚动（react-window 或自实现），首屏渲染 < 200ms。
  - **测试**：playwright `stash-large-perf.spec.ts`（5000 items）
  - **锚点**：TBD
- ❌ 模态弹窗（共享仓库移动到触屏不可达）
  - **验收**：触屏断点 (\<= 768px) 显示底部固定 bar，按钮始终可见。
  - **测试**：playwright `stash-mobile-sticky-bar.spec.ts`
  - **锚点**：`Inventory.tsx:127-153`

---

## 四、符文计算 (`/runeword`)

> 范围：**全量符文之语数据库查询 + 当前持有符文匹配 + 推荐**。

### 4.1 双列布局（左：选择+推荐 / 右：筛选+结果）

#### 4.1.1 左上：符文之语计算器

- **3 个 tier 组**：`低级(r01-r10) / 中级(r11-r20) / 高级(r21-r33)`。
- 每颗符文 52×52 按钮，hover 高亮金边，支持 `pointerdown + drag enter` 多选 / `pointerdown + click` 单选切换。
- 状态栏：`已选 N 个符文` + `从仓库加载`（`loadContext` 同步字符角色 d2s + d2i + invoke 后端拿底材，合并去重排序）+ `清空全部`。

#### 4.1.2 左下：推荐轮播

- 取 `filteredResults` 前 6 → 偏好 `hasMatchingBase` 的前 3，缺则补无底材。
- 3 张卡轮播（4s auto-rotate，hover/leave 暂停）。
- 3 种 tier（按"符文齐全 + 有底材"组合）：
  - **🟡 金 · 可制作**：符文全 + 有底材
  - **🔵 蓝 · 有底材**：缺符文
  - **🟢 绿 · 符文齐全**：缺底材
  - **⚫ 暗**：都没有
- 4★+ 顶级符文之语左上 ★ 浮标。

#### 4.1.3 右上：筛选

5 大筛选项：

- **名称**（zh/en 模糊匹配）
- **分类**：4 phase 标签：`开荒过渡 / 开荒必备 / 后期热门 / 然并卵`（来自 `runewordMeta.json::phase`）
- **最低星级**：1~5 颗 `★/☆` 按钮
- **孔数**：当前结果可选孔数自动列出
- **底材**：盾牌/武器/护甲/远程/近战/爪/长柄/矛/法杖/头盔/斧/权杖/锤/钉锤/棒/剑/匕首/圣骑盾/死灵法杖（`BASE_TYPE_OPTIONS`）
- **必须包含**：在所选符文里二次挑选（chips，可叠多选）

#### 4.1.4 右下：结果网格

- `grid-template-columns: repeat(auto-fill, minmax(300px, 1fr))` 自适应。
- 每张卡同 4 色背景规则，显示中文名 + 阶段 label + 英文名 + 星级 + 等级 + 孔数 + 符文序列 + 底材 code + 推荐底材 + 过渡 + 满变 + 备注 + 词缀翻译。

### 4.2 默认加载策略

- 启动立即同步读 localStorage 全量缓存 → 立即渲染（zero-latency）。
- 后台 `find_runewords({ ownedRunes: 全33符文 })` 刷新 + 持久化。
- 同时拉 `get_runeword_context`（同步符文 + 底材 code 集合），更新 `socketedTypes`。

### 4.3 推荐 tier 排序

- 与持有全集对比：
  - tier 3（🟡）> tier 2（🟢）> tier 1（🔵）> tier 0（⚫）
  - 同 tier 内按 `stars` 降序
- "从仓库加载" 时同步从角色 d2s 缓存 + d2i 缓存 + Rust 端底材合并 `owned_runes`。

### 4.4 数据/命令（详见附录 A）

`find_runewords` / `get_runeword_context` / `get_localized_skill_texts` / `get_app_config`

### 4.5 需求点 / 缺口

- ⚠️ **P0 升** · 符文消耗统计
  - **验收**：玩家选 N 个符文后，顶部出现统计条 `"你这 N 个符文可用于 X 个符文之语，覆盖 Y 个练级阶段"`，并列出"哪些符文之语将因你缺 1 颗卡住"。
  - **测试**：vitest `runeword-consumption-stat.spec.tsx`
  - **锚点**：`RunewordCalc.tsx:486-545`（在 `filteredResults` 之后插 summary）
- ⚠️ **P0 升** · 底材 quality 过滤
  - **验收**：底材类型 chip 旁增加 `品质` 子 chip：`白板 / 优秀 / 无形 / 任意`，默认 `任意`。
  - **测试**：vitest `runeword-base-quality-filter.spec.tsx`
  - **锚点**：`RunewordCalc.tsx:587-593`（socket 之下扩展）
- ⚠️ **P1** · 符文位置追溯
  - **验收**：点击"未拥有"的 rune 按钮，弹 panel 列出"该 rune 出现在 N 个角色背包/M 个 d2i 页/P 件装备 socketed"，点击位置直跳。
  - **测试**：vitest `runeword-rune-location.spec.tsx`
  - **锚点**：TBD（依赖本地 cache + 角色 Tab）
- ⚠️ **P1** · stars ≥ 4 排序修正：Spirit 2★ 但最有价值
  - **验收**：stars ≥ 4 浮标仅作视觉；主排序仍按 `rec_best` 优先级，但 `recommend_rank` 列额外展示（数据库 `runewordMeta.json` 加字段）。
  - **测试**：vitest `runeword-rank-by-rec-best.spec.tsx`
  - **锚点**：`RunewordCalc.tsx:528-543`, `data/runewordMeta.json`
- ⚠️ **P1** · `loadContext` 报错后无回滚
  - **验收**：合并 merged 失败时保留前一份 `socketedTypes` 不变 + toast "加载失败，沿用上一次"。
  - **测试**：vitest `runeword-load-context-rollback.spec.tsx`
  - **锚点**：`RunewordCalc.tsx:419-455`
- ❌ 缺"职业筛选"——按 builds / 主流 BD 切出合适符文之语
  - **验收**：phase filter 旁增加"职业"chip：`所有 / 野蛮人 / 死灵 / ...`。
  - **测试**：vitest `runeword-class-filter.spec.tsx`
  - **锚点**：TBD
- ❌ 缺"按等级要求过滤"
  - **验收**：`req_lvl` slider 0-99，过滤后实时显示。
  - **测试**：vitest `runeword-req-lvl-filter.spec.tsx`
  - **锚点**：TBD
- ❌ 缺"购买来源"提示
  - **验收**：阶段 chip 旁显示"市场建议价" badge，hover 列出 3 个最近的 `ListedItem`。
  - **测试**：vitest `runeword-market-price-link.spec.tsx`
  - **锚点**：TBD
- ❌ 缺"收藏符文之语"功能
  - **验收**：卡右上角加"♡"按钮，收藏后写入 localStorage；顶部 TabBar 增加"我的收藏"视图。
  - **测试**：vitest `runeword-favorite.spec.tsx`
  - **锚点**：TBD
- ❌ 缺"装备对比预览"
  - **验收**：选 2 个符文之语 + 点"对比"，弹并列 modal，stat diff 红绿高亮。
  - **测试**：vitest `runeword-compare.spec.tsx`
  - **锚点**：TBD
- ❌ 缺"未拥有符文"位置显示
  - **测试**：vitest `runeword-rune-location.spec.tsx`
- ❌ 缺"DB 持久化筛选 preset"
  - **验收**：右上"保存预设"按钮，按名字存 SQLite，下次启动一键加载。
  - **测试**：vitest `runeword-preset-persist.spec.tsx`
  - **锚点**：TBD
- ⚠️ 多语种 `zhTW/enUS` 词缀未翻译（仅 zhCN 走 `translateAffixZh`）
  - **验收**：`zhTW / enUS` 下词缀列直接显示英文原文 + tooltip 显示中文兜底。
  - **测试**：vitest `runeword-affix-i18n.spec.tsx`
  - **锚点**：`RunewordCalc.tsx:742-752`
- ⚠️ `BASE_TYPE_CODES_TO_ZH` 缺 enUS 翻译
  - **验收**：`enUS` 模式展示英文底材名，zhCN 维持中文。
  - **测试**：vitest `runeword-base-type-i18n.spec.tsx`
  - **锚点**：`RunewordCalc.tsx:99-104`

---

## 五、跨菜单共性需求（横向）

| 模块 | 需求 | 影响菜单 | 验收锚点 |
|------|------|----------|----------|
| 性能 | 超大 stash（> 5000 件）虚拟滚动 | 共享仓库 | §3.7 项 |
| 性能 | d2s 加载三段进度精确化 | 角色 | §1.2 stage1/stage3 error toast |
| i18n | zhTW / enUS 缺译清单已审计 | 全部 | §4.5 + §1.3.4 fallback |
| 数据 | mod stash 解析覆盖率 | 共享仓库 | 跟踪 d2i-mod-stash-parser-replan memory |
| 数据 | 个人仓库 page=5 动态 size | 角色·背包 | §1.3.2 |
| 数据 | `equipmentSkillBonuses` 聚合 UI | 角色·装备 | §1.3.1 |
| 流程 | 角色死亡/被识别为 `is_dead` 时弹横幅 | 角色 | §1.2 过滤条件联动 |
| 流程 | 装备"未装备但可穿"提示（等级/属性不足） | 装备详情 | TBD |
| 流程 | 上架后立刻从 d2i 写入（CLAUDE.md Notes） | 共享仓库→市场 | TBD |
| 测试 | E2E：四种 tab 切换、外取件、内嵌进仓库、符文过滤都用 vitest | 全部 | 见附录 D |
| UI | `ItemTooltip` 嵌套多层 hover | 全部 | TBD |
| 合规 | Warlock 全身像走本地不入仓路径 | 角色 | §0.1 |

---

## 六、优先级矩阵 v2

### P0（按玩家真实使用频率排）

1. 角色 · 切换主副手（野蛮人 / 死灵 / 刺客 daily）
2. 装备 · 一键替换（切装高频）
3. 共享仓库 · 装备上平台（mod 装备页筛选）
4. 共享仓库 · 批量上架 / 入仓
5. 仓库 · 取回时选位
6. 仓库 · 错误改 error toast
7. 符文计算 · 符文消耗统计（防丢三连）
8. 符文计算 · 底材 quality 过滤（normal/superior/ethereal）

### P1

- 多语种技能名（zhCN/zhTW fix）
- 符文位置追溯
- 角色 stage1/stage3 错误 toast 整合
- 角色存档 hash 变更 toast
- 装备 Tab 装备对比（30 天 DAU 守卫：0 立即下线）
- 符文 stars ≥ 4 浮标 + Spirit 例外提示
- `loadContext` rollback
- 多语种词缀翻译

### P2

- 装备一键入箱 ↔ 任务自动同步
- 符文之语收藏预设 / 装备对比
- 符文计算 底材图例本地化
- d2emu hero editor 集成
- Warlock 全身像 CC0 替代方案
- 收藏 JSON/CSV 导入导出
- 角色右键菜单（重命名 / 删除 / 复制）

---

## 附录 A · Tauri 命令登记表

> 命名规范：`<verb>_<noun>`，动词统一第三人称单数（`list_*`、`get_*`、`extract_*`、`update_*`、`remove_*`）。

| Command | 用途 | 签名 | 性能目标 | 失败行为 |
|---------|------|------|---------|---------|
| `list_characters_brief` | 列角色 + 轻量数据 | `({ dir })` | < 50ms | 空列表 |
| `load_character_background` | 异步加载 d2s | `({ path })` | stage1 < 100ms, stage3 < 2s | event `char:error` + toast |
| `read_stash` | 读 d2i | `({})` | < 1s（5000 件） | 空 stash + toast |
| `extract_character_equipment` | 全身装备入库 | `({ path })` | < 200ms | toast |
| `list_item` | 上架（不写 d2i） | `({ stashFile, itemName, itemCode, itemKind, quantity, unitPrice })` | < 200ms | toast + 保留 stock |
| `cancel_listing` | 撤销上架 | `({ itemId })` | < 200ms | toast |
| `warehouse_deposit` | 入扩展仓库 | `({ stashPath, itemCode, pageIndex, quantity })` | < 300ms | toast |
| `warehouse_withdraw` | 取回到 d2i | `({ itemId, stashPath, pageIndex, positionX, positionY })` | < 300ms | toast + 失败保留原位 |
| `warehouse_search` | 多条件 search | `({ source_character?, item_kind?, equipment_slot?, quality?, search_text? })` | < 100ms | 空列表（v2 改 error toast） |
| `warehouse_remove` | 物理删 | `({ itemId })` | < 100ms | toast |
| `warehouse_update_meta` | 改 page_name/tags/notes | `({ itemId, pageName, tags, notes })` | < 100ms | toast |
| `find_runewords` | 匹配符文之语 | `({ ownedRunes: string[] })` | < 200ms | 空列表 |
| `get_runeword_context` | 当前底材集合 | `({})` | < 100ms | 空集合 |
| `get_localized_skill_texts` | 多语种技能名 | `({ language })` | < 100ms | 空对象 + 英文 fallback |
| `get_price_suggestion` | 建议售价 | `({ itemName, itemKind })` | < 100ms | fallback 到 1 + 隐藏按钮 |
| `get_app_config` | 全局配置 | `({})` | < 50ms | 默认 config |
| `get_balance` | 当前 token | `({})` | < 50ms | 0 |

**编排建议**：
- 凡涉及 d2s/d2i 写操作的命令（`extract_* / list_item / warehouse_withdraw`）必须**先备份**（参考 CLAUDE.md 备份策略），并支持 cancel。
- 所有命令返回值统一 `{ ok: boolean, data?, error? }`，前端 schema 化校验（zod）。

---

## 附录 B · localStorage Key 总账

> 跨页 stale 风险由 §B.2 治理。

### §B.1 Key 清单

| Key | 类型 | 写入方 | 失效策略 | stale 风险 |
|-----|------|--------|----------|------------|
| `d2r-last-character` | string | `Characters.tsx:14` | 玩家手动"切角色" | OK |
| `d2r-character-names` | string[] | `Characters.tsx:24, 92` | 应用启动 + 用户刷新按钮 | 短暂陈旧，无害 |
| `d2r-char-class-<name>` | `CharClassCache` | `Characters.tsx:99-108` | file_hash 不变 OK；否则 `dismissCharChanged` | hash 变更时残留旧 class 信息 |
| `d2r-char-full-<name>` | `CharacterInfo` JSON | `Characters.tsx:36-41, 220` | 手动刷新 + `clearCharCache` | **跨页 stale**（角色 / 符文计算共用） |
| `d2r-char-stash-<name>` | `StashResult` JSON | `CharacterPanel.tsx:278-281` | 显式 `forceRefresh` | **跨页 stale**（角色 + 共享仓库 + RunewordCalc 共用） |
| `runeword-cache-all` | `RW[]` | `RunewordCalc.tsx:268-285` | 用户刷新 | OK（≥ 10k runes 才刷） |
| `runeword-context-cache` | `RunewordContext` | `RunewordCalc.tsx:274-278` | `clearRunewordContextCache()` 由角色切换触发 | OK |
| `d2r-import-status` | `ImportStatus[]` | `hooks/useImportProgress` | 任务完成自动清 | OK |

### §B.2 跨页 stale 治理（统一方案）

- 凡跨页用的 full cache（`d2r-char-full-<name>`, `d2r-char-stash-<name>`）一律加 `imported_at` timestamp。
- 切回页面 / 切角色时若 `now - imported_at > 60s`，自动 forceRefresh。
- 此外提供"立即刷新"按钮，方便手动 invalidate。

### §B.3 跨菜单 staleness 触发矩阵

| Triggering Action | 失效 key | 实现方式 |
|------|------|------|
| 进入共享仓库 tab | `d2r-char-stash-<name>` | tab 切换监听 |
| d2s 写入成功 | `d2r-char-full-<name>`, `d2r-char-class-<name>` | 命令返回成功回执 |
| d2i 写入成功 | `d2r-char-stash-<name>` | 同上 |
| 角色切换 | `runeword-context-cache` | `clearRunewordContextCache()` |
| `extract_character_equipment` | `d2r-char-stash-<name>`, `d2r-char-full-<name>` | ok 回执 |

---

## 附录 C · 数据流图占位

```
                    [ D2R 玩家账号 ]
                           │
                  ┌────────┴─────────┐
                  ▼                  ▼
       [save/<name>.d2s]      [save/<name>.d2i]
                  │                  │
                  │ Tauri invoke     │
                  ▼                  ▼
       ┌────────────────────────────────────┐
       │       src-tauri/                   │
       │  ┌──────────────────────────────┐  │
       │  │ protocol/d2s/parser.rs       │  │
       │  │  ├─ header.rs                │  │
       │  │  ├─ attributes.rs            │  │
       │  │  └─ items.rs (parse_items)   │  │
       │  └──────────────────────────────┘  │
       │  ┌──────────────────────────────┐  │
       │  │ protocol/d2i/parser.rs       │  │
       │  │  ├─ page_header.rs           │  │
       │  │  ├─ page.rs                  │  │
       │  │  └─ legacy/item.rs (complete)│  │
       │  └──────────────────────────────┘  │
       │  ┌──────────────────────────────┐  │
       │  │ commands/*.rs (Tauri IPC)    │  │
       │  └──────────────────────────────┘  │
       └────────────────────────────────────┘
                          │ JSON projection
                          ▼
                ┌──────────────────────┐
                │ web/src/types.ts     │
                │  - CharacterInfo     │
                │  - StashResult       │
                │  - WarehouseItem     │
                │  - ListedItem        │
                │  - SkillBonus        │
                └──────────────────────┘
                          │ React state
                          ▼
   ┌──────────────────────────────────────────┐
   │ web/src/pages/{Characters|Warehouse|     │
   │ Inventory|Config|Catalog|Listings|       │
   │ History|RunewordCalc|Grail|Builds|Home}  │
   │ web/src/components/{CharacterPanel|      │
   │ EquipmentPanel|EquipmentDetailModal|     │
   │ InventoryView|SkillTree|SkillDetails…}   │
   └──────────────────────────────────────────┘
                          │ Tauri invoke (write)
                          ▼
                ┌──────────────────────┐
                │ commands/*.rs        │
                │  (write d2s/d2i/     │
                │   SQLite)            │
                └──────────────────────┘
                          │
                  ┌───────┴────────┐
                  ▼                ▼
         [save/<name>.d2s]  [save/<name>.d2i]
              (+ .d2s.bak + .d2i.bak 备份)
```

**关键观察**：

1. **只读投影**：parser 只输出 JSON，不持有内部状态，跨页 stale 由 caller 自己管（→ 附录 B）。
2. **写入对称**：命令返回 `{ ok, data?, error? }`；前端在收到 `ok=true` 后才更新本地 cache。
3. **备份强制**：所有写命令实现层必须先 `cp <save> <save>.bak.<ts>`，失败时回滚（参考 CLAUDE.md 备份策略）。

---

## 附录 D · 验收与测试模板

每条 P0 / P1 三件套标准：

```yaml
需求: <一句话>
验收:
  - 可观察信号 1
  - 可观察信号 2
  - 可观察信号 3
测试:
  - vitest:<file>.spec.ts
  - 或 playwright:e2e/<file>.spec.ts
锚点:
  - file:line
  - file:line
```

**测试目录约定**：
- 单测：`web/tests/unit/<domain>/<file>.spec.ts`
- 组件测：`web/tests/components/<file>.spec.tsx`
- E2E：`web/tests/e2e/<flow>.spec.ts`
- 集成（前端 + Tauri mock）：`web/tests/integration/<cmd>.spec.ts`

**回归门槛**：任意 P0 完成必须带动 ≥2 个测试 add，否则 PR 不收。

---

## 附录 E · 引用文件清单

### 一级路由

- `web/src/App.tsx` :: `NAV[0..11]`、`Routes`

### 一级页面

- `web/src/pages/Characters.tsx`
- `web/src/pages/Warehouse.tsx`
- `web/src/pages/Inventory.tsx`（"共享仓库"）
- `web/src/pages/RunewordCalc.tsx`

### UI 组件

- `web/src/components/CharacterPanel.tsx`
- `web/src/components/EquipmentPanel.tsx`
- `web/src/components/EquipmentDetailModal.tsx`
- `web/src/components/InventoryView.tsx`
- `web/src/components/SkillTree.tsx`
- `web/src/components/SkillDetailsPanel.tsx`
- `web/src/components/SellModal.tsx`
- `web/src/components/DConfirmModal.tsx`
- `web/src/components/EmptyState.tsx`

### 类型 / 桥接

- `web/src/types.ts`
- `web/src/tauri.ts`
- `web/src/utils/itemImages.ts`
- `web/src/utils/runewordCache.ts`
- `web/src/utils/skillPresentation.ts`

### 数据（本地 / 静态）

- `web/src/data/runewordMeta.json`
- `web/src/data/skills.ts`
- `web/src/data/itemNames.ts`
- `web/src/data/skillTreeLayouts.ts`

### Rust 后端（src-tauri）

- `src-tauri/src/commands/character.rs`
- `src-tauri/src/commands/warehouse.rs`
- `src-tauri/src/commands/marketplace.rs`
- `src-tauri/src/protocol/d2s/parser.rs`
- `src-tauri/src/protocol/d2i/parser.rs`

---

## 版本与变更日志

| 版本 | 日期 | 变更摘要 |
|------|------|---------|
| v1 | 2026-07-20 | 初版：从代码盘点 4 个一级菜单所有界面功能 |
| v2 | 2026-07-20 | 多角色评审迭代：加 §0 版权边界 + 附录 A/B/C/D + 每条 P0 三件套 + 优先级重排 |
