/**
 * RunewordStatBanner — 符文消耗统计 (v2 需求文档 §4.5 防丢三连 P0)
 *
 * 输入:
 *   - ownedRunes: 当前玩家拥有的符文 code 集 (Set<string>)
 *   - results: find_runewords 全量返回 (any[]),每项 { runes: string[], name_en: string, ... }
 *
 * 输出三段:
 *   1. covered       -- 符文齐全的符文之语数
 *   2. oneMissing    -- 缺 1 个符文的符文之语 (按缺的符文聚合)
 *   3. phaseCoverage -- 当前能 cover 的阶段数 (开荒过渡/开荒必备/后期热门/然并卵)
 *
 * 沿用 d2emu-card / d2emu-lede / d2emu-kpi-row 风格。
 */

import { useMemo } from 'react'
import type { JSX } from 'react'
import runewordMeta from '../data/runewordMeta.json'

interface RunewordEntry {
  runes: string[]
  name_en: string
  name_zh?: string | null
  name_zh_tw?: string | null
}

interface MetaShape {
  [k: string]: {
    en: string
    phase?: string | null
    stars?: number | null
  }
}

interface PhaseBucket {
  count: number
  rws: string[]
}

const PHASE_LABEL: Record<string, string> = {
  farm1: '开荒过渡',
  farm2: '开荒必备',
  hot1: '后期热门',
  useless2: '然并卵',
}

function localizeName(rw: RunewordEntry, lang: string): string {
  // Try direct fields first
  if (lang === 'enUS') return rw.name_en
  let name: string | null | undefined
  if (lang === 'zhTW') name = rw.name_zh_tw || rw.name_zh
  else name = rw.name_zh || rw.name_zh_tw  // zhCN default
  if (name) return name
  // Fallback: look up runewordMeta.json
  const meta = findMeta(rw.name_en)
  if (meta) {
    const m = meta as unknown as Record<string, unknown>
    if (lang === 'zhTW') return (m.zh_tw as string) || (m.zh as string) || rw.name_en
    return (m.zh as string) || (m.zh_tw as string) || rw.name_en
  }
  return rw.name_en
}

function findMeta(enName: string): MetaShape[string] | undefined {
  for (const key of Object.keys(runewordMeta as MetaShape)) {
    const m = (runewordMeta as MetaShape)[key]
    if (m.en === enName) return m
  }
  return undefined
}

export interface RunewordStatBannerProps {
  /** 当前玩家拥有的符文集 (Set<string> like "r01"-"r33") */
  ownedRunes: Set<string>
  /** find_runewords 全量结果 */
  results: RunewordEntry[]
  /** 当前语言(用于 name_zh vs name_en vs name_zh_tw 切换) */
  language?: string
  /** 渲染模式: full=完整 | stats=仅统计行 | bottlenecks=仅缺符文卡点 */
  mode?: 'full' | 'stats' | 'bottlenecks'
}

