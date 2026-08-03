# 装备管理器 + Build 推荐系统 — 需求文档与技术方案（精简版）

> 写入 `.d2s` 延后，本阶段只做**读取 + 推荐**

---

## 一、需求文档

### 1.1 业务目标

让单机 D2R 玩家能：
1. **查看角色装备** → 一键存入 SQLite 统一管理（原始 `.d2s` 不变）
2. **跨角色装备库** — 同一账号下多个角色的装备互通查看
3. **Build 推荐** — 根据仓库 + 角色当前装备，自动推荐最优 Build 和缺口分析

### 1.2 用户故事

```
作为一名玩多个角色的玩家，
我想要看到我所有角色装备的统一视图，
以便快速定位某件装备在哪个角色身上。

作为一名想换 Build 的玩家，
我想要系统根据我已有的装备推荐适合的 Build，
并告诉我还需要刷什么。
```

### 1.3 功能清单

#### P0 — 核心流程
- [x] **角色 `.d2s` 解析** — 已有：装备/背包/腰带/佣兵完全读取
- [ ] **角色装备→SQLite**：新增 `extract_character_equipment` 命令，解析 `.d2s` 后把装备/背包/腰带物品写入 `warehouse_items` 表（关联来源角色名）
- [ ] **仓库按角色筛选**：前端仓库页面可过滤"来自角色 XXX 的装备"
- [ ] **角色装备快照**：每次提取保留时间戳，可回溯历史装备

#### P1 — Build 推荐
- [ ] **Build 知识库**：JSON 定义 30+ 经典 Build，含核心装备/可选装备/符文之语/技能框架
- [ ] **匹配引擎**：遍历 SQLite 仓库 + 角色当前装备 → 计算每个 Build 完成度 %
- [ ] **缺口分析**：未拥有的核心装备 → "需要刷 / 可制作" 建议
- [ ] **推荐面板 UI**：Build 卡片列表（按完成度排序），点击展开详情

#### P2 — 增强（延后）
- [ ] 写入 `.d2s` 清空背包
- [ ] 从仓库恢复装备到角色
- [ ] 装备对比
- [ ] Build 模拟装配

---

## 二、技术方案

### 2.1 架构变更

```
新增模块:
├── services/
│   └── build_service.rs          ← Build 匹配引擎
│
├── data/builds/
│   ├── mod.rs                    ← Build 知识库加载器
│   ├── sorceress.json
│   ├── paladin.json
│   └── ...                       ← 每个职业一个 JSON
│
├── commands/
│   └── character.rs              ← 扩展：extract_character_equipment
│
├── web/src/
│   ├── pages/Builds.tsx           ← Build 推荐页面
│   ├── components/BuildCard.tsx   ← Build 卡片
│   └── components/GapAnalysis.tsx ← 缺口分析
│
└── database/
    └── repository/
        └── equipment_repo.rs      ← 扩展仓库查询：按角色/装备位筛选
```

### 2.2 数据模型扩展

#### WarehousedItem 新增 3 个字段（非破坏性）

```rust
pub struct WarehousedItem {
    // ... 现有 18 字段不变 ...

    // 新增 ↓
    pub source_character: Option<String>,  // 来源角色名 ("EchoingStrike")
    pub source_save_path: Option<String>,  // 来源 d2s 路径
    pub slot_equipped: Option<String>,     // 原装备位 ("helm"/"weapon_main"/...)
                                           // None = 来自背包或共享仓库
}
```

为什么只有 3 个字段：`item_code`、`item_name`、`quality`、`quantity`、`item_kind` 等已有字段可直接复用，不需要额外扩展。

### 2.3 核心流程

#### 流程一：角色装备 → SQLite（新增 `extract_character_equipment`）

