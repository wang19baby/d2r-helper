import { useRef, type ReactNode } from 'react'
import { useFocusTrap } from '../hooks/useFocusTrap'

interface Props {
  title?: ReactNode
  children: ReactNode
  confirmText?: string
  cancelText?: string
  danger?: boolean
  loading?: boolean
  onConfirm: () => void
  onClose: () => void
}

/**
 * D2ConfirmModal — 通用确认弹窗
 *
 * 沿用 d2emu 设计语言:
 *  - fixed 遮罩 + 居中卡片
 *  - danger 模式: 红色确认按钮用于不可逆操作
 *  - loading 状态禁用按钮
 *  - 焦点陷阱: Tab 循环在 Modal 内
 */
export default function D2ConfirmModal({
  title,
  children,
  confirmText = '确认',
  cancelText = '取消',
  danger = false,
  loading = false,
  onConfirm,
  onClose,
}: Props) {
  const cardRef = useRef<HTMLDivElement>(null)
  useFocusTrap(cardRef, onClose, true)

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={onClose}>
      <div className="absolute inset-0" style={{ background: 'rgba(0,0,0,0.75)' }} />
      <div ref={cardRef} className="relative d2emu-card d2emu-modal"
        onClick={e => e.stopPropagation()}>

        <div style={{
          borderBottom: '1px solid var(--color-d2emu-line)',
          paddingBottom: 12, marginBottom: 14,
        }}>
          {title && (
            <h3 className="font-d2emu-title" style={{
              textAlign: 'left', padding: 0,
              fontSize: 22, letterSpacing: 2, margin: 0,
              color: danger ? 'var(--color-d2emu-bad, #ef5350)' : 'var(--color-d2emu-gold-bright, #fff)',
            }}>
              {danger && <i className="fa-solid fa-triangle-exclamation" style={{ marginRight: 8 }} />}
              {title}
            </h3>
          )}
        </div>

        <div style={{
          padding: '10px 12px', marginBottom: 14,
          background: 'rgba(0,0,0,0.4)',
          border: `1px solid ${danger ? 'rgba(239,83,80,0.25)' : 'var(--color-d2emu-line)'}`,
          borderRadius: 4, fontSize: 14, lineHeight: 1.6,
          color: 'var(--color-d2emu-text, #e8e8e8)',
        }}>
          {children}
        </div>

        <div style={{ display: 'flex', gap: 10 }}>
          <button
            className={danger ? 'd2emu-btn d2emu-btn-danger' : 'd2emu-btn d2emu-btn-action'}
            style={{ flex: 1 }}
            disabled={loading}
            onClick={onConfirm}>
            {loading ? (
              <><i className="fa-solid fa-spinner fa-spin" /> 处理中…</>
            ) : (
              <><i className={`fa-solid ${danger ? 'fa-trash' : 'fa-check'}`} /> {confirmText}</>
            )}
          </button>
          <button className="d2emu-btn d2emu-btn-ghost" disabled={loading} onClick={onClose}>
            {cancelText}
          </button>
        </div>
      </div>
    </div>
  )
}
