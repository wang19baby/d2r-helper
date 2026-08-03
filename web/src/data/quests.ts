//! D2R 任务（Quest）基础数据。
//!
//! 数据来源:
//! - 27 个真实任务分布在 5 个 Act (A1/A2/A3/A5 各 6 个, A4 只有 3 个)
//! - d2s kf 段每个 act 分配 6 槽位 × 3 难度 = 90 bits（其中 A4 后 3 槽未用）
//! - 每个任务有 3 个难度副本（普通 / 恶梦 / 地狱）
//!
//! 字段说明:
//! - `id`         : 0-based 全局索引
//! - `act`        : 1-5
//! - `questId`    : 0-based act 内 index（与 d2s kf 段 bit 顺序一致）
//! - `name`       : 英文显示名
//! - `d2emuKey`   : d2emu hero editor 内部 PascalCase 键名（用于匹配 questIconIndexes）
//! - `iconIndex`  : d2emu 用的 quest icon 索引 (1-6, 与 act 内位置 0-based + 1)
//! - `rewards`    : 完成该任务给的属性/技能点（如果不给则字段省略）

export type D2RAct = 1 | 2 | 3 | 4 | 5

/**
 * 单条任务对角色属性加成的描述。
 * `statPoints` / `skillPoints` 是**单次完成**给的属性点（不分难度）。
 *
 * 已知规则（D2R 标准 + 魔改 mod 常见值）:
 * - Den Of Evil        : 1 skill pt (NM/Hell 有效, Normal 不给)
 * - The Search For Cain: 1 stat pt  (仅 NM/Hell)
 * - Tools Of The Trade : 1 stat pt  (任何难度都给)
 * - Radament's Lair    : 1 stat pt  (仅 NM/Hell)
 * - Lam Esen's Tome    : 2 stat pt  (任何难度都给)
 *
 * ⚠️ 这些是 d2s 二进制里没有的"游戏规则",只能从玩家社区知识获取。
 * 如果发现数字对不上,可在此处调整。
 */
export interface QuestRewards {
  statPoints?: number
  skillPoints?: number
}

export interface QuestDef {
  id: number
  act: D2RAct
  /** act 内 0-based index（与 d2s kf 段位顺序一致） */
  questId: number
  name: string
  name_cn?: string
  /** d2emu hero editor 内部 PascalCase 键（用于匹配 questIconIndexes） */
  d2emuKey: string
  /** d2emu 用的 quest icon 索引 1-6（act 内位置 0-based + 1） */
  iconIndex: number
  /** 该任务完成给的属性 / 技能点（如果不给则字段省略） */
  rewards?: QuestRewards
}

/** 真实存在的任务数 = 27 (A1=6 + A2=6 + A3=6 + A4=3 + A5=6) */
export const TOTAL_QUESTS = 27

/**
 * d2s kf 段为每个 act 预留的 quest 槽位数（即使 A4 只用 3 个，仍占 6 槽位）。
 * 真实写入时,A4 后 3 槽位填 0 (未完成任务)。
 */
export const QUESTS_PER_ACT_SLOT = 6

/**
 * 按 act 查询任务（保持 act 内顺序）
 */
export function questsByAct(act: D2RAct): QuestDef[] {
  return QUESTS.filter(q => q.act === act)
}

/**
 * 通过 d2emu key 查任务（用于 d2emu 兼容 / 反向引用）
 */
export function questByD2emuKey(key: string): QuestDef | undefined {
  return QUESTS.find(q => q.d2emuKey === key)
}

/**
 * 从 QuestEntry[] 列表里聚合属性 / 技能点总和。
 * 对每个 quest entry:如果 completed=true 且对应 QuestDef 有 rewards,
 *   累加 statPoints / skillPoints。
 *
 * 输入: 角色保存里所有 difficulty 的 quest 记录
 * 输出: { stat, skill } — 该角色目前已得的"任务奖励"属性 / 技能点
 *
 * 注意: 实际 D2R 里"任务奖励属性"通常是按难度独立给的
 *   (NM/Hell 才给, Normal 不给)。本函数做的是**总和**,展示用足够,
 *   严格 per-difficulty 拆分需要把 QuestDef 的 statPoints/skillPoints
 *   改成 { normal, nightmare, hell } 结构 — 那是后续 M2 改进项。
 */
export function computeQuestBonuses(
  quests: Array<{ act: number; quest_id: number; completed: boolean }>
): { stat: number; skill: number } {
  const completed = new Set(
    quests.filter(q => q.completed).map(q => `${q.act}:${q.quest_id}`)
  )
  let stat = 0
  let skill = 0
  for (const def of QUESTS) {
    if (!completed.has(`${def.act}:${def.questId}`)) continue
    if (def.rewards?.statPoints) stat += def.rewards.statPoints
    if (def.rewards?.skillPoints) skill += def.rewards.skillPoints
  }
  return { stat, skill }
}

