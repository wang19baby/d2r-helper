import { skillName } from '../../data/skills'

const TAB_NAMES_ZHCN: Record<string, [string, string, string]> = {
  Amazon: ['弓和十字弓', '被动与魔法', '标枪和长矛'],
  Sorceress: ['火焰技能', '闪电技能', '冰霜技能'],
  Necromancer: ['诅咒', '白骨和毒素', '召唤技能'],
  Paladin: ['战斗技能', '攻击光环', '防御光环'],
  Barbarian: ['战斗技能', '战斗专精', '战嗥'],
  Druid: ['召唤技能', '变形技能', '元素技能'],
  Assassin: ['陷阱', '影子训练', '武学技能'],
  Warlock: ['符印/毁灭', '咒术/邪能', '召唤/仆从'],
}

const TAB_NAMES_EN: Record<string, [string, string, string]> = {
  Amazon: ['Bow & Crossbow', 'Passive & Magic', 'Javelin & Spear'],
  Sorceress: ['Fire', 'Lightning', 'Cold'],
  Necromancer: ['Curses', 'Poison & Bone', 'Summoning'],
  Paladin: ['Combat', 'Offensive Auras', 'Defensive Auras'],
  Barbarian: ['Combat', 'Masteries', 'Warcries'],
  Druid: ['Summoning', 'Shape Shifting', 'Elemental'],
  Assassin: ['Traps', 'Shadow Disciplines', 'Martial Arts'],
  Warlock: ['Sigils', 'Eldritch', 'Summoning'],
}

interface Props {
  skills: { id: number; level: number }[]
  class_en: string
  language?: string
}

/**
 * CharacterSkills — 技能列表面板，按 3 系分组显示
 */
export default function CharacterSkills({ skills, class_en, language }: Props) {
  const real = skills.filter(s => s.id > 0 || s.level > 0)
  if (!real.length) {
    return (
      <div style={{
        color: 'var(--color-d2emu-muted, #888)',
        font: '600 14px/1 "Source Sans 3", sans-serif',
        letterSpacing: '0.08em', textTransform: 'uppercase',
        padding: 14, textAlign: 'center', fontStyle: 'italic', fontSize: 14,
      }}>
        （未分配技能）
      </div>
    )
  }
  const sorted = [...real].sort((a, b) => a.id - b.id)
  const tabs = language === 'zhCN'
    ? (TAB_NAMES_ZHCN[class_en] || ['技能组1', '技能组2', '技能组3'])
    : (TAB_NAMES_EN[class_en] || ['Tab 1', 'Tab 2', 'Tab 3'])
  const rawGroups = [sorted.filter(s => s.id < 10), sorted.filter(s => s.id >= 10 && s.id < 20), sorted.filter(s => s.id >= 20)]
  // Warlock 职业布局索引 0↔2 交换，分组顺序同步
  const groups = class_en === 'Warlock' ? [rawGroups[2], rawGroups[1], rawGroups[0]] : rawGroups

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      {groups.map((group, gi) => group.length > 0 ? (
        <div key={gi}>
          <div style={{
            color: 'var(--color-d2emu-gold, #FBB13A)',
            font: '600 14px/1 "Source Sans 3", sans-serif',
            padding: '4px 0 4px', marginBottom: 2,
            borderBottom: '1px solid var(--color-d2emu-line, #252525)',
          }}>
            {tabs[gi]}
          </div>
          <div style={{
            display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
            gap: '2px 20px', padding: '4px 0',
          }}>
            {group.map(s => (
              <div key={s.id} style={{ display: 'flex', justifyContent: 'space-between', gap: 10, padding: '3px 0' }}>
                <span style={{
                  color: 'var(--color-d2emu-text, #e8e8e8)',
                  font: '500 14px/1.6 "Source Sans 3", sans-serif',
                  overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
                }}>
                  {skillName(class_en, s.id)}
                </span>
                <span style={{
                  color: s.level >= 20 ? '#d4a837' : 'var(--color-d2emu-muted, #888)',
                  font: '600 14px/1.6 "Source Sans 3", sans-serif', flexShrink: 0,
                }}>
                  {s.level}
                </span>
              </div>
            ))}
          </div>
        </div>
      ) : null)}
    </div>
  )
}
