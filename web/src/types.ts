// ═══ 类型定义 ═══

export interface StashSocketedItem {
  code: string
  item_name: string
  quality: string
  quantity: number
}
/** Alias for StashSocketedItem — used by ItemTooltip for socketed-items display. */
export type StashSocketedItemInfo = StashSocketedItem

export interface StashItem {
  /** 后端 read_stash 返回的唯一 id (格式 stash-page-x-y-seq) */
  id: string
  item_name: string
  /** 英文名称（可选，tooltip 英文行用） */
  name_en?: string
  quality: string
  code: string
  kind: string
  icon: string
  position_x: number
  position_y: number
  inv_width: number
  inv_height: number
  page_index: number
  /** 堆叠数量（符文/宝石等 stackable 物品才有） */
  quantity?: number
  mod_added: boolean
  tooltip_lines: string[]
  /**
   * 结构化 tooltip 数据（后端 classify_tooltip 已分类）。
   * 优先消费此字段,老数据 / mod_added 时降级到 tooltip_lines。
   */
  tooltip?: TooltipData
  socketed_items: StashSocketedItem[]
  /**
   * 装备位 (D2 item 4b location_id):
   *   0=None(容器存放), 1=Head, 2=Neck, 3=Torso, 4=RightHand, 5=LeftHand,
   *   6=RightFinger, 7=LeftFinger, 8=Waist, 9=Feet, 10=Hands,
   *   11=Trinket1(D2R), 12=Trinket2(D2R)
   * 后端 d2i bitstream 直接解析。undefined = 解析失败(老数据)。
   */
  equipment_slot?: number
  /**
   * 底材类型 (D2 txt type 列,如 "sword"/"axe"/"shield"/"helm"/"armor"/"spear"/...)。
   * 来源:game_data_loader::get_item_def(code).item_type。
   * undefined = game_data 未加载 / code 不在 vanilla 表 / 非装备类物品。
   */
  base_type?: string
  /**
   * per-code 默认收藏页(后端 resolve 后回填)。
   * 由 `warehouse_resolve_default` 写入;不改时为 null/undefined。
   */
  default_page_name?: string | null
}

export interface StashPageInfo {
  index: number
  is_stackable: boolean
  item_count: number
  label: string
  grid_width: number
  grid_height: number
}

export interface StashResult {
  stash_name: string
  stash_file: string | null
  item_count: number
  read_status: string
  items: StashItem[]
  pages: StashPageInfo[]
}

export interface ListedItem {
  id: string
  name: string
  quantity: number
  unit_price: number
  listed_at: string | null
  sell_after_seconds: number
  status: string | null
  /** 物品 4-char code,如 "r01" (El Rune) */
  item_code: string | null
  /** 物品 kind,"rune" / "gem" / "potion" / "key" / "essence" / ... */
  item_kind: string | null
  /** 物品品质,"unique" / "set" / "rare" / "magic" / "normal" */
  quality: string | null
  /** 上架人(角色存档文件名,如 "EchoingStrike.d2s") */
  listed_by: string | null
}

export interface PriceSuggestion {
  base_price: number
  suggested_price: number
  min_price: number
  max_price: number
  variation: number
  has_reference: boolean
}

export interface ModMeta {
  name: string
  version: string
  description: string
  author: string
  save_path: string
}

export interface ResourceFileInfo {
  role: string
  file_type: string
  relation: string
  path: string
  exists: boolean
  languages: string[]
}

export interface ResourceManifest {
  profile_id: string
  source_kind: string
  game_version: string
  mod_name: string
  game_root: string
  excel_path: string
  strings_path: string
  strings_legacy_path: string
  active_language: string
  supported_languages: string[]
  txt_files: ResourceFileInfo[]
  json_files: ResourceFileInfo[]
  fallback_chain: string[]
  notes: string[]
}

export interface ImportStatus {
  table_name: string
  rows: number
  source: string
  elapsed_ms: number
  status: string
}

export interface ProfileInfo {
  id: number
  profile_key: string
  source_kind: string
  mod_name: string
  game_version: string
  active_language: string
  game_root: string
  excel_path: string
  checksum: string
  source_path: string
  imported_at: string | null
  import_status: ImportStatus[]
  localized_count: number
  created_at: string
}

