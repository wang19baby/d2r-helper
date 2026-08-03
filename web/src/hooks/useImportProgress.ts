import { useState, useEffect, useRef, useCallback } from 'react'
import { tauriInvoke } from '../tauri'

export interface ImportProgressTable {
  table_name: string
  status: string
  rows: number
  elapsed_ms?: number
}

interface UseImportProgressOptions {
  /** Called when import transitions from running → not running */
  onComplete?: () => void
}

interface UseImportProgressReturn {
  running: boolean
  tables: ImportProgressTable[]
}

/**
 * useImportProgress — 轮询后端导入进度
 *
 * 挂载时执行一次初始检查，如果导入正在运行则 2s 轮询进度，
 * 导入完成后自动停止轮询并触发 onComplete + 'import-complete' event。
 * 卸载时清理定时器。
 */
export function useImportProgress(options?: UseImportProgressOptions): UseImportProgressReturn {
  const [running, setRunning] = useState(false)
  const [tables, setTables] = useState<ImportProgressTable[]>([])
  const timerRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined)
  const { onComplete } = options ?? {}

  const stopPolling = useCallback(() => {
    clearInterval(timerRef.current)
    timerRef.current = undefined
  }, [])

  const pollOnce = useCallback(async () => {
    try {
      const s = await tauriInvoke('get_import_progress') as any
      setRunning(s.running)
      setTables(s.tables || [])
      if (!s.running) {
        stopPolling()
        window.dispatchEvent(new CustomEvent('import-complete'))
        onComplete?.()
      }
    } catch { /* silent — retry on next tick if polling */ }
  }, [stopPolling, onComplete])

  const startPolling = useCallback(() => {
    if (timerRef.current) return
    timerRef.current = setInterval(pollOnce, 2000)
  }, [pollOnce])

  useEffect(() => {
    // Initial check: is import already running?
    ;(async () => {
      try {
        const s = await tauriInvoke('get_import_progress') as any
        setRunning(s.running)
        setTables(s.tables || [])
        if (s.running) startPolling()
      } catch { /* ignore */ }
    })()

    return () => {
      stopPolling()
    }
  }, [startPolling, stopPolling])

  return { running, tables }
}
