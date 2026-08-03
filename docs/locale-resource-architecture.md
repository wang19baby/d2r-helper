# 多语言资源与 SQLite 资源层

## 目标

把当前散落在运行时的 `txt/json` 资源关系整理成可扩展的资源层，满足以下场景：

- 同时支持 `enUS / zhCN / zhTW`
- 支持多个 mod
- 支持多个原版版本，例如 `2.4 / 2.7 / 3.0 / 3.2`
- 让角色页、仓库页、tooltip、市场页共用同一套名称/本地化来源

## 当前资源来源

### TXT 类

这些文件主要负责“协议定义”和“基础英文名”：

- `misc.txt`
- `armor.txt`
- `weapons.txt`
- `uniqueitems.txt`
- `setitems.txt`
- `sets.txt`
- `skills.txt`
- `skilldesc.txt`
- `ItemStatCost.txt`

其中：

- `misc/armor/weapons.txt`
  - 提供 `code -> name/type/invwidth/invheight/stackable`
  - 是基础物品表
- `uniqueitems.txt`
  - 提供 `unique_id -> 英文名/base code`
- `setitems.txt + sets.txt`
  - 提供 `set_id -> 套装物品名/套装名/奖励`
- `skills.txt + skilldesc.txt`
  - 提供技能、灵气、技能页名称语义
- `ItemStatCost.txt`
  - 提供 stat 位宽、参数位宽、编码信息

### JSON 类

这些文件主要负责“显示文本”和“多语言”：

- `item-names.json`
- `item-runes.json`
- `item-gems.json`
- `item-nameaffixes.json`
- `item-rarenames.json`
- `item-modifiers.json`

在 mod 目录中通常位于：

- `data/local/lng/strings/`
- `data/local/lng/strings-legacy/`

其中：

- `strings/`
  - 主语言资源
- `strings-legacy/`
  - 旧版补丁资源，作为回补来源

## 当前解析关系

当前名称解析逻辑是：

1. 从 `misc/armor/weapons.txt` 构建 `code -> 英文基础名`
2. 从 `item-*.json` 读取 `Key -> zhCN/zhTW/enUS`
3. 用英文基础名或直接 code 反查 JSON，得到目标语言显示名
4. 若失败，再回退到内置表或硬编码 fallback

这个逻辑单 mod 可以工作，但在多 mod / 多版本下会出现问题：

- 资源来源不可追踪
- 无法稳定缓存
- 同一个 `code` 在不同 profile 下可能含义不同
- 同一个 `Key` 在不同 mod/版本中可能翻译不同

## 新资源模型

### 资源画像 `resource_profile`

一套资源上下文由以下字段唯一确定：

- `source_kind`
- `mod_name`
- `game_version`

实际落库键为：

- `profile_key`

推荐格式：

- `vanilla:2.4`
- `vanilla:2.7`
- `mod:仙道轮回:3.2`

### 资源文件 `resource_file`

用于记录：

- 当前 profile 用到了哪些文件
- 文件角色是什么
- 文件是否存在
- 文件支持哪些语言

### 语言字符串 `localized_string`

用于记录：

- `profile_id`
- `namespace`
- `string_key`
- `language`
- `text_value`
- `source_path`

这张表是当前第一阶段真正入库的数据表。

## 已落地的 SQLite 表

当前代码已经创建以下资源层表：

- `resource_profile`
- `resource_file`
- `localized_string`

并且在读取配置时会做两件事：

1. 根据当前 `game_root + active_mod + game_version + language` 构建 `resource_manifest`
2. 将 `resource_manifest` 与 JSON 语言字符串导入 SQLite

## 当前已导入的数据

已导入：

- profile 行
- 资源文件行
- JSON 语言字符串行

暂未导入为结构化数据表：

- `item_base`
- `unique_item_def`
- `set_item_def`
- `runeword_def`
- `stat_def`
- `skill_def`

这些属于第二阶段。

## 推荐的下一阶段

### 第二阶段：TXT 结构化导入

增加以下表：

- `item_base`
- `unique_item_def`
- `set_item_def`
- `runeword_def`
- `stat_def`
- `skill_def`

### 第三阶段：业务查询统一走 `profile_id`

角色页、仓库页、tooltip、市场页统一带上：

- `profile_id`

这样所有显示文本和定义查询都能按 profile 稳定命中，不再临时扫描磁盘目录。

## 设计原则

- 原始 `txt/json` 仍然保留在磁盘，作为源数据
- SQLite 负责做索引、关系、缓存和查询层
- 所有资源查询都应显式区分 `profile_id`
- 不再假设“一个 code 全局唯一”