export interface AppConfig {
  save_folder: string
  default_folder: string
  game_root: string
  profile_key: string
  active_mod: string
  game_version: string
  language: string
  stash_grid_size: number
  backpack_cols: number
  backpack_rows: number
  cube_cols: number
  cube_rows: number
  available_mods: string[]
  mod_metadata: ModMeta[]
  game_data_path: string
  resource_manifest?: ResourceManifest | null
  import_status: ImportStatus[]
  profiles: ProfileInfo[]
}

export interface BackupResult {
  success: boolean
  backup_count: number
  message: string
}

export interface BackupFileInfo {
  filename: string
  file_type: 'character' | 'stash' | 'config' | 'other'
  backup_size: number
  current_size: number | null
}

export interface BackupEntry {
  timestamp: string
  path: string
  files: BackupFileInfo[]
  total_size: number
  created_at: string
}

export interface AutoBackupEntry {
  filename: string
  original_stash: string
  operation: string
  timestamp: string
  size: number
  path: string
}

export interface AutoSaveInfo {
  file_count: number
  total_size: number
  files: string[]
  timestamp: string
}

export interface SafetyBackupEntry {
  dirname: string
  files: string[]
  file_count: number
  total_size: number
  timestamp: string
}

export interface BuyResult {
  new_balance: number
  item_id: string
  stash_path: string
}

export interface HistoryEntry {
  id: number
  tx_type: string
  item_id: string | null
  token_amount: number
  description: string
  date: string | null
}

export interface VirtualItem {
  id: string
  name: string
  item_code: string | null
  item_kind: string | null
  item_type: string | null
  quality: string | null
  level: number | null
  attributes: string | null
  source: string | null
  exported_from: string | null
  purchased_at: string | null
  token_price: number | null
  status: string | null
  quantity: number | null
  unit_price: number | null
  listed_at: string | null
  sell_after_seconds: number | null
}

export interface WarehouseItem {
  id: string
  item_code: string
  item_name: string
  /** 英文名（后端 localize_warehouse_items 提供） */
  name_en?: string
  item_kind: string
  quality: string | null
  quantity: number
  page_name: string
  imported_at: string
  tags: string
  notes: string
  tooltip_lines: string[]
  /** 结构化 tooltip 数据（后端 localize_warehouse_items 构建, 与角色页同款分类）。
   *  优先于 tooltip_lines; 老数据/降级时为 undefined。 */
  tooltip?: TooltipData | null
  /** 仓库页索引 */
  page_index: number
  /** 在网格中的 X 坐标 */
  position_x: number
  /** 在网格中的 Y 坐标 */
  position_y: number
  /** 物品占格宽度 */
  inv_width: number
  /** 物品占格高度 */
  inv_height: number
  /** 可选: 后端透传的图标 URL (优先级高于 code 派生) */
  icon?: string
  /**
   * 解析后的默认收藏页(per-item override > per-code 默认 > null)。
   * 后端由 `warehouse_resolve_default` 返回,前端可读不可改。
   */
  default_page_name?: string | null
}

export interface WarehouseMetaDraft {
  page_name: string
  tags: string
  notes: string
}

/** 单个原始 stat（匹配 Rust ItemStat）。 */
export interface ItemStat {
  id: number
  param: number
  value: number
  skill_tab?: number | null
  skill_level?: number | null
  skill_id?: number | null
  max_charges?: number | null
}

/** 分类原始 stat（匹配 Rust ItemStats）。 */
export interface ItemStats {
  /** 基础属性（superior 底材增强） */
  base?: ItemStat[]
  /** 词缀/魔法属性 */
  affix?: ItemStat[]
  /** 符文之语额外词缀 */
  runeword?: ItemStat[]
  /** 套装词缀 */
  set_bonus?: ItemStat[]
}

// 角色信息 (read_character_info 响应)