/**
 * 30 个任务完整数据。act 内顺序与 d2s kf 段 bit 顺序严格一致。
 *
 * 数据来源:
 *   d2s 协议 kf 段 (Tasks) — D2SLib Tasks.cs QuestId 枚举
 *   d2emu hero editor 内部命名
 *   游戏内任务名（取自 Quest.txt 字段）
 */
export const QUESTS: QuestDef[] = [
  // ── Act 1 — 6 个 ──
  { id: 0, act: 1, questId: 0, name: 'Den Of Evil',              name_cn: '邪恶洞穴',          d2emuKey: 'DenOfEvil', iconIndex: 1, rewards: { skillPoints: 1 } },
  { id: 1, act: 1, questId: 1, name: 'Sisters Burial Grounds',   name_cn: '修女埋骨之地',       d2emuKey: 'SistersBurialGrounds', iconIndex: 2 },
  { id: 2, act: 1, questId: 2, name: 'The Search For Cain',      name_cn: '寻找凯恩',           d2emuKey: 'TheSearchForCain', iconIndex: 3, rewards: { statPoints: 1 } },
  { id: 3, act: 1, questId: 3, name: 'The Forgotten Tower',      name_cn: '遗忘之塔',           d2emuKey: 'TheForgottenTower', iconIndex: 4 },
  { id: 4, act: 1, questId: 4, name: 'Tools Of The Trade',       name_cn: '交易的工具',         d2emuKey: 'ToolsOfTheTrade', iconIndex: 5, rewards: { statPoints: 1 } },
  { id: 5, act: 1, questId: 5, name: 'Sisters To The Slaughter', name_cn: '屠杀的修女',         d2emuKey: 'SistersToTheSlaughter', iconIndex: 6 },

  // ── Act 2 — 6 个 ──
  { id: 6,  act: 2, questId: 0, name: "Radament's Lair",         name_cn: '罗达门特的巢穴',     d2emuKey: 'RadamentsLair', iconIndex: 1, rewards: { statPoints: 1 } },
  { id: 7,  act: 2, questId: 1, name: 'The Horadric Staff',      name_cn: '赫拉迪克法杖',       d2emuKey: 'TheHoradricStaff', iconIndex: 2 },
  { id: 8,  act: 2, questId: 2, name: 'Tainted Sun',             name_cn: '受黯的阳光',         d2emuKey: 'TaintedSun', iconIndex: 3 },
  { id: 9,  act: 2, questId: 3, name: 'Arcane Sanctuary',        name_cn: '神秘庇护所',         d2emuKey: 'ArcaneSanctuary', iconIndex: 4 },
  { id: 10, act: 2, questId: 4, name: 'The Summoner',            name_cn: '召唤者',            d2emuKey: 'TheSummoner', iconIndex: 5 },
  { id: 11, act: 2, questId: 5, name: 'The Seven Tombs',         name_cn: '七座古墓',          d2emuKey: 'TheSevenTombs', iconIndex: 6 },

  // ── Act 3 — 6 个 ──
  { id: 12, act: 3, questId: 0, name: 'The Golden Bird',           name_cn: '黄金之鸟',        d2emuKey: 'TheGoldenBird', iconIndex: 1 },
  { id: 13, act: 3, questId: 1, name: "Khalim's Will",             name_cn: '卡林的意志',      d2emuKey: 'KhalimsWill', iconIndex: 2 },
  { id: 14, act: 3, questId: 2, name: 'Blade Of The Old Religion', name_cn: '古老宗教之刃',    d2emuKey: 'BladeOfTheOldReligion', iconIndex: 3 },
  { id: 15, act: 3, questId: 3, name: "Lam Esen's Tome",           name_cn: '蓝·依森的古书',  d2emuKey: 'LamEsensTome', iconIndex: 4, rewards: { statPoints: 2 } },
  { id: 16, act: 3, questId: 4, name: 'The Blackened Temple',      name_cn: '幽暗地窖',        d2emuKey: 'TheBlackenedTemple', iconIndex: 5 },
  { id: 17, act: 3, questId: 5, name: 'The Guardian',              name_cn: '守护者',          d2emuKey: 'TheGuardian', iconIndex: 6 },

  // ── Act 4 — 3 个 ──
  { id: 18, act: 4, questId: 0, name: 'The Fallen Angel', name_cn: '堕落的天使', d2emuKey: 'TheFallenAngel', iconIndex: 1 },
  { id: 19, act: 4, questId: 1, name: 'Hellforge',        name_cn: '地狱熔炉',   d2emuKey: 'Hellforge', iconIndex: 2 },
  { id: 20, act: 4, questId: 2, name: "Terror's End",     name_cn: '恐怖终点',   d2emuKey: 'TerrorsEnd', iconIndex: 3 },

  // ── Act 5 — 6 个 ──
  { id: 21, act: 5, questId: 0, name: 'Siege On Harrogath',        name_cn: '哈洛加斯围城战',  d2emuKey: 'SiegeOnHarrogath', iconIndex: 1 },
  { id: 22, act: 5, questId: 1, name: 'Rescue On Mount Arreat',    name_cn: '亚瑞特山的救援',  d2emuKey: 'RescueOnMountArreat', iconIndex: 2 },
  { id: 23, act: 5, questId: 2, name: 'Prison Of Ice',             name_cn: '冰之囚',         d2emuKey: 'PrisonOfIce', iconIndex: 3 },
  { id: 24, act: 5, questId: 3, name: 'Betrayal Of Harrogath',      name_cn: '哈洛加斯的背叛',  d2emuKey: 'BetrayalOfHarrogath', iconIndex: 4 },
  { id: 25, act: 5, questId: 4, name: 'Rite Of Passage',           name_cn: '通道的仪式',      d2emuKey: 'RiteOfPassage', iconIndex: 5 },
  { id: 26, act: 5, questId: 5, name: 'Eve Of Destruction',        name_cn: '毁灭前夕',        d2emuKey: 'EveOfDestruction', iconIndex: 6 },
]

