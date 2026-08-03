# D2R Marketplace — Tauri (Rust) Port

暗黑破坏神 II：重制版 离线单机拍卖行

> **Rust + Tauri v2 重构版**，原项目为 Python/Flask 实现。  
> 原项目地址：`../d2r-marketplace/`

---

## 概览

D2R Marketplace 是一个运行在本地的离线拍卖行系统，专为《暗黑破坏神 II：重制版》单机存档设计。它直接读写游戏共享仓库（`.d2i`）文件，让你无需联网即可通过代币经济系统上架、买卖物品。

### 核心功能

- 📦 **共享仓库查看** — 解析 D2R `.d2i` 格式，读取共享仓库所有堆叠物品
- 🛒 **符文市场** — 直接用代币购买符文，写入共享仓库
- 📤 **物品上架** — 从仓库提取物品上架，设置价格，自动计时出售
- 🔄 **自动出售** — 到期自动完成交易，代币自动到账
- ❌ **取消上架** — 未出售前随时取消，物品自动归还仓库
- ⚙️ **游戏配置** — 支持原版/模组存档路径、多语言物品名称加载
- 💾 **自动备份** — 修改存档前自动备份，防止损坏

---

## 架构

```
┌──────────────────────────────────────────────────────────────┐
│                    Tauri v2 Shell (WebView)                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              React 前端 (web/src/)                   │   │
│  │  Home │ Inventory │ Catalog │ Listings │ Config      │   │
│  │          Tailwind CSS v4 + D2 主题                   │   │
│  └──────────────┬───────────────────────────────────────┘   │
│                 │ Tauri IPC (@tauri-apps/api)               │
│  ┌──────────────▼───────────────────────────────────────┐   │
│  │              Rust 后端 (src-tauri/src/)              │   │
│  │  ┌─────────┐ ┌──────────┐ ┌────────────────────┐   │   │
│  │  │commands │ │ database │ │  core/             │   │   │
│  │  │ (IPC)   │ │ (SQLite) │ │  BitReader/Writer  │   │   │
│  │  │         │ │  models  │ │  Huffman           │   │   │
│  │  │         │ │  CRUD    │ │  ParseError/Result │   │   │
│  │  │         │ │          │ │  ProtocolVersion   │   │   │
│  │  │         │ │          │ └────────────────────┘   │   │
│  │  │         │ │          │ ┌────────────────────┐   │   │
│  │  │         │ │          │ │  protocol/         │   │   │
│  │  │         │ │          │ │  common/           │   │   │
│  │  │         │ │          │ │   Item/Flags/Mode  │   │   │
│  │  │         │ │          │ │   Page ★3b fix     │   │   │
│  │  │         │ │          │ │   Quality/Location │   │   │
│  │  │         │ │          │ │   StatList/Stat    │   │   │
│  │  │         │ │          │ │   FieldSet         │   │   │
│  │  │         │ │          │ │  d2i/              │   │   │
│  │  │         │ │          │ │   parser + summary │   │   │
│  │  │         │ │          │ │   legacy/ (compat) │   │   │
│  │  │         │ │          │ │  d2s/ (角色存档)  │   │   │
│  │  │         │ │          │ └────────────────────┘   │   │
│  │  │         │ │          │ ┌────────────────────┐   │   │
│  │  │         │ │          │ │  data/             │   │   │
│  │  │         │ │          │ │  stat_cost 表      │   │   │
│  │  │         │ │          │ │  items 常量        │   │   │
│  │  │         │ │          │ └────────────────────┘   │   │
│  │  └────┬────┘ └──────────┘ ┌──────────┐               │   │
│  │       └─────────────────────┤ market   │               │   │
│  │                             │ pricing  │               │   │
│  │                             │sell_time │               │   │
│  │                             │trade_rls │               │   │
│  │                             └──────────┘               │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### 数据流

1. **读取仓库**：设置存档路径 → `read_stash` 命令 → Rust 原生解析器解析 `.d2i` 堆叠页 → 返回物品列表到前端
2. **上架物品**：仓库选择物品 → 设置价格 → `list_item` 命令 → 从 `.d2i` 扣除实物 → 数据库记录上架信息（含自动出售计时器）
3. **购买符文**：市场选择符文 → `buy_item` 命令 → 检查代币余额 → 写入 `.d2i` 文件 → 扣除代币 → 记录交易
4. **自动出售**：后端 `process_due_listings()` 检查所有活跃上架 → 超时的自动标记已售 → 代币入账
5. **取消上架**：`cancel_listing` 命令 → 将物品写回 `.d2i` → 标记已取消

---

## 路线图

### 玩家物品交换（规划中）

通过加密密钥体系实现玩家间点对点物品交换：

1. **密钥生成** — 每位玩家在本地生成非对称密钥对（公钥/私钥），私钥自行保管
2. **公钥交换** — 通过任意渠道（社交媒体/即时通讯/密钥服务器）交换公钥
3. **物品打包加密** — 发送方用接收方公钥加密物品数据，生成可校验的交换包
4. **拉取** — 接收方导入交换包，用私钥解密，物品进入本地仓库

技术选型建议：X25519 密钥交换 + ChaCha20-Poly1305 加密 + Ed25519 签名，数据库用 SQLite（或 SQLite WAL 支持网络复制）。

---

## 已知限制

- **装备物品仅支持查看**：武器/防具/首饰等非堆叠物品目前仅可查看 tooltip，无法存入收藏库或上架交易
- **部分装备可能解析/存入失败**：边界情况（如稀有前缀名称、特殊模组格式、嵌物等）可能在读或写时报错；建议操作前备份存档
- **仅支持 Windows**：Tauri v2 + Windows 构建，未测试 macOS/Linux
- **单机离线**：不包含任何网络功能，无法与其他玩家交互

---

## 技术栈

### Rust 后端 (src-tauri/)

| 依赖 | 版本 | 用途 |
|------|------|------|
| [Tauri](https://tauri.app/) | 2.x | 桌面应用框架（含 tray-icon 特性） |
| [tauri-plugin-shell](https://github.com/tauri-apps/plugins-workspace) | 2.x | Shell 命令调用 |
| [tauri-plugin-dialog](https://github.com/tauri-apps/plugins-workspace) | 2.x | 原生文件夹选择对话框 |
| [tauri-plugin-fs](https://github.com/tauri-apps/plugins-workspace) | 2.x | 文件系统访问 |
| [rusqlite](https://github.com/rusqlite/rusqlite) | 0.34 | SQLite 数据库（bundled 模式） |
| [serde](https://github.com/serde-rs/serde) | 1.x | 序列化/反序列化 |
| [serde_json](https://github.com/serde-rs/json) | 1.x | JSON 处理 |
| [bitvec](https://github.com/bitvecto-rs/bitvec) | 1.x | 位级 I/O（`.d2i` 格式解析核心） |
| [uuid](https://github.com/uuid-rs/uuid) | 1.x | UUID v4 生成 |
| [chrono](https://github.com/chronotope/chrono) | 0.4 | 时间戳与时间计算 |
| [rand](https://github.com/rust-random/rand) | 0.8 | 随机数（出售时间抖动） |
| [thiserror](https://github.com/dtolnay/thiserror) | 2.x | 自定义错误类型 |
| [shellexpand](https://github.com/netvl/shellexpand) | 3.x | 路径环境变量展开 |
| [which](https://github.com/harryfei/which-rs) | 7.x | 查找 Node.js 可执行路径 |
| log / env_logger | 0.4/0.11 | 日志 |

### React 前端 (web/)

| 依赖 | 版本 | 用途 |
|------|------|------|
| [React](https://react.dev/) | 19.x | UI 框架 |
| [TypeScript](https://www.typescriptlang.org/) | ~6.0 | 类型安全 |
| [Vite](https://vite.dev/) | 8.x | 构建工具 + 开发服务器 |
| [Tailwind CSS](https://tailwindcss.com/) | v4 | 实用优先的 CSS 框架 |
| [@tauri-apps/api](https://github.com/tauri-apps/tauri) | 2.x | Tauri IPC 通信 |
| [@tauri-apps/plugin-dialog](https://github.com/tauri-apps/plugins-workspace) | 2.x | 前端对话框 API |
| [@tauri-apps/plugin-fs](https://github.com/tauri-apps/plugins-workspace) | 2.x | 前端文件系统 API |
| [@tauri-apps/plugin-shell](https://github.com/tauri-apps/plugins-workspace) | 2.x | 前端 Shell API |
| [oxlint](https://oxc.rs/) | 1.x | Rust 编写的快速 Linter |

### 构建系统

- **Rust 版本**：edition 2024
- **构建工具**：`cargo` + `tauri-build` 2.x
- **前端构建**：Vite 8 → `web/dist/` → Tauri webview 加载
- **开发模式**：Vite 开发服务器 (port 3147) + Tauri dev

---

## 项目结构

```
d2r-marketplace-tauri/
│
├── src-tauri/                          # Rust 后端
│   ├── Cargo.toml                      # Rust 依赖清单
│   ├── tauri.conf.json                 # Tauri v2 配置（窗口、CSP、打包）
│   ├── build.rs                        # Tauri 构建脚本
│   ├── icons/                          # 应用图标
│   └── src/
│       ├── main.rs                     # 入口 → lib::run()
│       ├── lib.rs                      # Tauri 启动、插件注册、IPC 命令注册
│       │
│       ├── commands/                   # Tauri IPC 命令处理层
│       │   ├── mod.rs
│       │   ├── stash.rs                # read_stash / read_stash_file / create_stash_backup
│       │   ├── marketplace.rs          # 买卖/上架/取消/导入/导出/价格建议
│       │   ├── balance.rs              # get_balance
│       │   └── config.rs               # 存档路径/游戏目录/模组/语言配置
│       │
│       ├── database/                   # SQLite 数据层
│       │   ├── mod.rs
│       │   ├── db.rs                   # 建表、CRUD、自动出售逻辑
│       │   └── models.rs               # VirtualItem / ListedItem / SoldItem / Transaction / AppConfig
│       │
│       ├── stash/                      # D2R .d2i 二进制格式解析器
│       │   ├── mod.rs
│       │   ├── bit_reader.rs           # LSB-first 位读取器
│       │   ├── bit_writer.rs           # LSB-first 位写入器
│       │   ├── huffman.rs              # 霍夫曼编解码（物品类型字符串）
│       │   ├── page.rs                 # D2I Page 解析（0xAA55AA55 头部、拆分/合并）
│       │   ├── item.rs                 # StashItem 结构体、读写/更新
│       │   ├── node_reader.rs          # Node.js 后备解析器
│       │   ├── constants.rs            # 物品代码映射表、堆叠物品代码集
│       │   ├── game_items.rs           # 1200+ 游戏物品定义
│       │   ├── game_item_names.rs      # 英文物品名称表
│       │   ├── item_names.rs           # 从游戏数据文件加载本地化名称
│       │   ├── chinese_item_names.rs   # 中文物品名称表
│       │   └── magical_props.rs        # 魔法属性表
│       │
│       └── market/                     # 经济/逻辑层
│           ├── mod.rs
│           ├── pricing.rs              # 参考价格、出售价格计算
│           ├── sell_time.rs            # 自动出售计时器
│           └── trade_rules.rs          # 交易规则（当前仅支持符文）
│
├── web/                                # React 前端（主前端）
│   ├── index.html                      # React 入口 HTML
│   ├── package.json                    # 前端依赖
│   ├── vite.config.ts                  # Vite 构建配置
│   ├── tsconfig*.json                  # TypeScript 配置
│   └── src/
│       ├── main.tsx                    # React 渲染入口
│       ├── App.tsx                     # 主应用（导航、余额显示）
│       ├── tauri.ts                    # Tauri IPC 封装
│       ├── types.ts                    # TypeScript 类型定义
│       ├── index.css                   # 全局样式 + Tailwind v4
│       ├── components/
│       │   ├── Toast.tsx               # 消息通知组件
│       │   └── SellModal.tsx           # 出售/上架对话框
│       └── pages/
│           ├── Home.tsx                # 欢迎页
│           ├── Inventory.tsx           # 共享仓库查看
│           ├── Catalog.tsx             # 符文市场
│           ├── Listings.tsx            # 上架管理
│           ├── Config.tsx              # 设置
│           └── Support.tsx             # 赞助页
│
├── src/                                # 旧版前端（Plain HTML/CSS/JS，过渡使用）
│   ├── index.html                      # 完整功能页面
│   ├── assets/
│   │   ├── css/                        # D2 主题 CSS
│   │   ├── js/main.js                  # 原 Flask API 调用
│   │   └── img/                        # 物品图标
│   └── package.json                    # 根级脚本
│
├── tools/                              # 数据提取和调试工具
│   ├── extract_constants.cjs           # 从游戏提取物品常量
│   ├── extract_all.cjs                 # 提取所有物品定义
│   ├── extract_chinese_names.cjs       # 提取中文名称
│   ├── all_items.json                  # 游戏物品数据
│   ├── chinese_item_names.json         # 中文物品名称
│   └── ...                             # JSON 数据 / 检查脚本
│
├── CLAUDE.md                           # AI 编程助手指南
├── package.json                        # 根目录脚本
└── README.md                           # 本文件
```

---

## 快速开始

### 前置条件

- [Rust](https://www.rust-lang.org/)（edition 2024，推荐使用 [rustup](https://rustup.rs/)）
- [Node.js](https://nodejs.org/)（>=20，用于前端构建）
- 如果使用 Node.js 后备解析器：原项目 `d2r-marketplace/tools/d2r_parser/`（自动检测）

### 开发模式

```bash
# 1. 安装前端依赖
cd web
npm install