/** 结构化 tooltip 数据（匹配 Rust TooltipData）。 */
export interface TooltipData {
  /** 基础信息行（名称、代码、类型、品质） */
  base_info: string[]
  /** 词缀属性行（+力量、+生命、抗性等） */
  stats: string[]
  /** 隐藏调试信息（物品等级、ID 等） */
  hidden_info: string[]
  /** 套装加成信息 */
  set_info: string[]
  /** 孔 + 镶嵌物品(后端结构化填,前端 ItemTooltip 可直接渲染) */
  sockets?: SocketsInfo
  /** 基础属性（如防御、伤害） */
  base_stats?: string[]
  /** 仅词缀属性（不含基础属性） */
  affix_stats?: string[]
  /** 符文之语加成 */
  runeword_stats?: string[]
  /** 套装加成 */
  set_bonus_stats?: string[]
}

/** 物品的孔位信息 */
export interface SocketsInfo {
  /** 总孔数(可能 > items.length,空孔用 length 表达) */
  count: number
  /** 已镶嵌物品列表(按孔位顺序) */
  items: SocketedItemInfo[]
}

/** 镶嵌物品精简信息(嵌在 TooltipData.sockets.items 里) */
export interface SocketedItemInfo {
  code: string
  name_zh?: string
  name_en?: string
  quality?: number
  amount: number
}
// 单个结构化技能/充能词缀（来自 ItemStat 的拆分字段）
// - skill_tab: stat 188 — +N to <Tab> Skill Levels
// - chance_to_cast: stat 195-201 — +X% chance to cast <Skill>
// - skill_charges: stat 204 — +N charges of <Skill>
export interface SkillBonus {
  /** 原始 stat_id */
  stat_id: number
  kind: 'skill_tab' | 'chance_to_cast' | 'skill_charges' | 'single_skill'
  skill_id?: number | null
  skill_tab?: number | null
  skill_level?: number | null
  chance_pct?: number | null
  max_charges?: number | null
  current_charges?: number | null
  /** 技能名称（后端解析，仅 single_skill 有值） */
  skill_name?: string | null
}

/** 角色技能硬点（职业内 0-based skill id）。 */
export interface SkillEntry {
  id: number
  level: number
}

/** list_characters_brief 响应 — 轻量角色信息 */
export interface CharacterBriefInfo {
  name: string
  class_en: string
  class_cn: string
  level: number
  is_hardcore: boolean
  is_dead: boolean
  is_expansion: boolean
  file_hash: string
  save_timestamp: number
}