/**
 * 校验: id 序列 0..29 连续; act 内 questId 0-based 严格连续; A4 只有 0-2。
 */
export function validateQuests(): {
  ok: boolean
  errors: string[]
} {
  const errors: string[] = []
  const seen = new Set<number>()
  for (const q of QUESTS) {
    if (seen.has(q.id)) errors.push(`duplicate id ${q.id}`)
    seen.add(q.id)
  }
  for (let i = 0; i < TOTAL_QUESTS; i++) {
    if (!QUESTS.find(q => q.id === i)) errors.push(`missing id ${i}`)
  }
  // per-act questId continuity (A1/A2/A3/A5 各 6 槽位; A4 只有 3)
  for (const act of [1, 2, 3, 5] as D2RAct[]) {
    const expected = [0, 1, 2, 3, 4, 5]
    const actual = questsByAct(act).map(q => q.questId)
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      errors.push(`act ${act} questId mismatch: ${actual.join(',')}`)
    }
  }
  // Act 4: 真实只有 0,1,2 (后 3 槽位是 d2s kf 段 padding)
  const a4 = questsByAct(4).map(q => q.questId)
  if (JSON.stringify(a4) !== JSON.stringify([0, 1, 2])) {
    errors.push(`act 4 questId mismatch: ${a4.join(',')}`)
  }
  return { ok: errors.length === 0, errors }
}

/**
 * d2emu questIconIndexes 镜像（用于前端 SVG/PNG icon 资源匹配）。
 * 与 d2emu hero editor bundle 中的 questIconIndexes 同语义。
 */
export const QUEST_ICON_INDEX: Record<string, number> = {
  DenOfEvil: 1,
  SistersBurialGrounds: 2,
  TheSearchForCain: 3,
  TheForgottenTower: 4,
  ToolsOfTheTrade: 5,
  SistersToTheSlaughter: 6,

  RadamentsLair: 1,
  TheHoradricStaff: 2,
  TaintedSun: 3,
  ArcaneSanctuary: 4,
  TheSummoner: 5,
  TheSevenTombs: 6,

  TheGoldenBird: 1,
  KhalimsWill: 2,
  BladeOfTheOldReligion: 3,
  LamEsensTome: 4,
  TheBlackenedTemple: 5,
  TheGuardian: 6,

  TheFallenAngel: 1,
  Hellforge: 2,
  TerrorsEnd: 3,

  SiegeOnHarrogath: 1,
  RescueOnMountArreat: 2,
  PrisonOfIce: 3,
  BetrayalOfHarrogath: 4,
  RiteOfPassage: 5,
  EveOfDestruction: 6,
}

/** 3 难度常量（用于 Quest/Waypoint tab 的 折叠分组） */
export const DIFFICULTIES = [
  { id: 0, code: 'normal',    label: '普通',  labelEn: 'Normal' },
  { id: 1, code: 'nightmare', label: '恶梦',  labelEn: 'Nightmare' },
  { id: 2, code: 'hell',      label: '地狱',  labelEn: 'Hell' },
] as const

export type DifficultyId = 0 | 1 | 2
