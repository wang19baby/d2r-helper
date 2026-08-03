// ═══ Tauri IPC wrapper ═══
// 统一处理 invoke 的各种访问路径

type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>

function getInvoke(): InvokeFn | null {
  const w = window as any
  return w.__TAURI__?.core?.invoke
    || w.__TAURI__?.invoke
    || w.__TAURI_INTERNALS__?.invoke
    || null
}

export async function tauriInvoke(cmd: string, args?: Record<string, unknown>): Promise<any> {
  const invoke = getInvoke()
  if (!invoke) throw new Error('不在 Tauri 环境中运行')
  return invoke(cmd, args ?? {})
}

// ── 金额格式化 ──
export function fmt(n: number | string): string {
  return Number(n || 0).toLocaleString('zh-CN')
}
