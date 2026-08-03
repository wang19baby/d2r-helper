import type { ReactNode } from 'react'

export interface KpiItem {
  /** 主标签 */
  label: string
  /** 主数值 */
  value: ReactNode
  /** 可选:副 delta 行 */
  delta?: ReactNode
  /** 可选:trend (green=up / red=down / undefined=neutral) */
  trend?: 'up' | 'down' | 'neutral'
  /** 可选:gold 强调 (主要 KPI 用烫金大字) */
  gold?: boolean
}

export interface KpiRowProps {
  items: KpiItem[]
  className?: string
  /** 列数(默认 4) */
  columns?: 2 | 3 | 4 | 5 | 6
}

/**
 * KpiRow — 4 列 KPI 摘要条
 *
 * 沿用 d2emu 设计语言:
 *  - 渐变 panel + 1px line + 左 3px 金边
 *  - 大字 Cinzel 数值 (烫金可选)
 *  - delta 行小字 + 绿/红 trend
 */
export default function KpiRow({
  items, className, columns = 4,
}: KpiRowProps) {
  return (
    <div
      className={`d2emu-kpi-row ${className || ''}`}
      role="list"
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
        gap: 12,
        margin: '18px 0 16px',
      }}>
      {items.map((item, i) => {
        const trendColor = item.trend === 'up'
          ? '#6ad26a'
          : item.trend === 'down'
          ? '#ef5350'
          : 'var(--color-d2emu-text-muted, #8c8a85)'
        return (
          <div key={i} role="listitem"
            className={`d2emu-kpi ${item.gold ? 'd2emu-kpi-gold' : ''}`}
            style={{
              padding: '14px 18px',
              background: 'linear-gradient(180deg, #1a1d21, #11151a)',
              border: '1px solid var(--color-d2emu-line, #252525)',
              borderRadius: 6,
              position: 'relative',
              overflow: 'hidden',
              minWidth: 0,
            }}>
            <span style={{
              position: 'absolute', left: 0, top: 0, bottom: 0, width: 3,
              background: 'var(--color-d2emu-gold, #c7b377)',
            }} />
            <div style={{
              color: 'var(--color-d2emu-text-muted, #8c8a85)',
              font: '600 14px/1 Roboto, sans-serif',
              letterSpacing: '0.12em',
              textTransform: 'uppercase',
            }}>
              {item.label}
            </div>
            <div style={{
              color: item.gold
                ? 'var(--color-d2emu-gold-bright, #fbb13a)'
                : 'var(--color-d2emu-text, #e5e2de)',
              font: '700 24px/1 Cinzel, serif',
              letterSpacing: 1,
              margin: '8px 0 4px',
              whiteSpace: 'nowrap',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
            }}>
              {item.value}
            </div>
            {item.delta != null && (
              <div style={{
                color: trendColor,
                font: '600 14px/1 Roboto Mono, monospace',
                whiteSpace: 'nowrap',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
              }}>
                {item.trend === 'up' && <i className="fa-solid fa-arrow-up" style={{ marginRight: 4 }} />}
                {item.trend === 'down' && <i className="fa-solid fa-arrow-down" style={{ marginRight: 4 }} />}
                {item.delta}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}