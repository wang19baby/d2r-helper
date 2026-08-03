import { useState } from 'react'
import type { CharacterInfo } from '../../types'
import { WAYPOINTS, ACT_WAYPOINT_BOUNDS } from '../../data/waypoints'
import { questsByAct, computeQuestBonuses, DIFFICULTIES, type D2RAct } from '../../data/quests'
import { questIconUrl } from '../../utils/questIcons'
import TabBar, { type TabBarItem } from '../../components/TabBar'

const LABEL: React.CSSProperties = {
  color: 'var(--color-d2emu-muted, #888)', font: '600 14px/1 "Source Sans 3", sans-serif',
  letterSpacing: '0.08em', textTransform: 'uppercase', whiteSpace: 'nowrap',
}

/* ── Pad bits array to 39 length ── */
function padBits(bits: boolean[]): boolean[] {
  if (bits.length >= 39) return bits
  const filled = new Array(39).fill(false)
  for (let i = 0; i < bits.length && i < 39; i++) filled[i] = bits[i]
  return filled
}
/* ── Waypoints ── */
export function CharacterWaypoints({ waypoints, language }: { waypoints: CharacterInfo['waypoints']; language?: string }) {
  const [activeDiff, setActiveDiff] = useState(0)
  const entries: { id: number; diff: string; bits: boolean[] }[] = [
    { id: 0, diff: '普通', bits: padBits(waypoints?.normal ?? []) },
    { id: 1, diff: '恶梦', bits: padBits(waypoints?.nightmare ?? []) },
    { id: 2, diff: '地狱', bits: padBits(waypoints?.hell ?? []) },
  ]
  const current = entries[activeDiff]
  return (
    <div style={{ padding: '6px 0', maxWidth: 960 }}>
      <div className="d2emu-section-title" style={{ marginBottom: 8 }}>小站</div>
      <TabBar
        variant="sub"
        activeId={activeDiff}
        onChange={id => setActiveDiff(Number(id))}
        items={entries.map(e => ({ id: e.id, label: e.diff, count: e.bits.filter(Boolean).length }))}
      />
      <div style={{ marginTop: 12, display: 'flex', gap: 8 }}>
        {ACT_WAYPOINT_BOUNDS.map(([lo, hi, _act, label]) => (
          <div key={lo}>
            <div style={{ ...LABEL, fontSize: 14, marginBottom: 4, textAlign: 'center' }}>{label}</div>
            {WAYPOINTS.slice(lo, hi).map((wp, bi) => {
              const on = current.bits[lo + bi]
              return (
                <div key={bi} className="d2emu-progress-cell" style={{
                  borderColor: on ? 'rgba(251,177,58,0.3)' : 'var(--color-d2emu-line-soft, #1a1a1a)',
                  background: on ? 'rgba(251,177,58,0.06)' : 'transparent',
                }}>
                  <span style={{ display: 'inline-block', width: 8, height: 8, borderRadius: '50%', flexShrink: 0,
                    background: on ? 'var(--color-d2emu-gold, #FBB13A)' : '#333' }} />
                  <span style={{ font: '400 14px/1 "Source Sans 3", sans-serif',
                    color: on ? 'var(--color-d2emu-text, #e8e8e8)' : '#555',
                    overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {language === 'zhCN' && wp.name_cn ? wp.name_cn : wp.name}
                  </span>
                </div>
              )
            })}
          </div>
        ))}
      </div>
    </div>
  )
}

/* ── Quest bonus summary ── */
function QuestBonusSummary({ quests }: { quests: CharacterInfo['quests'] }) {
  const byDiff = DIFFICULTIES.map(d => {
    const list = (quests ?? []).filter(q => q.difficulty === d.id)
    const b = computeQuestBonuses(list)
    const completed = list.filter(q => q.completed).length
    const total = list.length
    return { ...d, ...b, completed, total }
  })
  const totalStat = byDiff.reduce((s, d) => s + d.stat, 0)
  const totalSkill = byDiff.reduce((s, d) => s + d.skill, 0)
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, minmax(0, 1fr))', gap: 8, marginBottom: 14 }}>
      {byDiff.map(d => (
        <div key={d.id} style={{ padding: '8px 10px', border: '1px solid var(--color-d2emu-line, #252525)',
          borderRadius: 4, background: 'rgba(0,0,0,0.2)' }}>
          <div style={{ color: 'var(--color-d2emu-muted, #888)', font: '600 14px/1 "Source Sans 3", sans-serif',
            letterSpacing: '0.08em', textTransform: 'uppercase', marginBottom: 4 }}>
            {d.label}
          </div>
          <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
            <span style={{ color: 'var(--color-d2emu-gold, #FBB13A)', font: '700 18px/1 "Source Sans 3", sans-serif',
              fontVariantNumeric: 'tabular-nums' }}>{d.completed}/{d.total}</span>
            <span style={{ color: 'var(--color-d2emu-muted, #888)', font: '400 14px/1 sans-serif' }}>完成</span>
          </div>
          {(d.stat > 0 || d.skill > 0) && (
            <div style={{ marginTop: 4, fontSize: 14, color: 'var(--color-d2emu-text, #e8e8e8)', display: 'flex', gap: 8 }}>
              {d.stat > 0 && <span style={{ color: 'var(--color-d2emu-blue, #4f83c7)' }}>+{d.stat} 属性</span>}
              {d.skill > 0 && <span style={{ color: 'var(--color-d2emu-orange, #b87020)' }}>+{d.skill} 技能</span>}
            </div>
          )}
        </div>
      ))}
      <div style={{ padding: '8px 10px', border: '1px solid var(--color-d2emu-gold, #FBB13A)', borderRadius: 4,
        background: 'rgba(251,177,58,0.06)' }}>
        <div style={{ color: 'var(--color-d2emu-gold, #FBB13A)', font: '600 14px/1 "Source Sans 3", sans-serif',
          letterSpacing: '0.08em', textTransform: 'uppercase', marginBottom: 4 }}>总奖励</div>
        <div style={{ color: 'var(--color-d2emu-gold-bright, #fff)', font: '700 18px/1 "Source Sans 3", sans-serif',
          fontVariantNumeric: 'tabular-nums', display: 'flex', gap: 10 }}>
          <span style={{ color: 'var(--color-d2emu-blue, #4f83c7)' }}>+{totalStat} 属性</span>
          <span style={{ color: 'var(--color-d2emu-orange, #b87020)' }}>+{totalSkill} 技能</span>
        </div>
        <div style={{ marginTop: 4, fontSize: 14, color: 'var(--color-d2emu-muted, #888)' }}>(任务奖励)</div>
      </div>
    </div>
  )
}

