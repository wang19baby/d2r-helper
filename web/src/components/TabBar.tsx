import { useCallback, type ReactNode } from 'react'

export interface TabBarItem {
  /** 唯一 id,通常用 page.index */
  id: string | number
  /** tab 文本(默认大写,可在 children 覆盖) */
  label: ReactNode
  /** 可选:右侧 count 数字 */
  count?: number | string
  /** 可选 tooltip */
  title?: string
  /** 可选:不可用状态 */
  disabled?: boolean
}

export interface TabBarProps {
  /** tab 列表 */
  items: TabBarItem[]
  /** 当前激活 id */
  activeId: string | number
  /** 切换回调 */
  onChange: (id: string | number) => void
  /** 可选:variant 风格 (默认 'sub') */
  variant?: 'main' | 'sub'
  /** 可选:附加 className */
  className?: string
}

function useTabKeyboard(
  items: TabBarItem[], activeId: string | number, onChange: (id: string | number) => void,
) {
  return useCallback((e: React.KeyboardEvent) => {
    const currentIndex = items.findIndex(i => i.id === activeId)
    let nextIndex = -1
    switch (e.key) {
      case 'ArrowLeft':
        e.preventDefault()
        nextIndex = currentIndex > 0 ? currentIndex - 1 : items.length - 1
        break
      case 'ArrowRight':
        e.preventDefault()
        nextIndex = currentIndex < items.length - 1 ? currentIndex + 1 : 0
        break
      case 'Home':
        e.preventDefault()
        nextIndex = 0
        break
      case 'End':
        e.preventDefault()
        nextIndex = items.length - 1
        break
    }
    if (nextIndex >= 0 && !items[nextIndex]?.disabled) {
      onChange(items[nextIndex].id)
    }
  }, [items, activeId, onChange])
}

/**
 * TabBar — 通用 tab 切换组件
 *
 * 两套视觉风格:
 *  - 'main' : 用于主导航(red 800000 实底,7-tab 反白)
 *  - 'sub'  : 用于 stash sub-tabs / 工具栏(灰底 tab 金色高亮)
 *
 * 支持键盘导航: ArrowLeft/ArrowRight/Home/End
 */
export default function TabBar({
  items, activeId, onChange, variant = 'sub', className,
}: TabBarProps) {
  const onKeyDown = useTabKeyboard(items, activeId, onChange)

  if (variant === 'main') {
    return (
      <nav className={`d2emu-tab-bar ${className || ''}`}
        role="tablist" aria-orientation="horizontal"
        onKeyDown={onKeyDown}
        style={{
          display: 'flex', gap: 0,
          background: 'var(--color-d2emu-bg, #0a0a0a)',
          borderBottom: '2px solid var(--color-d2emu-line, #252525)',
        }}>
        {items.map(item => {
          const isOn = activeId === item.id
          return (
            <button key={item.id} role="tab" aria-selected={isOn}
              disabled={item.disabled}
              title={item.title}
              tabIndex={isOn ? 0 : -1}
              onClick={() => !item.disabled && onChange(item.id)}
              style={{
                padding: '10px 18px',
                background: isOn ? 'var(--color-d2emu-red, #800000)' : 'transparent',
                color: isOn ? '#fff' : 'var(--color-d2emu-text-muted, #8c8a85)',
                font: '700 12px/1 Roboto, sans-serif',
                letterSpacing: '0.08em',
                textTransform: 'uppercase',
                whiteSpace: 'nowrap',
                flexShrink: 0,
                cursor: item.disabled ? 'not-allowed' : 'pointer',
                border: 'none',
                borderRight: '1px solid var(--color-d2emu-line, #252525)',
                opacity: item.disabled ? 0.4 : 1,
                transition: '120ms',
              }}
              onMouseEnter={e => {
                if (!isOn && !item.disabled) {
                  e.currentTarget.style.background = 'rgba(128,0,0,0.15)'
                  e.currentTarget.style.color = 'var(--color-d2emu-text, #e5e2de)'
                }
              }}
              onMouseLeave={e => {
                if (!isOn) {
                  e.currentTarget.style.background = 'transparent'
                  e.currentTarget.style.color = 'var(--color-d2emu-text-muted, #8c8a85)'
                }
              }}>
              {item.label}
              {item.count != null && (
                <span style={{ marginLeft: 8, fontFamily: 'Roboto Mono, monospace', fontSize: 11 }}>
                  ({item.count})
                </span>
              )}
            </button>
          )
        })}
      </nav>
    )
  }

  // 'sub' variant
  return (
    <div className={`d2emu-subtab-bar ${className || ''}`}
      role="tablist" aria-orientation="horizontal"
      onKeyDown={onKeyDown}
      style={{
        display: 'flex', gap: 4, flexWrap: 'wrap',
        borderBottom: '1px solid var(--color-d2emu-line, #252525)',
        padding: '0 4px',
      }}>
      {items.map(item => {
        const isOn = activeId === item.id
        return (
          <button key={item.id} role="tab" aria-selected={isOn}
            disabled={item.disabled}
            title={item.title}
            tabIndex={isOn ? 0 : -1}
            onClick={() => !item.disabled && onChange(item.id)}
            className={`d2emu-subtab ${isOn ? 'is-on' : ''}`}
            style={{
              padding: '8px 16px',
              background: isOn ? 'var(--color-d2emu-panel, #1a1d21)' : 'transparent',
              border: '1px solid',
              borderColor: isOn ? 'var(--color-d2emu-line, #252525)' : 'transparent',
              borderBottom: 'none',
              color: isOn ? 'var(--color-d2emu-text, #e5e2de)' : 'var(--color-d2emu-text-muted, #8c8a85)',
              font: '600 11px/1 Roboto, sans-serif',
              letterSpacing: '0.08em',
              textTransform: 'uppercase',
              cursor: item.disabled ? 'not-allowed' : 'pointer',
              borderRadius: '4px 4px 0 0',
              position: 'relative',
              top: 1,
              opacity: item.disabled ? 0.5 : 1,
              whiteSpace: 'nowrap',
              transition: '120ms',
            }}>
            {item.label}
            {item.count != null && (
              <span className="count" style={{
                marginLeft: 6,
                color: isOn ? 'var(--color-d2emu-gold, #c7b377)' : 'var(--color-d2emu-text-muted, #8c8a85)',
                font: '700 10px/1 Roboto Mono, monospace',
              }}>
                {item.count}
              </span>
            )}
          </button>
        )
      })}
    </div>
  )
}
