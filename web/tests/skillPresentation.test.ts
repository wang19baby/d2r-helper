import test from 'node:test'
import assert from 'node:assert/strict'

import {
  classifySkillBonuses,
  findDirectionalNode,
  prepareTooltipStats,
  summarizeSkills,
} from '../src/utils/skillPresentation.ts'
import type { SkillBonus, SkillEntry } from '../src/types.ts'

test('summarizeSkills aggregates hard points and tree totals', () => {
  const skills: SkillEntry[] = [
    { id: 0, level: 2 },
    { id: 4, level: 3 },
    { id: 4, level: 2 },
    { id: 18, level: 4 },
    { id: 25, level: 1 },
    { id: 29, level: 0 },
  ]

  const summary = summarizeSkills(skills)

  assert.equal(summary.levels.get(0), 2)
  assert.equal(summary.levels.get(4), 5)
  assert.equal(summary.totalHardPoints, 12)
  assert.equal(summary.investedSkillCount, 4)
  assert.deepEqual(summary.pointsByTree, [7, 4, 1])
})

test('classifySkillBonuses keeps equipment effects separate from hard points', () => {
  const bonuses: SkillBonus[] = [
    { stat_id: 188, kind: 'skill_tab', skill_tab: 2, skill_level: 3 },
    { stat_id: 198, kind: 'chance_to_cast', skill_id: 64, skill_level: 5, chance_pct: 10 },
    { stat_id: 204, kind: 'skill_charges', skill_id: 54, skill_level: 1, current_charges: 8, max_charges: 12 },
  ]

  const groups = classifySkillBonuses(bonuses)

  assert.equal(groups.totalCount, 3)
  assert.deepEqual(groups.skillTab, [bonuses[0]])
  assert.deepEqual(groups.chanceToCast, [bonuses[1]])
  assert.deepEqual(groups.skillCharges, [bonuses[2]])
})

test('prepareTooltipStats renders resolved values and counts unresolved formulas', () => {
  const result = prepareTooltipStats([
    { tab: 1, line: 1, label: 'Walk/Run Speed:', calc: 'dm91' },
    { tab: 1, line: 2, label: 'Damage:', value: '12-20' },
    { tab: 3, line: 3, label: 'Synergy:', value: '+5%' },
    { tab: 1, line: 4, label: '', value: 'ignored' },
  ], 1)

  assert.deepEqual(result.rows, [{ label: 'Damage:', value: '12-20' }])
  assert.equal(result.unresolvedCount, 1)
})

test('findDirectionalNode selects the nearest aligned node in the requested direction', () => {
  const nodes = [
    { skillId: 1, x: 50, y: 50 },
    { skillId: 2, x: 80, y: 50 },
    { skillId: 3, x: 60, y: 80 },
    { skillId: 4, x: 20, y: 50 },
    { skillId: 5, x: 50, y: 20 },
  ]

  assert.equal(findDirectionalNode(nodes, 1, 'ArrowRight'), 2)
  assert.equal(findDirectionalNode(nodes, 1, 'ArrowLeft'), 4)
  assert.equal(findDirectionalNode(nodes, 1, 'ArrowUp'), 5)
  assert.equal(findDirectionalNode(nodes, 2, 'ArrowRight'), null)
})