export default function RunewordStatBanner({
  ownedRunes,
  results,
  language = 'zhCN',
  mode = 'full',
}: RunewordStatBannerProps): JSX.Element | null {
  const stats = useMemo(() => {
    let covered = 0
    const oneMissingMap = new Map<string, string[]>()  // missing rune → rw names
    const phaseCount: Record<string, PhaseBucket> = {}

    for (const rw of results) {
      const runes = rw.runes
      const allOn = runes.every(r => ownedRunes.has(r))
      if (allOn) {
        covered++
        const meta = findMeta(rw.name_en)
        if (meta?.phase) {
          const bucket = (phaseCount[meta.phase] ||= { count: 0, rws: [] })
          bucket.count++
          bucket.rws.push(localizeName(rw, language))
        }
        continue
      }
      // one-missing
      const missing = runes.find(r => !ownedRunes.has(r))
      if (missing && runes.length - 1 === runes.filter(r => ownedRunes.has(r)).length) {
        let list = oneMissingMap.get(missing)
        if (!list) { list = []; oneMissingMap.set(missing, list) }
        list.push(localizeName(rw, language))
      }
    }

    const oneMissing = Array.from(oneMissingMap.entries())
      .map(([rune, rws]) => ({ rune, rws }))
      .sort((a, b) => b.rws.length - a.rws.length)
      .slice(0, 5)  // 只展示前 5 个最常见的卡点符文

    return { covered, oneMissing, phaseCount }
  }, [ownedRunes, results])

  // 空结果时仍然渲染占位（保持布局高度一致）
  const isEmpty = results.length === 0

  const phaseSummary = Object.entries(stats.phaseCount)
    .filter(([key]) => PHASE_LABEL[key])
    .map(([key, bucket]) => `${PHASE_LABEL[key]} ${bucket.count}`)
    .join(' / ')

  if (mode === 'stats') {
    return (
      <section className="d2emu-card" aria-label="符文消耗统计">
        <header className="d2emu-card-header" style={{ borderBottom: 'none', paddingBottom: 0 }}>
          <div className="min-w-0">
            <p className="d2emu-kicker">
              <i className="fa-solid fa-chart-bar" style={{ marginRight: 6 }} />
              你的 {ownedRunes.size} 个符文可制作
            </p>
          </div>
        </header>
        <div style={{ display: 'grid', gap: 12, marginTop: 8 }}>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12, alignItems: 'baseline' }}>
            <div>
              <span style={{ color: 'var(--color-d2emu-gold)', fontWeight: 700, fontSize: 22 }}>
                {stats.covered}
              </span>{' '}
              <span className="d2emu-muted" style={{ fontSize: 14, textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                个符文之语
              </span>
            </div>
            {phaseSummary && (
              <div style={{ color: 'var(--color-d2emu-muted)', fontSize: 14 }}>
                <span style={{ textTransform: 'uppercase', letterSpacing: '0.06em', marginRight: 6 }}>覆盖</span>
                {phaseSummary}
              </div>
            )}
          </div>
          {stats.covered === 0 && stats.oneMissing.length === 0 && (
            <p className="d2emu-lede" style={{ margin: 0 }}>
              当前未携带任何符文,不能制作符文之语。可以点
              "从仓库加载"自动导入。
            </p>
          )}
        </div>
      </section>
    )
  }

  if (mode === 'bottlenecks') {
    return (
      <section className="d2emu-card" aria-label="符文卡点统计">
        <header className="d2emu-card-header" style={{ borderBottom: 'none', paddingBottom: 0 }}>
          <div className="min-w-0">
            <p className="d2emu-kicker">
              <i className="fa-solid fa-exclamation-triangle" style={{ marginRight: 6, color: '#b8860b' }} />
              缺 1 个 (前 5 名卡点)
            </p>
          </div>
        </header>
        {stats.oneMissing.length > 0 ? (
        <div style={{ marginTop: 8 }}>
          <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            {stats.oneMissing.map(({ rune, rws }) => (
              <li key={rune} style={{
                display: 'inline-flex', alignItems: 'baseline', gap: 6,
                padding: '4px 10px',
                border: '1px solid #b8860b',
                borderRadius: 999,
                background: 'linear-gradient(180deg, rgba(184,134,11,0.10), rgba(184,134,11,0.04))',
                fontSize: 14,
              }}>
                <code style={{ fontFamily: 'JetBrains Mono, monospace', color: '#e8c86a' }}>{rune}</code>
                <span className="d2emu-muted" style={{ fontSize: 13 }}>
                  ×{rws.length} 件 ({rws.slice(0, 3).join(', ')}{rws.length > 3 ? ` 等${rws.length - 3}` : ''})
                </span>
              </li>
            ))}
          </ul>
        </div>
        ) : (
        <div style={{ marginTop: 8, minHeight: 32 }} />
        )}
      </section>
    )
  }

  // mode === 'full'
  return (
    <section className="d2emu-card" aria-label="符文消耗统计">
      <header className="d2emu-card-header" style={{ borderBottom: 'none', paddingBottom: 0 }}>
        <div className="min-w-0">
          <p className="d2emu-kicker">
            <i className="fa-solid fa-chart-bar" style={{ marginRight: 6 }} />
            你的 {ownedRunes.size} 个符文可制作
          </p>
        </div>
      </header>
      <div style={{ display: 'grid', gap: 12, marginTop: 8 }}>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12, alignItems: 'baseline' }}>
          <div>
            <span style={{ color: 'var(--color-d2emu-gold)', fontWeight: 700, fontSize: 22 }}>
              {stats.covered}
            </span>{' '}
            <span className="d2emu-muted" style={{ fontSize: 14, textTransform: 'uppercase', letterSpacing: '0.06em' }}>
              个符文之语
            </span>
          </div>
          {phaseSummary && (
            <div style={{ color: 'var(--color-d2emu-muted)', fontSize: 14 }}>
              <span style={{ textTransform: 'uppercase', letterSpacing: '0.06em', marginRight: 6 }}>覆盖</span>
              {phaseSummary}
            </div>
          )}
        </div>

        {stats.oneMissing.length > 0 && (
          <div>
            <div style={{
              color: 'var(--color-d2emu-muted)',
              fontSize: 14,
              fontWeight: 600,
              textTransform: 'uppercase',
              letterSpacing: '0.06em',
              marginBottom: 6,
            }}>
              <i className="fa-solid fa-exclamation-triangle" style={{ marginRight: 4, color: '#b8860b' }} />
              缺 1 个 (前 5 名卡点)
            </div>
            <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexWrap: 'wrap', gap: 6 }}>
              {stats.oneMissing.map(({ rune, rws }) => (
                <li key={rune} style={{
                  display: 'inline-flex', alignItems: 'baseline', gap: 6,
                  padding: '4px 10px',
                  border: '1px solid #b8860b',
                  borderRadius: 999,
                  background: 'linear-gradient(180deg, rgba(184,134,11,0.10), rgba(184,134,11,0.04))',
                  fontSize: 14,
                }}>
                  <code style={{ fontFamily: 'JetBrains Mono, monospace', color: '#e8c86a' }}>{rune}</code>
                  <span className="d2emu-muted" style={{ fontSize: 13 }}>
                    ×{rws.length} 件 ({rws.slice(0, 3).join(', ')}{rws.length > 3 ? ` 等${rws.length - 3}` : ''})
                  </span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {stats.covered === 0 && stats.oneMissing.length === 0 && (
          <p className="d2emu-lede" style={{ margin: 0 }}>
            当前未携带任何符文,不能制作符文之语。可以点
            "从仓库加载"自动导入。
          </p>
        )}
      </div>
    </section>
  )
}
