import { test } from 'node:test'
import assert from 'node:assert/strict'
import { skillTabName, SKILL_TAB_NAMES } from '../src/utils/skillTabs.ts'

test('skill tab names cover all 8 classes', () => {
  const classes = ['amazon', 'assassin', 'barbarian', 'druid', 'necromancer', 'paladin', 'sorceress', 'warlock']
  for (const c of classes) {
    assert.equal(SKILL_TAB_NAMES[c].length, 3, `${c} should have 3 tabs`)
  }
})

test('barbarian tab order follows in-game UI: Combat Skills, Masteries, Warcries', () => {
  assert.equal(skillTabName('Barbarian', 0, 'zhCN'), '战斗技能')
  assert.equal(skillTabName('Barbarian', 0, 'enUS'), 'Combat Skills')
  assert.equal(skillTabName('Barbarian', 1, 'zhCN'), '战斗精通')
  assert.equal(skillTabName('Barbarian', 1, 'enUS'), 'Combat Masteries')
  assert.equal(skillTabName('Barbarian', 2, 'zhCN'), '呐喊技能')
  assert.equal(skillTabName('Barbarian', 2, 'enUS'), 'Warcries')
})

test('paladin tab order: Combat Skills, Offensive Auras, Defensive Auras', () => {
  assert.equal(skillTabName('Paladin', 0, 'zhCN'), '战斗技能')
  assert.equal(skillTabName('Paladin', 1, 'zhCN'), '攻击光环')
  assert.equal(skillTabName('Paladin', 2, 'zhCN'), '防御光环')
})

test('assassin tab order: Traps, Shadow Disciplines, Martial Arts', () => {
  assert.equal(skillTabName('Assassin', 0, 'zhCN'), '陷阱')
  assert.equal(skillTabName('Assassin', 1, 'zhCN'), '影身训练')
  assert.equal(skillTabName('Assassin', 2, 'zhCN'), '武学技艺')
})

test('warlock (mod class) tabs resolve', () => {
  assert.equal(skillTabName('Warlock', 2, 'zhCN'), '混沌')
  assert.equal(skillTabName('Warlock', 0, 'enUS'), 'Demon')
})

test('unknown class falls back to tab index', () => {
  assert.equal(skillTabName('Unknown', 2, 'zhCN'), '技能页 2')
  assert.equal(skillTabName('Unknown', 0, 'enUS'), 'Skill Tab 0')
  assert.equal(skillTabName(undefined, null, 'zhCN'), '技能页')
})

test('case-insensitive class matching', () => {
  assert.equal(skillTabName('barbarian', 2, 'zhCN'), '呐喊技能')
})