# 2. 启动开发（前端 Vite dev server + Tauri 窗口）
cd ../src-tauri
cargo tauri dev

# 或从根目录
cd ..
npm run dev
```

开发模式下：
- Vite 在 `http://localhost:3147` 启动热重载服务器
- Tauri 窗口加载该 URL
- Rust 代码改动需手动重启

### 构建发布版

```bash
cd src-tauri && cargo tauri build
```

构建产物位于 `src-tauri/target/release/`。

### 仅编译 Rust

```bash
cd src-tauri && cargo build
```

### 运行测试

```bash
cd src-tauri && cargo test
cd web && npm run build   # TS 类型检查 + Vite 产物验证
```

依赖真实游戏存档的测试（个人存档已不随仓库分发）在文件缺失时自动 SKIP；
有本地存档时真实执行。格式解析规格与未对齐项见
[docs/protocol-d2s.md](docs/protocol-d2s.md) 与 [docs/protocol-d2i.md](docs/protocol-d2i.md)。

---

## 配置

### 存档路径

应用首次启动会创建 SQLite 数据库在：

| 平台 | 路径 |
|------|------|
| Windows | `%LOCALAPPDATA%\D2RMarketplace\database\` |
| macOS | `~/Library/Application Support/D2RMarketplace/` |
| Linux | `~/.local/share/D2RMarketplace/` |

### 游戏数据

支持通过 `game_root` 配置加载多语言物品名称。如果配置了模组目录，还能自动检测已安装模组：

- 自动扫描 `mods/` 目录下的模组
- 读取 JSON 字符串文件（item-names.json, item-runes.json 等）获取本地化名称
- 回退到内嵌名称表

### 代币经济

| 参数 | 值 |
|------|-----|
| 初始余额 | 10,000 代币 |
| 出售价格 | 标价的 70% |
| 出售计时 | 依据标价/参考价比率自动计算 |

**出售时间与价格比率：**

| 比率（标价/参考价） | 出售等待时间 |
|---|---|
| ≤80% | 1–10 分钟 |
| 80–95% | 1–25 分钟 |
| 95–105% | 2–30 分钟 |
| 105–125% | 30 分钟–3 小时 |
| 125–160% | 3–8 小时 |
| 160%+ | 8–24 小时 |
| 无参考价 | 30–90 分钟 |

药水类快 45%，宝石类快 25%。

---

## 仓库格式 (D2R .d2i)

参见原项目文档：`docs/d2r-stash-reverse-engineering.md`

**核心要点：**
- **多页结构**：64 字节头部（魔数 `0xAA55AA55`），最多 50 页（mod 扩展）
- **位级编码**：LSB-first 逐位读写
- **霍夫曼编码**：物品类型字符串（4 字符），3 字符 mod 扩展也支持
- **仓库堆叠尾部**：1 位标志 + 8 位数量
- **版本**：v96/v97/v98/v105/v111，硬编码默认 v105
- **★ Page 3b 字段**：v97+ 才有（ItemPage 枚举：Equipped/Backpack/MyStash/SharedStash），新协议层已修复漏读 bug

---

## 迁移状态

| 模块 | 状态 | 备注 |
|------|------|------|
| `core/bitio/` | ✅ 完成 | BitReader + BitWriter + peek/align/slice |
| `core/encoding/` (Huffman) | ✅ 完成 | 20 个测试通过 |
| `core/result.rs` / `version.rs` | ✅ 完成 | ParseError + ProtocolVersion (V96/97/98/105/111) |
| `protocol/common/item_*` | ✅ 完成 | ItemFlags/Mode/Location/Page/Quality + FieldSet |
| **`protocol/common/item_page.rs`** | ★ 完成 | **Page 3b 字段修复**（之前漏读导致 Page[0] 80 装备只解 47 个） |
| `protocol/common/stat.rs` / `stat_list.rs` / `stat_table.rs` | ✅ 完成 | 0x1FF 终止符 + sub-property 自动展开 |
| `protocol/d2i/parser.rs` | ✅ 完成 | compact + complete header（★ Page 3b 修复） + chest-stackable trailer |
| `protocol/d2i/page*.rs` / `summary.rs` | ✅ 完成 | 64-byte page header + 多页切分 + 轻量摘要 |
| `protocol/d2i/legacy/` | ⚠️ 兼容层 | 21 文件从原 stash/ 迁入，含完整 1500 行 complete header |
| `protocol/d2s/` (角色存档) | ✅ 基础完成 | header + attributes（61 bytes）+ skills |
| `data/stat_cost.rs` | ✅ 完成 | 420+ stat 定义 + build_stat_table() |

### 业务层

| 组件 | 状态 | 备注 |
|------|------|------|
| 数据库 Schema + CRUD | ✅ 完成 | SQLite via rusqlite |
| D2I 页面拆分/合并 | ✅ 完成 | 真实 d2i fixture 测试通过 |
| 物品读写（堆叠类 + 装备） | ✅ 完成 | 通过 protocol::d2i::legacy 委托 |
| 物品常量 + 查找 | ✅ 完成 | data::items / ITEM_CODE_MAP |
| 物品 stat（magical properties） | ✅ 完成 | 通过 stat_list 完整解析 |
| 市场定价 | ✅ 完成 | 从 Python 移植 |
| 出售时间计算 | ✅ 完成 | 含单元测试 |
| 交易规则 | ✅ 完成 | 含单元测试 |
| Tauri 命令 | ✅ 完成 | 27 个 IPC 命令注册 |
| React 前端 (web/) | ✅ 基本实现 | 需完善 UI |
| 存档文件发现 | ✅ 完成 | 与原版一致 |
| 备份系统 | ✅ 完成 | 时间戳命名拷贝 |
| 自动出售 (process_due) | ✅ 完成 | 页面加载时检查 |
| 取消上架（归还仓库） | ✅ 完成 | 写回 .d2i |
| 购买物品（写入仓库） | ✅ 完成 | 基于克隆创建 |
| 角色信息 (d2s) | ✅ 完成 | read_character_info / list_characters |
| 配置命令 | ✅ 完成 | shellexpand 展开环境变量 |
| Node.js 后备解析器 | ✅ 完成 | 自动检测原项目路径 |

### 性能改进（真实 d2i 集成测试）

`ModernSharedStashSoftCoreV2.d2i`（18 KB）解析对比：

| 阶段 | items 解析数 | 3-char 干净码 |
|------|------|------|
| Phase 1（贪婪，无 stat_list） | 1270（错误） | 22.8% |
| Phase 2.3（+stat_list） | 278 | 45.7% |
| Phase 3.1（+complete header） | **217**（精准） | 45.2% |
| Legacy baseline | ~125 simple | — |

Page 3b 字段修复使 Page[0] 装备能完整解析（之前漏 33 个装备位流错位）。

---

## 开发注意事项

- 🪟 **仅支持 Windows**（打包目标 msi/nsis，CI 为 windows-latest；macOS/Linux 不在支持范围）
- ⚠️ **不要同时运行 D2R Marketplace 和游戏** — 可能导致存档损坏
- 当前仅支持**堆叠类物品**（符文、宝石、药水、钥匙、精华、赦免徽章、碎片）
- 市场目前仅允许**符文购买**直接写入仓库
- 最大堆叠数 `MAX_STACK = 99`（硬编码在 `item.rs`）
- React 前端端口固定 `3147`（strictPort）
- `.d2i` 解析优先使用 Rust 原生解析器，Node.js 解析器作为后备
- 已知缺陷：`warehouse_deposit_withdraw_e2e` 的 4 个 withdraw 测试在带真实 stash
  样本时失败（step4.5 重解析校验 `parse as '(none)'`）— 预存在问题，未修复，
  欢迎 PR（无样本时这些测试 SKIP，CI 不受影响）

---

## 许可

GPLv3（见 [LICENSE](LICENSE)）。

第三方资源（游戏图标、字体、格式参考）的版权与许可见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
