import type { SkillBonus, SkillEntry } from '../types.ts'
import { SKILL_TREE_LAYOUTS } from '../data/skillTreeLayouts.ts'

/** 职业内 0-based skill id 最大值 (3 棵 tab × 10 = 30,Warlock/扩展职业可能突破时调整) */
export const MAX_SKILL_ID = 30

/** 该职业是否在 skillTreeLayouts 里有完整三塔布局 (Sorceress/Paladin/Assassin/Druid 等) */
export function hasSkillTreeLayout(classEn: string | undefined): boolean {
  if (!classEn) return false
  return Array.isArray(SKILL_TREE_LAYOUTS[classEn]) && SKILL_TREE_LAYOUTS[classEn].length > 0
}

export interface SkillSummary {
  levels: Map<number, number>
  totalHardPoints: number
  investedSkillCount: number
  pointsByTree: [number, number, number]
}

export interface ClassifiedSkillBonuses {
  skillTab: SkillBonus[]
  chanceToCast: SkillBonus[]
  skillCharges: SkillBonus[]
  singleSkill: SkillBonus[]
  totalCount: number
}

export interface TooltipStatLike {
  tab: number
  line: number
  label: string
  value?: string
  calc?: string
}

export interface PreparedTooltipStats {
  rows: { label: string; value: string }[]
  unresolvedCount: number
}

export interface DirectionalSkillNode {
  skillId: number
  x: number
  y: number
}

export type SkillDirectionKey = 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'

export function summarizeSkills(skills: SkillEntry[]): SkillSummary {
  const levels = new Map<number, number>()

  for (const skill of skills) {
    if (skill.id < 0 || skill.id > MAX_SKILL_ID - 1 || skill.level <= 0) continue
    levels.set(skill.id, (levels.get(skill.id) ?? 0) + skill.level)
  }

  let totalHardPoints = 0
  const pointsByTree: [number, number, number] = [0, 0, 0]

  for (const [id, level] of levels) {
    totalHardPoints += level
    pointsByTree[Math.min(2, Math.floor(id / 10))] += level
  }

  return {
    levels,
    totalHardPoints,
    investedSkillCount: levels.size,
    pointsByTree,
  }
}

export function classifySkillBonuses(bonuses: SkillBonus[]): ClassifiedSkillBonuses {
  const skillTab: SkillBonus[] = []
  const chanceToCast: SkillBonus[] = []
  const skillCharges: SkillBonus[] = []
  const singleSkill: SkillBonus[] = []

  for (const bonus of bonuses) {
    if (bonus.kind === 'skill_tab') skillTab.push(bonus)
    else if (bonus.kind === 'chance_to_cast') chanceToCast.push(bonus)
    else if (bonus.kind === 'skill_charges') skillCharges.push(bonus)
    else if (bonus.kind === 'single_skill') singleSkill.push(bonus)
  }

  return {
    skillTab,
    chanceToCast,
    skillCharges,
    singleSkill,
    totalCount: skillTab.length + chanceToCast.length + skillCharges.length + singleSkill.length,
  }
}
export function prepareTooltipStats(
  stats: TooltipStatLike[] | undefined,
  tab: number,
): PreparedTooltipStats {
  const rows: PreparedTooltipStats['rows'] = []
  let unresolvedCount = 0

  for (const stat of stats ?? []) {
    if (stat.tab !== tab || !stat.label) continue
    if (stat.value) rows.push({ label: stat.label, value: stat.value })
    else if (stat.calc) unresolvedCount += 1
  }

  return { rows, unresolvedCount }
}

export function findDirectionalNode(
  nodes: DirectionalSkillNode[],
  currentSkillId: number,
  key: SkillDirectionKey,
): number | null {
  const current = nodes.find(node => node.skillId === currentSkillId)
  if (!current) return null

  const horizontal = key === 'ArrowLeft' || key === 'ArrowRight'
  const forward = key === 'ArrowRight' || key === 'ArrowDown'
  let best: { id: number; score: number } | null = null

  for (const node of nodes) {
    if (node.skillId === currentSkillId) continue

    const axisDelta = horizontal ? node.x - current.x : node.y - current.y
    if ((forward && axisDelta <= 0) || (!forward && axisDelta >= 0)) continue

    const crossDelta = horizontal ? Math.abs(node.y - current.y) : Math.abs(node.x - current.x)
    const score = Math.abs(axisDelta) + crossDelta * 2
    if (!best || score < best.score) best = { id: node.skillId, score }
  }

  return best?.id ?? null
}