```
用户点击"存入仓库"按钮
  │
  ├── 1. read_character_info(path)  // 已有 ✅
  │     返回: equipment[12], backpack_items[N], belt_items[M]
  │
  ├── 2. 遍历物品，转为 WarehousedItem
  │     for each item in equipment + backpack + belt:
  │       ├── 填充: item_code, item_name, quality, quantity = 1
  │       ├── 填充: source_character, source_save_path, slot_equipped
  │       └── 写入: WarehouseRepo::add()
  │
  └── 3. 返回提取结果 { extracted_count, warehouse_ids, skipped_items }
```

注意：**不修改 `.d2s` 文件**——角色背包保留原样。`source_character` 用于前端展示和区分。

#### 流程二：Build 推荐

```
用户打开 "Build 推荐" 页面
  │
  ├── 1. build_service::load_knowledge_base()
  │     加载 data/builds/*.json → Vec<BuildDefinition>
  │
  ├── 2. 收集用户装备
  │     ├── warehouse_list_all()         // SQLite 中所有装备
  │     └── read_character_info(path)    // 角色当前穿戴（可选）
  │
  ├── 3. build_service::match_builds(equipment, builds, class)
  │     for each build:
  │       ├── core_score (0~1) = 已有核心件 / 总核心件
  │       ├── opt_score  (0~1) = 已有可选件 / 总可选件
  │       ├── rw_score   (0~1) = 已有符文数 / 总符文数
  │       ├── total = 0.6*core + 0.3*opt + 0.1*rw
  │       └── gap: 列出缺失的核心件 + 获取途径建议
  │     → 按 total 降序
  │
  └── 4. 前端渲染
```

### 2.4 接口设计

#### 新增 3 个 Tauri 命令

```rust
/// 提取角色装备到仓库（不修改 .d2s）
#[tauri::command]
fn extract_character_equipment(
    state: State<AppState>,
    path: String,                            // d2s 文件路径
    include_backpack: Option<bool>,          // 默认 true
    include_equipped: Option<bool>,          // 默认 true
    include_belt: Option<bool>,              // 默认 false（药水通常不需要存）
) -> Result<ExtractResult, String>;

/// 获取 Build 推荐
#[tauri::command]
fn get_build_recommendations(
    state: State<AppState>,
    character_path: Option<String>,  // 指定角色可更精准
    class_filter: Option<String>,    // 可选：只推荐某个职业
) -> Result<Vec<BuildRecommendation>, String>;

/// 获取单个 Build 详情
#[tauri::command]
fn get_build_detail(
    state: State<AppState>,
    build_id: String,
    character_path: Option<String>,
) -> Result<BuildDetail, String>;
```

