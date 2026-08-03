/**
 * RunewordCalc 页面的 localStorage 缓存层。
 *
 * 设计:
 * - 跨页面打开复用,避免每次进入 RunewordCalc 都重新 invoke Rust 后端
 * - find_runewords(ALL_RUNES) 结果稳定,适合长期缓存
 * - get_runeword_context(底材信息)跟当前存档挂钩,需要在 Characters.tsx 切角色时 invalidate
 * - v1 后缀:schema 变更时换 key,旧数据自然失效,无需迁移
 */

const CACHE_KEY_RW = 'd2r:runeword:all-rw:v1'
const CACHE_KEY_CTX = 'd2r:runeword:context:v1'

export interface RunewordContextCache {
  owned_runes: string[]
  socketed_base_types: string[]
}

// ── ALL_RUNES 查询结果缓存 ──
export function loadAllRunewordsFromStorage(): any[] | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY_RW)
    return raw ? JSON.parse(raw) : null
  } catch { return null }
}
export function saveAllRunewordsToStorage(data: unknown): void {
  try { localStorage.setItem(CACHE_KEY_RW, JSON.stringify(data)) } catch { /* quota / private mode 静默 */ }
}

// ── 上下文(底材信息)缓存 ──
export function loadRunewordContextFromStorage(): RunewordContextCache | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY_CTX)
    return raw ? JSON.parse(raw) : null
  } catch { return null }
}
export function saveRunewordContextToStorage(ctx: RunewordContextCache): void {
  try { localStorage.setItem(CACHE_KEY_CTX, JSON.stringify(ctx)) } catch {}
}

/**
 * 主动失效 context 缓存。在以下场景调用:
 * - 用户切存档(Characters.tsx 监听 selectedChar 变化)
 * - 用户主动点 "从仓库加载" 按钮(loadContext,确保拿到最新存档)
 * - 任何其他"当前存档可能变了"的语义点
 */
export function clearRunewordContextCache(): void {
  try { localStorage.removeItem(CACHE_KEY_CTX) } catch {}
}