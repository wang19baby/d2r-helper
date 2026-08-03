import { useEffect, useRef, useCallback } from 'react'

/**
 * useFocusTrap — 焦点陷阱
 *
 * Modal 打开时限制 Tab 循环在容器内，按 Escape 关闭，关闭后 focus 回到触发元素。
 *
 * @param containerRef  Modal 容器 ref，其第一个 focusable 子元素会被自动聚焦
 * @param onClose       Escape 回调
 * @param active        是否启用（通常在 modal 打开时设为 true）
 */
export function useFocusTrap(
  containerRef: React.RefObject<HTMLElement | null>,
  onClose?: () => void,
  active = true,
) {
  const previousFocusRef = useRef<HTMLElement | null>(null)

  const getFocusableElements = useCallback(() => {
    const el = containerRef.current
    if (!el) return []
    const selector = [
      'a[href]', 'button:not([disabled])', 'input:not([disabled])',
      'select:not([disabled])', 'textarea:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ].join(', ')
    return Array.from(el.querySelectorAll<HTMLElement>(selector))
  }, [containerRef])

  const trapFocus = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault()
      onClose?.()
      return
    }
    if (e.key !== 'Tab') return

    const focusable = getFocusableElements()
    if (focusable.length === 0) {
      e.preventDefault()
      return
    }

    const first = focusable[0]
    const last = focusable[focusable.length - 1]

    if (e.shiftKey) {
      if (document.activeElement === first) {
        e.preventDefault()
        last.focus()
      }
    } else {
      if (document.activeElement === last) {
        e.preventDefault()
        first.focus()
      }
    }
  }, [getFocusableElements, onClose])

  const savePreviousFocus = useCallback(() => {
    if (document.activeElement instanceof HTMLElement) {
      previousFocusRef.current = document.activeElement
    }
  }, [])

  useEffect(() => {
    if (!active) return

    savePreviousFocus()

    // Auto-focus first element
    const raf = requestAnimationFrame(() => {
      const focusable = getFocusableElements()
      if (focusable.length > 0) focusable[0].focus()
    })

    document.addEventListener('keydown', trapFocus)

    return () => {
      cancelAnimationFrame(raf)
      document.removeEventListener('keydown', trapFocus)
      // Restore focus to trigger element
      previousFocusRef.current?.focus()
    }
  }, [active, trapFocus, getFocusableElements, savePreviousFocus])
}
