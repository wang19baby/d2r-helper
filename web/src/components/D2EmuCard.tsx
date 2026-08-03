import type { ReactNode } from 'react'

interface D2EmuCardProps {
  title?: ReactNode
  kicker?: ReactNode
  lede?: ReactNode
  tags?: string[]
  actions?: ReactNode
  children?: ReactNode
  className?: string
  /** 用 d2emu-card-quiet(无阴影)还是 d2emu-card(有阴影) */
  quiet?: boolean
  /** fill 模式: display:flex;flex-direction:column → 子元素可用 flex:1 占满剩余空间 */
  fill?: boolean
}

/**
 * 通用 D2Emu 风格卡片。
 * 复用 d2emu hero 段的结构: kicker / title / lede / tags / actions / body。
 * 用于扩展仓 / 仓库 / 角色档案等"档案级"区域。
 */
export default function D2EmuCard({
  title, kicker, lede, tags, actions, children, className = '', quiet = false, fill = false,
}: D2EmuCardProps) {
  const base = quiet ? 'd2emu-card-quiet' : 'd2emu-card'
  const hasHeader = !!(kicker || title || actions)
  return (
    <section className={`${base} ${className}`}
      style={fill ? { display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0 } : undefined}
    >
      {hasHeader && (
        <header className="d2emu-card-header" style={fill ? { flexShrink: 0 } : undefined}>
          <div className="min-w-0">
            {kicker && <p className="d2emu-kicker">{kicker}</p>}
            {title && <h2 className="d2emu-card-title">{title}</h2>}
            {lede && <p className="d2emu-lede" style={{ marginTop: 6 }}>{lede}</p>}
            {tags && tags.length > 0 && (
              <div className="d2emu-tags" style={{ marginTop: 10 }}>
                {tags.map((t, i) => <span key={i} className="d2emu-tag">{t}</span>)}
              </div>
            )}
          </div>
          {actions && <div className="d2emu-card-actions flex items-center gap-2 flex-shrink-0">{actions}</div>}
        </header>
      )}
      {children && (
        <div className={hasHeader ? 'mt-3' : ''}
          style={fill ? { flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' } : undefined}
        >
          {children}
        </div>
      )}
    </section>
  )
}