#### 新响应类型

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractResult {
    pub extracted_count: usize,
    pub warehouse_ids: Vec<String>,
    pub source_character: String,
    pub equipped_count: usize,
    pub backpack_count: usize,
    pub belt_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildRecommendation {
    pub build_id: String,
    pub class: String,
    pub name: String,
    pub name_zh: String,
    pub match_score: f64,               // 0.0 ~ 1.0
    pub core_owned: usize,
    pub core_total: usize,
    pub owned_items: Vec<OwnedItemRef>, // 已拥有的物品
    pub missing_core: Vec<MissingItem>, // 缺失核心件
    pub missing_runewords: Vec<MissingRuneword>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnedItemRef {
    pub warehouse_id: String,
    pub name: String,
    pub slot: String,
    pub source_character: Option<String>, // 在哪个角色身上
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissingItem {
    pub slot: String,
    pub name: String,
    pub code: String,
    pub weight: f64,
    pub acquisition: String,  // "boss: Mephisto" | "area: Chaos Sanctuary" | "cube: craft"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissingRuneword {
    pub name: String,
    pub runes: Vec<String>,
    pub owned_runes: Vec<String>,
    pub missing_runes: Vec<String>,
    pub socket_base: String,
}
```

### 2.5 匹配引擎设计

匹配算法尽可能简单（不解析 stat，只做 code 匹配）：

```rust
fn match_build(
    warehouse: &[WarehousedItem],
    build: &BuildDefinition,
) -> BuildMatch {
    let mut owned_core = 0;
    let mut missing_core = vec![];

    for req in &build.equipment.core {
        // 检查仓库是否有该物品
        let found = warehouse.iter().any(|w| w.item_code == req.code);
        // 也检查角色当前是否装备了
        // let found = found || current_equip.iter().any(|e| e.code == req.code);

        if found {
            owned_core += 1;
        } else {
            missing_core.push(MissingItem { ... });
        }
    }
    // ... 同理处理 optional 和 runewords ...

    let score = owned_core as f64 / build.equipment.core.len() as f64;
    BuildMatch { score, owned_core, missing_core, ... }
}
```

仓库查询接口扩展（`EquipmentRepo`）：
```rust
pub trait EquipmentRepository {
    fn list_by_character(&self, character_name: &str) -> Result<Vec<WarehousedItem>, String>;
    fn list_equipped(&self) -> Result<Vec<WarehousedItem>, String>;  // slot_equipped IS NOT NULL
    fn list_by_character_and_slot(&self, character: &str, slot: &str) -> Result<Vec<WarehousedItem>, String>;
}
```

---

## 三、开发计划

### Phase 1 — 装备提取（3 天）

| 任务 | 文件 | 工作量 |
|------|------|--------|
| `WarehousedItem` 扩展 3 字段 + migration | `database/models.rs`, `db.rs` | 0.5d |
| 装备转换：`CharacterInfoResult` → `Vec<WarehousedItem>` | `services/character_service.rs` | 1d |
| `extract_character_equipment` command + 去重逻辑 | `commands/character.rs` | 1d |
| 前端"存入仓库"按钮（角色面板/背包 tab） | `components/CharacterPanel.tsx` | 0.5d |

### Phase 2 — 仓库按角色筛选（1 天）

| 任务 | 文件 | 工作量 |
|------|------|--------|
| `warehouse_list_by_character` 查询 + IPC | `commands/warehouse.rs` + `repository/equipment_repo.rs` | 0.5d |
| 仓库页面新增"来源角色"下拉筛选项 | `pages/Warehouse.tsx` | 0.5d |

### Phase 3 — Build 知识库 + 匹配引擎（4 天）

| 任务 | 文件 | 工作量 |
|------|------|--------|
| Build 知识库目录 + JSON 加载器 | `data/builds/mod.rs` | 0.5d |
| 撰写 21 个 Build 定义 JSON（7 职业 × 3 流派） | `data/builds/*.json` | 2d |
| 匹配引擎 — 核心/可选/符文匹配 | `services/build_service.rs` | 1d |
| `get_build_recommendations` + `get_build_detail` command | `commands/build.rs` | 0.5d |

### Phase 4 — Build 推荐 UI（2 天）

| 任务 | 文件 | 工作量 |
|------|------|--------|
| Build 推荐页面 + 导航注册 | `pages/Builds.tsx`, `App.tsx` | 0.5d |
| Build 卡片组件（完成度进度条 + 核心件列表） | `components/BuildCard.tsx` | 0.5d |
| 缺口分析组件（缺失装备 + 获取途径） | `components/GapAnalysis.tsx` | 0.5d |
| Build 详情弹窗（完整装备列表 + 技能参考） | `components/BuildDetailModal.tsx` | 0.5d |

### 时间线

```
Week 1: Phase 1 (3d)     → 角色装备可存入 SQLite
        Phase 2 (1d)     → 仓库可按角色筛选
Week 2: Phase 3 (4d)     → Build 知识库 + 匹配引擎
Week 3: Phase 4 (2d)     → Build 推荐 UI
         Buffer  (1d)    → 集成测试 + 微调
────────────────────────────────
Total:           11d
```

### 依赖关系

```
Phase 1 ──→ Phase 2 ──┐
                       ├──→ 两路独立，可并行
Phase 3 ──────────────┘
         └── Phase 4 (依赖 Phase 1+3)
```
