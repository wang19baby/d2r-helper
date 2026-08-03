import { useMemo, useState } from 'react'
import D2EmuCard from './D2EmuCard'
import { magicPrefixes, magicSuffixes, rarePrefixes, rareSuffixes, type AffixRecord } from '../data/d2core/affixesIndex'
import { affixNameZh, affixNameDisplay, modCodeZh, ITYPE_ZH } from '../data/d2core/affix_zh'

// 风格常量(与 RunewordCalc 保持一致)
const GOLD = '#c7b377'
const MUTED = '#8a7e5f'
const TEXT = '#e8dcc4'
const LINE = '#2a2a2a'

type AffixKind = 'prefix' | 'suffix'

const SOURCES: Record<AffixKind, AffixRecord[]> = {
  prefix: magicPrefixes,
  suffix: magicSuffixes,
}

const RARE: Record<AffixKind, AffixRecord[]> = {
  prefix: rarePrefixes,
  suffix: rareSuffixes,
}

const RARITY_TABS = [
  { key: 'magic', label: '魔法词缀' },
  { key: 'rare', label: '稀有词缀' },
] as const

const ITYPE_OPTIONS: { value: string; label: string }[] = [
  { value: '__any', label: '全部物品' },
  { value: 'weap', label: '武器' },
  { value: 'armo', label: '防具' },
  { value: 'shld', label: '盾牌' },
  { value: 'helm', label: '头盔' },
  { value: 'belt', label: '腰带' },
  { value: 'boot', label: '靴' },
  { value: 'glov', label: '手套' },
  { value: 'ring', label: '戒指' },
  { value: 'amul', label: '项链' },
  { value: 'circ', label: '头环' },
  { value: 'phlm', label: '野蛮人头' },
  { value: 'pelt', label: '狼头' },
  { value: 'axe', label: '斧' },
  { value: 'swor', label: '剑' },
  { value: 'hamm', label: '锤' },
  { value: 'mace', label: '钉头锤' },
  { value: 'spea', label: '矛' },
  { value: 'pole', label: '长柄' },
  { value: 'bow', label: '弓' },
  { value: 'xbow', label: '弩' },
  { value: 'staf', label: '法杖' },
  { value: 'wand', label: '魔杖' },
]

interface Props {
  initialItemType?: string
  initialLevelRange?: [number, number]
}

function AffixCard({ affix }: { affix: AffixRecord }) {
  const lvl = (affix as any).level as number | undefined
  const itype = affix.itype1 ?? ''
  const { display: nameDisplay, hasZh } = affixNameDisplay(affix.name)
  const mods: { code: string; min?: number; max?: number }[] = []
  for (let n = 1; n <= 9; n++) {
    const c = (affix as any)[`mod${n}code`]
    if (c) mods.push({ code: c, min: (affix as any)[`mod${n}min`], max: (affix as any)[`mod${n}max`] })
    else break
  }
  return (
    <div style={{
      padding: '10px 12px', borderRadius: 6, background: '#0c0a08',
      border: `1px solid ${LINE}`, display: 'flex', flexDirection: 'column', gap: 6,
    }}>
      {/* 词缀名 */}
      <div style={{ fontWeight: 600, color: TEXT, fontSize: 14, lineHeight: 1.3 }}>
        {nameDisplay}
        {!hasZh && (
          <span style={{ fontSize: 10, color: '#555', fontWeight: 400, marginLeft: 6 }}>暂无翻译</span>
        )}
      </div>
      {/* 等级 + 物品类型 */}
      <div className="flex items-center" style={{ gap: 12 }}>
        {lvl != null && (
          <div style={{ fontSize: 13, color: MUTED }}>
            <span style={{ color: TEXT }}>{lvl}</span> 级
          </div>
        )}
        <div style={{ fontSize: 13, color: MUTED }}>
          <span style={{ color: '#6a9fd8' }}>{(ITYPE_ZH[itype] ?? itype) || '?'}</span>
        </div>
      </div>
      {/* 属性 */}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
        {mods.map((m, i) => {
          const zhMod = modCodeZh(m.code)
          const hasModZh = zhMod !== m.code
          return (
            <span key={i} style={{
              padding: '2px 7px', background: '#1f1810', borderRadius: 4, fontSize: 12,
              display: 'inline-flex', alignItems: 'center', gap: 3,
            }}>
              <span style={{ color: GOLD }}>{zhMod}</span>
              <span style={{ color: MUTED }}>
                {m.min !== undefined && m.max !== undefined ? ` ${m.min}-${m.max}` : m.min ?? ''}
              </span>
              {!hasModZh && (
                <span style={{ color: '#555', fontSize: 10, fontFamily: 'monospace' }}>{m.code}</span>
              )}
            </span>
          )
        })}
      </div>
    </div>
  )
}

