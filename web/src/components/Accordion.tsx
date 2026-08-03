import type { ReactNode } from 'react'

const GOLD = 'var(--color-d2emu-gold)'
const LINE = 'var(--color-d2emu-line)'
const MUTED = 'var(--color-d2emu-muted)'

interface AccordionProps {
  title: string
  icon?: string
  defaultOpen?: boolean
  children: ReactNode
  badge?: string
  searchMatch?: boolean
}

export default function Accordion({ title, icon, defaultOpen = false, children, badge, searchMatch }: AccordionProps) {
  return (
    <details open={defaultOpen} style={{
      border: `1px solid ${LINE}`,
      borderRadius: 8,
      background: 'rgba(255,255,255,0.02)',
      overflow: 'hidden',
      transition: 'border-color 0.2s',
      ...(searchMatch === false ? { display: 'none' } : {}),
      ...(searchMatch === true ? { borderColor: GOLD } : {}),
    }}>
      <summary style={{
        display: 'flex', alignItems: 'center', gap: 10,
        padding: '12px 16px', cursor: 'pointer',
        userSelect: 'none', listStyle: 'none',
        borderBottom: '1px solid transparent',
      }}>
        <span style={{ fontSize: 14, color: MUTED, transition: 'transform 0.2s' }}>
          <i className="fa-solid fa-chevron-right" />
        </span>
        {icon && <i className={`fa-solid ${icon}`} style={{ fontSize: 16, color: GOLD, width: 20, textAlign: 'center' }} />}
        <span style={{ fontSize: 15, fontWeight: 600, color: 'var(--color-d2emu-text)', flex: 1 }}>{title}</span>
        {badge && (
          <span style={{
            fontSize: 14, padding: '2px 8px', borderRadius: 10,
            background: 'rgba(201,163,74,0.15)', color: GOLD,
          }}>{badge}</span>
        )}
      </summary>
      <div style={{ padding: '8px 16px 16px' }}>
        {children}
      </div>
      <style>{`
        details summary::-webkit-details-marker { display: none; }
        details[open] summary .fa-chevron-right { transform: rotate(90deg); }
        details[open] summary { border-bottom-color: ${LINE}; }
        summary:hover { background: rgba(255,255,255,0.03); }
      `}</style>
    </details>
  )
}
