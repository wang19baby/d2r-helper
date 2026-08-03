import type { SkillBonus } from '../types'
import {
  classifySkillBonuses,
  prepareTooltipStats,
  type ClassifiedSkillBonuses,
  type TooltipStatLike,
} from '../utils/skillPresentation'

export interface SkillTooltip extends Record<string, unknown> {
  name: string
  desc?: string
  reqLevel: number
  maxLevel?: number
  manaCost?: { base: number; perLevel: number }
  damage?: Record<string, unknown>
  stats: TooltipStatLike[]
  synergies: { skillName: string; perLevel?: string }[]
  params?: Record<string, { label: string; value: number }>
}

export interface SkillDetailsSelection {
  id: number
  name: string
  description?: string
  level: number
  passive: boolean
  tooltip?: SkillTooltip
}

interface SkillDetailsPanelProps {
  selection: SkillDetailsSelection | null
  pinned: boolean
  equipmentBonuses: SkillBonus[]
  language?: string
}

const TEXT = {
  zhCN: {
    title: '技能详情', preview: '预览', pinned: '已固定', empty: '悬停或选择一个技能查看详情',
    hardPoints: '硬点', requiredLevel: '需求等级', passive: '被动技能', currentEffects: '当前效果',
    synergies: '协同技能', mana: '法力消耗', unresolved: '项公式数据尚未解析，已隐藏空值行',
    equipment: '装备技能词缀', tabBonus: '技能系', notApplied: '未计入硬点',
    chance: '几率施放', charges: '充能', skillId: '技能 ID', level: '等级', noBonuses: '未识别到结构化装备技能词缀',
  },
  enUS: {
    title: 'Skill details', preview: 'Preview', pinned: 'Pinned', empty: 'Hover or select a skill to inspect it',
    hardPoints: 'Hard points', requiredLevel: 'Required level', passive: 'Passive', currentEffects: 'Current effects',
    synergies: 'Synergies', mana: 'Mana cost', unresolved: 'formula values are unresolved; empty rows are hidden',
    equipment: 'Equipment skill effects', tabBonus: 'Skill tab', notApplied: 'not applied to hard points',
    chance: 'chance to cast', charges: 'charges', skillId: 'Skill ID', level: 'Level', noBonuses: 'No structured equipment skill effects detected',
  },
}

function bonusKey(bonus: SkillBonus, index: number): string {
  return `${bonus.kind}-${bonus.stat_id}-${bonus.skill_id ?? bonus.skill_tab ?? 'none'}-${index}`
}

function EquipmentBonusList({ groups, language }: { groups: ClassifiedSkillBonuses; language?: string }) {
  const t = language === 'enUS' ? TEXT.enUS : TEXT.zhCN
  if (groups.totalCount === 0) {
    return <p className="skill-details-muted">{t.noBonuses}</p>
  }

  return (
    <ul className="skill-bonus-list">
      {groups.skillTab.map((bonus, index) => (
        <li key={bonusKey(bonus, index)}>
          <span>{t.tabBonus} #{bonus.skill_tab ?? '—'}</span>
          <strong>+{bonus.skill_level ?? 0}</strong>
          <small>{t.notApplied}</small>
        </li>
      ))}
      {groups.chanceToCast.map((bonus, index) => (
        <li key={bonusKey(bonus, index)}>
          <span>{bonus.chance_pct ?? 0}% {t.chance}</span>
          <strong>{t.skillId} {bonus.skill_id ?? '—'}</strong>
          <small>{t.level} {bonus.skill_level ?? 0}</small>
        </li>
      ))}
      {groups.skillCharges.map((bonus, index) => (
        <li key={bonusKey(bonus, index)}>
          <span>{t.skillId} {bonus.skill_id ?? '—'}</span>
          <strong>{bonus.current_charges ?? 0}/{bonus.max_charges ?? 0} {t.charges}</strong>
          <small>{t.level} {bonus.skill_level ?? 0}</small>
        </li>
      ))}
    </ul>
  )
}

export default function SkillDetailsPanel({
  selection,
  pinned,
  equipmentBonuses,
  language,
}: SkillDetailsPanelProps) {
  const t = language === 'enUS' ? TEXT.enUS : TEXT.zhCN
  const groups = classifySkillBonuses(equipmentBonuses)
  const prepared = prepareTooltipStats(selection?.tooltip?.stats, 1)
  const synergyLabels = [
    ...(selection?.tooltip?.synergies ?? []).map(s => `${s.skillName}${s.perLevel ? ` ${s.perLevel}` : ''}`),
    ...(selection?.tooltip?.stats ?? []).filter(s => s.tab === 3 && s.label).map(s => s.label),
  ].filter((label, index, all) => all.indexOf(label) === index)

  return (
    <aside id="skill-details-panel" className="skill-details-panel" aria-labelledby="skill-details-heading">
      <div className="skill-details-heading-row">
        <h3 id="skill-details-heading">{t.title}</h3>
        {selection && <span className={`skill-details-mode${pinned ? ' is-pinned' : ''}`}>{pinned ? t.pinned : t.preview}</span>}
      </div>

      {selection ? (
        <div className="skill-details-content">
          <header className="skill-details-skill-header">
            <span className="skill-details-id">#{selection.id}</span>
            <h4>{selection.name}</h4>
            <strong>
              {t.hardPoints} {selection.level}
              {selection.tooltip?.maxLevel != null ? ` / ${selection.tooltip.maxLevel}` : ''}
            </strong>
          </header>

          <div className="skill-details-meta">
            {selection.tooltip?.reqLevel != null && <span>{t.requiredLevel} {selection.tooltip.reqLevel}</span>}
            {selection.passive && <span>{t.passive}</span>}
            {selection.tooltip?.manaCost && (
              <span>{t.mana} {selection.tooltip.manaCost.base} + {selection.tooltip.manaCost.perLevel}/Lv</span>
            )}
          </div>

          {selection.description && <p className="skill-details-description">{selection.description}</p>}

          {prepared.rows.length > 0 && (
            <section className="skill-details-section">
              <h5>{t.currentEffects}</h5>
              <dl className="skill-effect-list">
                {prepared.rows.map((row, index) => (
                  <div key={`${row.label}-${index}`}>
                    <dt>{row.label}</dt>
                    <dd>{row.value}</dd>
                  </div>
                ))}
              </dl>
            </section>
          )}

          {prepared.unresolvedCount > 0 && (
            <p className="skill-details-formula-note">{prepared.unresolvedCount} {t.unresolved}</p>
          )}

          {synergyLabels.length > 0 && (
            <section className="skill-details-section">
              <h5>{t.synergies}</h5>
              <ul className="skill-synergy-list">
                {synergyLabels.map(label => <li key={label}>{label}</li>)}
              </ul>
            </section>
          )}
        </div>
      ) : (
        <p className="skill-details-empty">{t.empty}</p>
      )}

      <section className="skill-details-section skill-equipment-effects">
        <h5>{t.equipment}</h5>
        <EquipmentBonusList groups={groups} language={language} />
      </section>
    </aside>
  )
}