const ACT_IDS = [1, 2, 3, 4, 5] as const
const QUEST_NAMES_BY_ACT: [number, string, string[]][] = ACT_IDS.map(act => {
  const names = questsByAct(act).map(q => q.name)
  return [act, `A${act}`, names]
})

/* ── Quests ── */
export function CharacterQuests({ quests, language }: { quests: CharacterInfo['quests']; language?: string }) {
  const [activeDiff, setActiveDiff] = useState(0)
  const byDiff = [0, 1, 2].map(d => (quests ?? []).filter(q => q.difficulty === d))
  const diffLabels = ['普通', '恶梦', '地狱']
  return (
    <div style={{ padding: '6px 0', maxWidth: 960 }}>
      <div className="d2emu-section-title" style={{ marginBottom: 8 }}>任务进度</div>
      <QuestBonusSummary quests={quests ?? []} />
      <div style={{ marginTop: 8 }}>
        <TabBar
          variant="sub"
          activeId={activeDiff}
          onChange={id => setActiveDiff(Number(id))}
          items={byDiff.map((group, di) => ({ id: di, label: diffLabels[di], count: group.filter(q => q.completed).length }))}
        />
        <div style={{ marginTop: 12, display: 'flex', gap: 8 }}>
          {QUEST_NAMES_BY_ACT.map(([act, label, names]) => {
            const actQ = byDiff[activeDiff].filter(q => q.act === act)
            return (
              <div key={act}>
                <div style={{ ...LABEL, fontSize: 14, marginBottom: 4, textAlign: 'center' }}>{label}</div>
                {names.map((n, bi) => {
                  const entry = actQ.find(q => q.quest_id === bi)
                  const done = entry?.completed ?? false
                  const iconUrl = questIconUrl(act, bi + 1)
                  return (
                    <div key={bi} className="d2emu-progress-cell" style={{
                      borderColor: done ? 'rgba(76,175,80,0.3)' : 'var(--color-d2emu-line-soft, #1a1a1a)',
                      background: done ? 'rgba(76,175,80,0.06)' : 'transparent',
                    }}>
                      <span style={{ display: 'inline-block', width: 8, height: 8, borderRadius: '50%', flexShrink: 0,
                        background: done ? '#4caf50' : '#333' }} />
                      {iconUrl ? <img src={iconUrl} alt="" aria-hidden="true" style={{ width: 22, height: 22, flexShrink: 0, imageRendering: 'pixelated', objectFit: 'contain' }} />
                        : <span style={{ width: 22, height: 22, flexShrink: 0 }} />}
                      <span style={{ font: '400 14px/1 "Source Sans 3", sans-serif',
                        color: done ? 'var(--color-d2emu-text, #e8e8e8)' : '#555',
                        overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flex: 1, minWidth: 0 }}>
                        {language === 'zhCN' ? (questsByAct(act as D2RAct)[bi]?.name_cn || n) : n}
                      </span>
                    </div>
                  )
                })}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}

/** Safe bit test for positions >31 (JS 32-bit overflow workaround) */
function testBit(n: number, bit: number): boolean {
  if (bit < 32) return !!((n >>> bit) & 1)
  // For bit >= 32, manually compute the higher word
  const hi = Math.floor(n / 0x100000000)
  return !!((hi >>> (bit - 32)) & 1)
}

/** w4 NPC 名称 (D2SLib NPCDialogs.cs 顺序) */
const W4_NPC_NAMES: [string, string][] = [
  [ '0', 'Warriv(A2)' ],  [ '1', '-01' ],       [ '2', 'Charsi' ],      [ '3', 'Warriv(A1)' ],
  [ '4', 'Kashya' ],      [ '5', 'Akara' ],       [ '6', 'Gheed' ],       [ '7', '-07' ],
  [ '8', 'Greiz' ],       [ '9', 'Jerhyn' ],      ['10', 'Meshif(A2)' ],  ['11', 'Geglash' ],
  ['12', 'Lysander' ],    ['13', 'Fara' ],        ['14', 'Drogan' ],      ['15', '-0F' ],
  ['16', 'Alkor' ],       ['17', 'Hratli' ],      ['18', 'Ashera' ],      ['19', '-13' ],
  ['20', '-14' ],         ['21', 'Cain(A3)' ],    ['22', '-16' ],         ['23', 'Elzix' ],
  ['24', 'Malah' ],       ['25', 'Anya' ],        ['26', '-1A' ],         ['27', 'Natalya' ],
  ['28', 'Meshif(A3)' ],  ['29', '-1D' ],         ['30', '-1F' ],         ['31', 'Ormus' ],
  ['32', '-21' ],         ['33', '-22' ],         ['34', '-23' ],         ['35', '-24' ],
  ['36', '-25' ],         ['37', 'Cain(A5)' ],    ['38', 'Qualkehk' ],    ['39', 'Nihlathak' ],
  ['40', '-29' ],
]
const DIFF_KEYS = ['normal', 'nightmare', 'hell'] as const

export function CharacterRewards({ w4, quests }: { w4?: CharacterInfo['w4']; quests?: CharacterInfo['quests'] }) {
  if (!w4) return null
  // NPC 奖励基于任务完成度（woo quest data），非 w4 bitmask
  // CLI 使用 data_idx(1-indexed data 位置), 但 QuestEntry.quest_id = display 顺序
  // Act I [1,2,3,5,4,6]: data_idx=1 → display pos 0 (邪恶洞穴)
  // Act V [3,4,5,6,8,7]: data_idx=3 → display pos 0 (哈洛加斯围城战)
  //                     data_idx=4 → display pos 1 (亚瑞特山的救援)
  const REWARD_QUESTS: [number, number, string][] = [
    [1, 0, 'Charsi打造'],      // Act I  Den of Evil → quest_id=0
    [5, 0, 'Qualkehk打孔'],    // Act V  Siege on Harrogath → quest_id=0
    [5, 1, 'Anya命名'],        // Act V  Rescue on Mount Arreat → quest_id=1
  ]
  const DIFF_KEYS = ['normal', 'nightmare', 'hell'] as const
  const DIFF_LABELS = ['Normal', 'Nightmare', 'Hell']

  const DIFF_INDEX: Record<string, number> = { normal: 0, nightmare: 1, hell: 2 }

  return (
    <div style={{ padding: '6px 0', maxWidth: 960 }}>
      <div className="d2emu-section-title" style={{ marginBottom: 8 }}>NPC 奖励</div>
      {DIFF_KEYS.map((diff, di) => {
        return (
          <div key={diff} style={{ marginBottom: 8 }}>
            <div style={{ color: '#FBB13A', fontWeight: 600, marginBottom: 2, fontSize: 13 }}>
              [{DIFF_LABELS[di]}]
            </div>
            {REWARD_QUESTS.map(([act, questId, label]) => {
              const done = !!(quests?.find(q =>
                q.difficulty === di && q.act === act && q.quest_id === questId && q.completed
              ))
              const text = done ? `可用 ${label}` : `未解锁 ${label}`
              const color = done ? '#4caf50' : '#666'
              return <div key={act + '_' + questId} style={{ color, fontSize: 13, marginLeft: 16 }}>{text}</div>
            })}
            {(() => {
              const denOfEvilDone = !!(quests?.find(q =>
                q.difficulty === di && q.act === 1 && q.quest_id === 0 && q.completed
              ))
              const respecText = denOfEvilDone
                ? '可用 重置属性/技能点'
                : '未解锁 重置属性/技能点（需完成邪恶洞穴）'
              return <div style={{ color: denOfEvilDone ? '#4caf50' : '#666', fontSize: 13, marginLeft: 16 }}>{respecText}</div>
            })()}
          </div>
        )
      })}
    </div>
  )
}

/* ── Mercenary ── */
export function CharacterMerc() {
  return (
    <div style={{ width: '100%', padding: 10, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 14 }}>
      <div style={{ ...LABEL, fontSize: 14, textAlign: 'center' }}>
        <i className="fa-solid fa-shield-halved" style={{ marginRight: 6 }} />佣兵系统
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 100px)', gap: 10,
        padding: 20, background: 'rgba(0,0,0,0.2)', border: '1px solid var(--color-d2emu-line, #252525)', borderRadius: 8 }}>
        {['武器', '盾牌', '头盔', '盔甲'].map(slot => (
          <div key={slot} className="d2emu-item-slot" style={{ width: 100, height: 100, flexDirection: 'column', gap: 6 }}>
            <i className="fa-regular fa-circle" style={{ fontSize: 22, color: 'var(--color-d2emu-muted, #555)', opacity: 0.4 }} />
            <span style={{ ...LABEL, fontSize: 14, color: '#555' }}>{slot}</span>
          </div>
        ))}
      </div>
      <div style={{ ...LABEL, textAlign: 'center', fontStyle: 'italic', color: '#555', fontSize: 14 }}>
        （需要解析 .d2s 佣兵数据）
      </div>
    </div>
  )
}