export default function MagicAffixPanel({ initialItemType, initialLevelRange }: Props) {
  const [rarity, setRarity] = useState<'magic' | 'rare'>('magic')
  const [kind, setKind] = useState<AffixKind>('prefix')
  const [itemType, setItemType] = useState<string>(initialItemType ?? '__any')
  const [keyword, setKeyword] = useState('')
  const [levelMax, setLevelMax] = useState<number>(initialLevelRange?.[1] ?? 99)

  const source = rarity === 'magic' ? SOURCES[kind] : RARE[kind]

  const filtered = useMemo(() => {
    const kw = keyword.trim().toLowerCase()
    return source.filter(it => {
      if (itemType !== '__any') {
        if (it.itype1 !== itemType && it.itype2 !== itemType && it.itype3 !== itemType) {
          return false
        }
      }
      const lvl = (it as any).level as number | undefined
      if (lvl != null && lvl > levelMax) return false
      if (kw && !String(it.name).toLowerCase().includes(kw) && !affixNameZh(it.name).includes(kw)) return false
      return true
    })
  }, [source, itemType, levelMax, keyword])

  const hasAnyFilter = itemType !== '__any' || keyword.trim() || levelMax < 99

  return (
    <div className="font-d2emu-ui" style={{
      display: 'flex', flexDirection: 'column', gap: 12,
      flex: 1, minHeight: 0, overflow: 'hidden',
    }}>
      {/* 筛选区 */}
      <div style={{ flexShrink: 0 }}>
        <D2EmuCard kicker={`词缀库 · ${filtered.length} 条匹配`}
          actions={hasAnyFilter ? (
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
              onClick={() => { setItemType('__any'); setKeyword(''); setLevelMax(99) }}>
              <i className="fa-solid fa-rotate" /> 清除
            </button>
          ) : undefined}
        >
          <div className="flex flex-wrap items-end gap-3" style={{ marginBottom: 6 }}>
            {/* 品质 */}
            <div>
              <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>
                <i className="fa-solid fa-tag" style={{ marginRight: 4 }} />品质
              </div>
              <div className="flex" style={{ gap: 3 }}>
                {RARITY_TABS.map(rt => (
                  <button key={rt.key}
                    onClick={() => setRarity(rt.key as any)}
                    style={{
                      padding: '3px 10px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                      background: rarity === rt.key ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : 'transparent',
                      border: rarity === rt.key ? '1px solid #c7b377' : `1px solid ${LINE}`,
                      color: rarity === rt.key ? GOLD : MUTED,
                    }}>
                    {rt.label}
                  </button>
                ))}
              </div>
            </div>
            {/* 类型(前/后缀) */}
            <div>
              <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>类型</div>
              <div className="flex" style={{ gap: 3 }}>
                {(['prefix', 'suffix'] as const).map(k => (
                  <button key={k}
                    onClick={() => setKind(k)}
                    style={{
                      padding: '3px 10px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                      background: kind === k ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : 'transparent',
                      border: kind === k ? '1px solid #c7b377' : `1px solid ${LINE}`,
                      color: kind === k ? GOLD : MUTED,
                    }}>
                    {k === 'prefix' ? '前缀' : '后缀'}
                  </button>
                ))}
              </div>
            </div>
            {/* 搜索 */}
            <div style={{ flex: '1 1 160px', minWidth: 120 }}>
              <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>
                <i className="fa-solid fa-search" style={{ marginRight: 4 }} />关键字
              </div>
              <input type="text" placeholder="输入名称…"
                value={keyword} onChange={e => setKeyword(e.target.value)}
                style={{
                  width: '100%', padding: '5px 8px', borderRadius: 4, border: `1px solid ${LINE}`,
                  background: '#0a0806', color: TEXT, fontSize: 13, outline: 'none', boxSizing: 'border-box',
                }} />
            </div>
            {/* 物品类型 */}
            <div style={{ flex: '0 0 130px' }}>
              <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>
                <i className="fa-solid fa-shield" style={{ marginRight: 4 }} />物品类型
              </div>
              <select value={itemType} onChange={e => setItemType(e.target.value)}
                style={{
                  width: '100%', padding: '4px 6px', borderRadius: 4, border: `1px solid ${LINE}`,
                  background: '#0a0806', color: TEXT, fontSize: 12, outline: 'none',
                }}>
                {ITYPE_OPTIONS.map(o => (
                  <option key={o.value} value={o.value}>{o.label}</option>
                ))}
              </select>
            </div>
            {/* 等级 */}
            <div style={{ flex: '0 0 140px' }}>
              <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>
                最高等级 <span style={{ color: GOLD, fontFamily: 'monospace' }}>≤{levelMax}</span>
              </div>
              <input type="range" min={1} max={99} value={levelMax}
                onChange={e => setLevelMax(parseInt(e.target.value, 10))}
                style={{ width: '100%', accentColor: GOLD }} />
            </div>
          </div>
        </D2EmuCard>
      </div>

      {/* 卡片网格 */}
      <div style={{ flex: 1, overflowY: 'auto', minHeight: 0 }}>
        <D2EmuCard>
          {filtered.length === 0 ? (
            <div style={{ color: MUTED, fontSize: 14, padding: 24, textAlign: 'center' }}>
              没有匹配的词缀。试试放宽类型或等级。
            </div>
          ) : (
            <div className="grid gap-2" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))' }}>
              {filtered.map((it, i) => (
                <AffixCard key={`${it.name}-${i}`} affix={it} />
              ))}
            </div>
          )}
        </D2EmuCard>
      </div>
    </div>
  )
}
