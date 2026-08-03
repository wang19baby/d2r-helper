import type { CSSProperties } from 'react'

const GOLD = 'var(--color-d2emu-gold)'
const TEXT = 'var(--color-d2emu-text)'
const MUTED = 'var(--color-d2emu-muted)'
const LINE = 'var(--color-d2emu-line)'

interface Props {
  searchQuery: string
  onSearchChange: (v: string) => void
  onSearchClear: () => void
}

/**
 * ConfigHeader — 设置页面标题 + 搜索栏
 */
export default function ConfigHeader({ searchQuery, onSearchChange, onSearchClear }: Props) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 12, marginBottom: 20,
      flexWrap: 'wrap',
    }}>
      <h2 style={{
        margin: 0, font: '700 22px/1 "Cinzel", serif',
        letterSpacing: 2, color: GOLD,
        display: 'flex', alignItems: 'center', gap: 10,
      }}>
        <i className="fa-solid fa-gear" /> 设置
      </h2>
      <div style={{ flex: 1, minWidth: 200, position: 'relative' }}>
        <i className="fa-solid fa-search" style={{
          color: MUTED, fontSize: 14, pointerEvents: 'none',
        }} />
        <input
          value={searchQuery}
          onChange={e => onSearchChange(e.target.value)}
          placeholder="搜索设置..."
          style={{
            width: '100%', padding: '8px 12px 8px 34px',
            background: 'rgba(255,255,255,0.04)',
            border: `1px solid ${LINE}`, borderRadius: 6,
            color: TEXT, fontSize: 14, outline: 'none',
          } as CSSProperties}
          onKeyDown={e => { if (e.key === 'Escape') onSearchClear() }}
        />
      </div>
    </div>
  )
}
