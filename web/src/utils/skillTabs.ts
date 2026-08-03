/**
 * Skill tab names per character class (D2R skillcategory{class}{n} data).
 * tab index (0-based, from stat 188 param & 0x7) → name, per class.
 * 数据源: localized_string skillcategory{class}{1..3} (已验证 8 职业完整)。
 */
export const SKILL_TAB_NAMES: Record<string, { zhCN: string; enUS: string }[]> = {
  amazon: [
    { zhCN: '标枪和长矛', enUS: 'Javelin and Spear' },
    { zhCN: '被动和魔法', enUS: 'Passive and Magic' },
    { zhCN: '弓和十字弩', enUS: 'Bow and Crossbow' },
  ],
  assassin: [
    { zhCN: '陷阱', enUS: 'Traps' },
    { zhCN: '影身训练', enUS: 'Shadow Disciplines' },
    { zhCN: '武学技艺', enUS: 'Martial Arts' },
  ],
  barbarian: [
    { zhCN: '战斗技能', enUS: 'Combat Skills' },
    { zhCN: '战斗精通', enUS: 'Combat Masteries' },
    { zhCN: '呐喊技能', enUS: 'Warcries' },
  ],
  druid: [
    { zhCN: '元素', enUS: 'Elemental' },
    { zhCN: '变形', enUS: 'Shape Shifting' },
    { zhCN: '召唤', enUS: 'Summoning' },
  ],
  necromancer: [
    { zhCN: '召唤术', enUS: 'Summoning' },
    { zhCN: '毒素和白骨术', enUS: 'Poison and Bone' },
    { zhCN: '诅咒术', enUS: 'Curses' },
  ],
  paladin: [
    { zhCN: '战斗技能', enUS: 'Combat Skills' },
    { zhCN: '攻击光环', enUS: 'Offensive Auras' },
    { zhCN: '防御光环', enUS: 'Defensive Auras' },
  ],
  sorceress: [
    { zhCN: '冰系法术', enUS: 'Cold Spells' },
    { zhCN: '雷系法术', enUS: 'Lightning Spells' },
    { zhCN: '火系法术', enUS: 'Fire Spells' },
  ],
  warlock: [
    { zhCN: '恶魔', enUS: 'Demon' },
    { zhCN: '邪术', enUS: 'Eldritch' },
    { zhCN: '混沌', enUS: 'Chaos' },
  ],
}

/** Resolve skill tab display name. Falls back to `技能页 N` / `Skill Tab N`. */
export function skillTabName(
  characterClass: string | undefined,
  tabIndex: number | null | undefined,
  language?: string,
): string {
  const en = language === 'enUS'
  const cls = (characterClass ?? '').toLowerCase()
  const tabs = SKILL_TAB_NAMES[cls]
  if (tabs && tabIndex != null && tabIndex >= 0 && tabIndex < tabs.length) {
    return en ? tabs[tabIndex].enUS : tabs[tabIndex].zhCN
  }
  return tabIndex != null
    ? en ? `Skill Tab ${tabIndex}` : `技能页 ${tabIndex}`
    : en ? 'Skill Tab' : '技能页'
}
