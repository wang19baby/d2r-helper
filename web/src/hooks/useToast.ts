import { useCallback } from 'react'
import { tauriInvoke } from '../tauri'
import { showToast, type ShowToastOptions } from '../components/Toast'

export interface ToastAction {
  /** 触发命令名,留空则只显示 toast */
  command?: string
  /** 命令参数 */
  args?: Record<string, unknown>
  /** 成功后触发 balance-update,默认 true(任何余额相关操作) */
  refreshBalance?: boolean
}

/**
 * useToast — 统一 toast hook
 *
 * 沿用现有 showToast,增加 2 个常用模式:
 *  - successWith: 成功 toast + 可选 IPC 调用 + 可选 balance 刷新
 *  - wrap: 包装 async 函数,自动错误捕获
 */
export function useToast() {
  const refreshBalance = useCallback(async () => {
    try {
      const b = await tauriInvoke('get_balance') as number
      window.dispatchEvent(new CustomEvent('balance-update', { detail: b }))
    } catch (e: any) {
      showToast(`刷新余额失败: ${e?.message || '未知错误'}`, 'error', { position: 'top' })
    }
  }, [])

  const fire = useCallback((
    message: string,
    kind: 'success' | 'error' | 'warning' | 'info' = 'info',
    opts?: ShowToastOptions,
  ) => showToast(message, kind, opts), [])

  const success = useCallback((m: string, opts?: ShowToastOptions) =>
    fire(m, 'success', opts), [fire])
  const error = useCallback((m: string, opts?: ShowToastOptions) =>
    fire(m, 'error', opts), [fire])
  const info = useCallback((m: string, opts?: ShowToastOptions) =>
    fire(m, 'info', opts), [fire])
  const warning = useCallback((m: string, opts?: ShowToastOptions) =>
    fire(m, 'warning', opts), [fire])

  /**
   * 包装一个 async 操作:自动 try/catch,成功后 toast + 可选 IPC + 可选 refreshBalance
   */
  const wrap = useCallback(async <T,>(
    fn: () => Promise<T>,
    opts: {
      success?: string | ((v: T) => string)
      error?: string | ((e: any) => string)
      then?: ToastAction
    } = {},
  ): Promise<T | undefined> => {
    try {
      const v = await fn()
      if (opts.success) {
        fire(typeof opts.success === 'function' ? opts.success(v) : opts.success, 'success')
      }
      if (opts.then?.command) {
        try {
          await tauriInvoke(opts.then.command, opts.then.args || {})
        } catch (e) {
          fire(`后续操作失败: ${(e as Error).message}`, 'warning')
        }
      }
      if (opts.then?.refreshBalance) {
        await refreshBalance()
      }
      return v
    } catch (e: any) {
      fire(
        opts.error
          ? (typeof opts.error === 'function' ? opts.error(e) : opts.error)
          : (e?.message || '操作失败'),
        'error',
      )
      return undefined
    }
  }, [fire, refreshBalance])

  return { success, error, info, warning, refreshBalance, wrap }
}