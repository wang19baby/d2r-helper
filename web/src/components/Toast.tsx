import { useState, useEffect, useCallback } from 'react'

type ToastVariant = 'success' | 'error' | 'warning' | 'info'
type ToastPosition = 'top' | 'bottom'

interface Toast {
  id: number
  msg: string
  type: ToastVariant
  leaving: boolean
  position: ToastPosition
}

let toastId = 0
let addToastFn: ((t: { msg: string; type: ToastVariant; position: ToastPosition }) => void) | null = null

export interface ShowToastOptions {
  position?: ToastPosition
}

export function showToast(msg: string, type: ToastVariant = 'success', options: ShowToastOptions = {}) {
  addToastFn?.({ msg, type, position: options.position ?? 'bottom' })
}

const TYPE_ICON: Record<ToastVariant, string> = {
  success: 'fa-circle-check',
  error: 'fa-circle-exclamation',
  warning: 'fa-triangle-exclamation',
  info: 'fa-circle-info',
}

const D2EMU_CLASS: Record<ToastVariant, string> = {
  info: 'd2emu-alert d2emu-alert-info',
  success: 'd2emu-alert d2emu-alert-success',
  warning: 'd2emu-alert d2emu-alert-warning',
  error: 'd2emu-alert d2emu-alert-error',
}

const D2EMU_HOST_CLASS: Record<ToastPosition, string> = {
  top: 'd2emu-alert-host',
  bottom: 'fixed bottom-5 right-5 z-[99999] flex flex-col gap-2.5',
}

const LEGACY_COLORS: Record<ToastVariant, string> = {
  success: 'border-green-700/40 text-green-300',
  error: 'border-red-700/40 text-red-300',
  warning: 'border-yellow-700/40 text-yellow-300',
  info: 'border-d2-gold/30 text-d2-gold',
}

export default function ToastContainer() {
  const [toasts, setToasts] = useState<Toast[]>([])

  const add = useCallback((t: { msg: string; type: ToastVariant; position: ToastPosition }) => {
    const id = ++toastId
    setToasts(prev => [...prev, { ...t, id, leaving: false }])
    setTimeout(() => {
      setToasts(prev => prev.map(x => x.id === id ? { ...x, leaving: true } : x))
      setTimeout(() => setToasts(prev => prev.filter(x => x.id !== id)), 250)
    }, 3000)
  }, [])

  useEffect(() => { addToastFn = add; return () => { addToastFn = null } }, [add])

  // 按位置分组,各自动画方向
  const top = toasts.filter(t => t.position === 'top')
  const bottom = toasts.filter(t => t.position === 'bottom')

  return (
    <>
      {top.length > 0 && (
        <div className={D2EMU_HOST_CLASS.top}>
          {top.map(t => (
            <div key={t.id} className={`${D2EMU_CLASS[t.type]} ${t.leaving ? 'is-leaving' : ''}`}>
              <i className={`fa-solid ${TYPE_ICON[t.type]}`} />
              <span>{t.msg}</span>
            </div>
          ))}
        </div>
      )}
      {bottom.length > 0 && (
        <div className={D2EMU_HOST_CLASS.bottom}>
          {bottom.map(t => (
            <div key={t.id}
              className={`min-w-[260px] max-w-[320px] px-3.5 py-3 rounded-lg border bg-[rgba(20,16,12,0.95)] shadow-lg text-sm transition-all duration-250 ${LEGACY_COLORS[t.type]} ${t.leaving ? 'opacity-0 translate-y-2' : 'opacity-100 translate-y-0'}`}>
              {t.msg}
            </div>
          ))}
        </div>
      )}
    </>
  )
}
