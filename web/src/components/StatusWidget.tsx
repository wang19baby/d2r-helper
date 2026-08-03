import type { ReactNode } from 'react'

const GOLD = 'var(--color-d2emu-gold)'
const TEXT = 'var(--color-d2emu-text)'
const MUTED = 'var(--color-d2emu-muted)'
const LINE = 'var(--color-d2emu-line)'

interface StatusWidgetProps {
  icon: string
  label: string
  status: 'ready' | 'warn' | 'error' | 'none'
  statusText: string
  children: ReactNode
  onClick?: () => void
}

const STATUS_COLORS = {
  ready: '#3bbf4f',
  warn: '#c9a34a',
  error: '#c94a4a',
  none: MUTED,
}

export default function StatusWidget({ icon, label, status, statusText, children, onClick }: StatusWidgetProps) {
  const dotColor = STATUS_COLORS[status]
  return (
    <div
      onClick={onClick}
      style={{
        flex: 1, minWidth: 0,
        padding: '12px 14px',
        border: `1px solid ${LINE}`,
        borderRadius: 8,
        background: 'rgba(255,255,255,0.02)',
        cursor: onClick ? 'pointer' : 'default',
        transition: 'border-color 0.2s, background 0.2s',
      }}
      onMouseEnter={e => { if (onClick) { (e.currentTarget as HTMLElement).style.borderColor = GOLD; (e.currentTarget as HTMLElement).style.background = 'rgba(201,163,74,0.05)' } }}
      onMouseLeave={e => { if (onClick) { (e.currentTarget as HTMLElement).style.borderColor = LINE; (e.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,0.02)' } }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
        <i className={`fa-solid ${icon}`} style={{ color: GOLD, fontSize: 16, width: 18, textAlign: 'center' }} />
        <span style={{ fontSize: 14, color: MUTED, fontWeight: 500, letterSpacing: '0.05em' }}>{label}</span>
        <span style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 4 }}>
          <span style={{
            width: 6, height: 6, borderRadius: '50%',
            background: dotColor, display: 'inline-block',
            boxShadow: `0 0 4px ${dotColor}`,
          }} />
          <span style={{ fontSize: 14, color: dotColor }}>{statusText}</span>
        </span>
      </div>
      <div style={{ fontSize: 14, color: TEXT, lineHeight: 1.4 }}>
        {children}
      </div>
    </div>
  )
}
