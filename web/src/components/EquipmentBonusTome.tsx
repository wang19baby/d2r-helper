/**
 * EquipmentBonusTome — 装备技能加层聚合 (v2 P0 §1.3.1)
 *
 * 聚合 character.equipment[*].skill_bonuses,按 kind 分三类:
 *   - skill_tab      (+N 技能系)
 *   - chance_to_cast (X% 几率施放)
 *   - skill_charges  (N/M 充能次数)
 *
 * 沿用 d2emu-section-title-arcane + skill-bonus-list 装饰 (index.css 已存在)。
 *
 * 数据契约见 types.ts::SkillBonus / utils/skillPresentation.ts::classifySkillBonuses
 */

import { useMemo } from 'react'
import type { JSX } from 'react'
import type { SkillBonus } from '../types'
import { classifySkillBonuses } from '../utils/skillPresentation'
import { skillTabName } from '../utils/skillTabs'

export interface EquipmentBonusTomeProps {
  bonuses: SkillBonus[]
  language?: string
  /** 角色职业 (class_en, e.g. "Barbarian") — 用于把 skill_tab 编号解析为名字 */
  characterClass?: string
}

const T = {
  zhCN: {
    title: '装备加层',
    none: '未识别到装备技能词缀',
    tabBonus: '技能系',
    notApplied: '未计入硬点',
    chance: '几率施放',
    charges: '充能',
    singleSkill: '技能',
    skillId: '技能 ID',
    level: '等级',
  },
  enUS: {
    title: 'Equipment bonuses',
    none: 'No equipment skill effects detected',
    tabBonus: 'Skill tab',
    notApplied: 'not applied to hard points',
    chance: 'chance to cast',
    charges: 'charges',
    singleSkill: 'skill',
    skillId: 'Skill ID',
    level: 'Level',
  },
}

export default function EquipmentBonusTome({
  bonuses,
  language,
  characterClass,
}: EquipmentBonusTomeProps): JSX.Element {
  const t = language === 'enUS' ? T.enUS : T.zhCN
  const groups = useMemo(() => classifySkillBonuses(bonuses), [bonuses])

  if (groups.totalCount === 0) {
    return (
      <section className="d2emu-card" aria-label={t.title}>
        <h3 className="d2emu-section-title-arcane">
          <i className="fa-solid fa-hat-wizard" style={{ marginRight: 6 }} />
          {t.title}
        </h3>
        <p className="d2emu-lede" style={{ marginTop: 8 }}>{t.none}</p>
      </section>
    )
  }

  return (
    <section className="d2emu-card" aria-label={t.title} style={{ marginBottom: 12 }}>
      <h3 className="d2emu-section-title-arcane">
        <i className="fa-solid fa-hat-wizard" style={{ marginRight: 6 }} />
        {t.title}
        <span style={{
          marginLeft: 10,
          font: '600 14px/1 "IM Fell English SC", serif',
          color: 'var(--color-d2emu-muted, #aaa)',
        }}>
          {groups.totalCount}
        </span>
      </h3>
      <ul className="skill-bonus-list" style={{ marginTop: 6 }}>
        {groups.skillTab.map((b, i) => (
          <li key={`tab-${b.skill_id ?? b.skill_tab ?? 'n'}-${i}`}>
            <span>{skillTabName(characterClass, b.skill_tab, language)}</span>
            <strong>+{b.skill_level ?? 0}</strong>
            <small>{t.notApplied}</small>
          </li>
        ))}
        {groups.chanceToCast.map((b, i) => (
          <li key={`ctc-${b.skill_id ?? 'n'}-${i}`}>
            <span>{b.chance_pct ?? 0}% {t.chance}</span>
            <strong>{t.skillId} {b.skill_id ?? '—'}</strong>
            <small>{t.level} {b.skill_level ?? 0}</small>
          </li>
        ))}
        {groups.skillCharges.map((b, i) => (
          <li key={`chg-${b.skill_id ?? 'n'}-${i}`}>
            <span>{t.skillId} {b.skill_id ?? '—'}</span>
            <strong>{b.current_charges ?? 0}/{b.max_charges ?? 0} {t.charges}</strong>
            <small>{t.level} {b.skill_level ?? 0}</small>
          </li>
        ))}
        {groups.singleSkill.map((b, i) => (
          <li key={`ss-${b.skill_id ?? 'n'}-${i}`}>
            <span>{b.skill_name ? `+${b.skill_level ?? 0} ${b.skill_name}` : `${t.singleSkill} ${b.skill_id ?? '—'}`}</span>
            {!b.skill_name && <strong>+{b.skill_level ?? 0}</strong>}
          </li>
        ))}
      </ul>
    </section>
  )
}
