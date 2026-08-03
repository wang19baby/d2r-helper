import type { ReactNode } from 'react'

export interface EmptyStateProps {
  /** FontAwesome icon class (without prefix), e.g. "box-open" */
  icon?: string
  /** 主标题(默认大写) */
  title: ReactNode
  /** 提示文案(可选) */
  hint?: ReactNode
  /** 可选 CTA 按钮(传入则渲染右侧) */
  action?: ReactNode
  /** 高度(px),默认 220 */
  minHeight?: number
  /** 紧凑模式(用于表格/列表末尾) */
  compact?: boolean
}

/**
 * EmptyState — 统一空状态占位
 *
 * 沿用 d2emu 设计语言:
 *  - 金边 dashed 圆角块
 *  - 大字 + 副字 + FA 图标
 *  - 可选 CTA 按钮(右侧 action slot)
 */
export default function EmptyState({
  icon = 'inbox',
  title,
  hint,
  action,
  minHeight = 220,
  compact = false,
}: EmptyStateProps) {
  return (
    <div className="d2emu-empty-state"
      style={{
        minHeight: compact ? 120 : minHeight,
        display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center', gap: 8,
        padding: compact ? '20px 16px' : '40px 28px',
        border: '1px dashed var(--color-d2emu-line, #252525)',
        borderRadius: 8,
        background: 'rgba(0,0,0,0.25)',
        textAlign: 'center',
      }}>
      <i className={`fa-solid fa-${icon}`}
        style={{
          fontSize: compact ? 32 : 48,
          color: 'var(--color-d2emu-gold, #FBB13A)',
          opacity: 0.55,
          marginBottom: compact ? 2 : 8,
          filter: 'drop-shadow(0 0 12px rgba(154,0,0,0.25))',
        }} />
      <strong style={{
        font: '600 14px/1.4 "Cinzel", serif',
        letterSpacing: 1.5,
        textTransform: 'uppercase',
        color: 'var(--color-d2emu-text, #e8e8e8)',
      }}>{title}</strong>
      {hint && (
        <span style={{
          fontSize: 14,
          color: 'var(--color-d2emu-muted, #888)',
          maxWidth: 360,
          lineHeight: 1.5,
        }}>{hint}</span>
      )}
      {action && <div style={{ marginTop: 12 }}>{action}</div>}
    </div>
  )
}