export interface CharacterInfo {
  name: string
  class: string
  class_en: string
  class_cn: string
  class_zh_tw: string
  level: number
  experience: number
  strength: number
  energy: number
  dexterity: number
  vitality: number
  current_hp: number
  max_hp: number
  current_mana: number
  max_mana: number
  is_hardcore: boolean
  is_expansion: boolean
  last_played: number
  /** 背包金币 */
  gold: number
  /** 仓库金币 */
  gold_bank: number
  /** 未分配属性点 */
  stat_points: number
  /** 未分配技能点 */
  new_skills: number
  source_path: string
  /**
   * 12 个装备槽位。
   * 与后端协议层保持一致，包含左右戒指和主/副两套武器组。
   */
  equipment: Array<{
    slot: string
    occupied: boolean
    code?: string
    name_zh?: string
    name_en?: string
    name_zh_tw?: string
    quality?: string
    tooltip_lines?: string[]
    /**
     * 结构化 tooltip 数据（后端 Rust classify_tooltip 已分类）。
     * 优先使用此字段代替 tooltip_lines 的正则猜测。
     */
    tooltip?: TooltipData
    /**
     * 结构化技能/充能词缀（Phase L, 2026-07-09）。
     * 前端可独立于 tooltip_lines 渲染"+1 暴风雪"等徽章,
     * 或聚合所有装备 +N 技能。
     * 魔改 layout fallback 此字段始终为空。
     */
    skill_bonuses?: SkillBonus[]
    durability_cur?: number
    durability_max?: number
    /** 原始 stat 分类数据 (base/affix/runeword/set_bonus) */
    stats?: ItemStats
  }>
  /**
   * 背包物品列表
   */
  backpack_items?: Array<{
    code: string
    x: number
    y: number
    amount: number
    inv_width: number
    inv_height: number
    quality?: number | null
    name_zh?: string
    name_en?: string
    name_zh_tw?: string
    page?: number | null
    tooltip_lines: string[]
    /** Phase L: 结构化技能/充能词缀 */
    skill_bonuses?: SkillBonus[]
    /** 文件偏移（16 进制） */
    raw_offset?: string
    /** 位长度 */
    raw_length: number
    /** 镶嵌物品（符文/宝石/珠宝等） */
    socketed_items?: Array<{
      code: string
      amount: number
      quality?: number | null
      name_zh?: string
      name_en?: string
    }>
    /** 原始 stat 分类数据 (base/affix/runeword/set_bonus) */
    stats?: ItemStats
    }>
  /**
   * 腰带物品列表
   */
  belt_items?: Array<{
    code: string
    x: number
    y: number
    amount: number
    inv_width: number
    inv_height: number
    quality?: number | null
    name_zh_tw?: string
    page?: number | null
    tooltip_lines: string[]
    /** Phase L: 结构化技能/充能词缀 */
    skill_bonuses?: SkillBonus[]
    raw_offset?: string
    raw_length: number
    /** 镶嵌物品（符文/宝石/珠宝等） */
    socketed_items?: Array<{
      code: string
      amount: number
      quality?: number | null
      name_zh?: string
      name_en?: string
    }>
    /** 原始 stat 分类数据 (base/affix/runeword/set_bonus) */
    stats?: ItemStats
    }>
  /**
   * 个人仓库物品（d2s JM 段 Page=MyStash,16×16 网格）
   * 由 d2s parser filter Page=5 暴露,共享 stash 在 d2i 文件另行处理
   */
  personal_stash_items?: Array<{
    code: string
    x: number
    y: number
    amount: number
    inv_width: number
    inv_height: number
    quality?: number | null
    name_zh?: string
    name_en?: string
    name_zh_tw?: string
    page?: number | null
    tooltip_lines: string[]
    skill_bonuses?: SkillBonus[]
    raw_offset?: string
    raw_length: number
    socketed_items?: Array<{
      code: string
      amount: number
      quality?: number | null
      name_zh?: string
      name_en?: string
    }>
    /** 原始 stat 分类数据 (base/affix/runeword/set_bonus) */
    stats?: ItemStats
  }>
  /**
   * 三难度小站标记（normal/nightmare/hell 各 40 bit）
   */
  waypoints?: {
    normal: boolean[]
    nightmare: boolean[]
    hell: boolean[]
  }
  /**
   * 任务进度列表
   */
  quests?: Array<{
    difficulty: number
    act: number
    quest_id: number
    completed: boolean
  }>
  /**
   * 魔改 layout 标记 — true 表示 d2s 文件布局魔改 (xieedi/happy_manman 类型),
   * Level/Class/Attributes 不可信,只装备位按 code 推断填充。
   * 前端可据此显示 disclaimer banner。
   */
  is_modified_layout?: boolean
  /** Woo! 段任务数据 */
  woo?: {
    progression: number
    difficulties: number[][][]
  }
  /** w4 NPC 奖励消费状态 */
  w4?: {
    block_type: number
    normal: number
    normal_extra: number
    nightmare: number
    nightmare_extra: number
    hell: number
    hell_extra: number
  }
  /** 佣兵装备 */
  merc_equipment?: Array<{
    slot: string
    occupied: boolean
    code?: string
    name_zh?: string
    name_en?: string
    name_zh_tw?: string
    quality?: string
    tooltip_lines?: string[]
    tooltip?: TooltipData
    durability_cur?: number
    durability_max?: number
    /** 原始 stat 分类数据 (base/affix/runeword/set_bonus) */
    stats?: ItemStats
  }>
  binary_structure: {
    detected_layout: string
    active_weapon: number
    attributes_offset: number
    protocol_equipped_slots: number
    display_equipped_slots: number
    item_layout: {
      location_id_bit_offset: number
      equipped_slot_bit_offset: number
      huffman_code_bit_offset: number
      socket_count_bits: number
      uid_bits: number
      ilvl_bits: number
      quality_bits: number
      stat_terminator: number
    }
  }
  skills_decoded?: SkillEntry[]
}

export interface NamespaceGap {
  namespace: string
  enus_count: number
  lang_count: number
  missing: number
  missing_pct: number
}

export interface LocaleDiagnosis {
  target_lang: string
  total_namespaces: number
  namespaces: NamespaceGap[]
  overall_missing_pct: number
}
