/**
 * runeWordStore — 符文之语匹配 + 上下文 (持有符文 + 底材) 的 L1 缓存
 *
 * 数据命令:
 *   find_runewords         — ownedRunes -> RunewordResult[]
 *   get_runeword_context   — { owned_runes, socketed_base_types }
 */

import { tauriInvoke } from '../tauri.ts'
import { getCache, bridgeCacheToWindow, emitInvalidate, type CacheSource } from './index.ts'

export const CACHE_NAME = 'runeword'
export const CONTEXT_KEY = 'runeword:context'

/** key derived from sorted ownedRunes list */
export const resultsKey = (ownedRunes: string[]): string =>
  `${CACHE_NAME}:${[...ownedRunes].sort().join(',')}`

export interface RunewordContext {
  owned_runes: string[]
  socketed_base_types: string[]
}

export interface RuneWordStore {
  readonly cacheName: typeof CACHE_NAME
  readonly contextKey: typeof CONTEXT_KEY
  resultsKey: typeof resultsKey
  getResults(ownedRunes: string[]): unknown[] | null
  setResults(ownedRunes: string[], data: unknown[], source?: CacheSource): void
  fetchResults(ownedRunes: string[]): Promise<unknown[]>
  getContext(): RunewordContext | null
  setContext(ctx: RunewordContext, source?: CacheSource): void
  fetchContext(): Promise<RunewordContext>
  invalidateContext(): void
}

let bridged = false
function ensureBridge(): void {
  if (bridged) return
  bridged = true
  bridgeCacheToWindow(getCache<unknown>(CACHE_NAME), CACHE_NAME)
}

function readContextL2(): RunewordContext | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = window.localStorage.getItem('runeword-context-cache')
    if (!raw) return null
    const parsed = JSON.parse(raw) as Partial<RunewordContext>
    if (!parsed || !Array.isArray(parsed.owned_runes)) return null
    return {
      owned_runes: parsed.owned_runes,
      socketed_base_types: parsed.socketed_base_types ?? [],
    }
  } catch {
    return null
  }
}

function writeContextL2(ctx: RunewordContext): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem('runeword-context-cache', JSON.stringify(ctx))
  } catch {
    // best-effort
  }
}

export const runeWordStore: RuneWordStore = {
  cacheName: CACHE_NAME,
  contextKey: CONTEXT_KEY,
  resultsKey,

  getResults(ownedRunes: string[]): unknown[] | null {
    ensureBridge()
    const cache = getCache<unknown[]>(CACHE_NAME)
    const cached = cache.get(resultsKey(ownedRunes))
    return cached ?? null
  },

  setResults(ownedRunes: string[], data: unknown[], source: CacheSource = 'ipc'): void {
    ensureBridge()
    getCache<unknown[]>(CACHE_NAME).set(resultsKey(ownedRunes), data, source)
  },

  async fetchResults(ownedRunes: string[]): Promise<unknown[]> {
    ensureBridge()
    const cache = getCache<unknown[]>(CACHE_NAME)
    const key = resultsKey(ownedRunes)
    const cached = cache.get(key)
    if (cached) return cached
    const data = (await tauriInvoke('find_runewords', { ownedRunes })) as unknown[]
    cache.set(key, data, 'ipc')
    return data
  },

  getContext(): RunewordContext | null {
    ensureBridge()
    const cache = getCache<RunewordContext>(CACHE_NAME)
    const cached = cache.get(CONTEXT_KEY)
    if (cached) return cached
    return readContextL2()
  },

  setContext(ctx: RunewordContext, source: CacheSource = 'ipc'): void {
    ensureBridge()
    getCache<RunewordContext>(CACHE_NAME).set(CONTEXT_KEY, ctx, source)
    writeContextL2(ctx)
  },

  async fetchContext(): Promise<RunewordContext> {
    ensureBridge()
    const cache = getCache<RunewordContext>(CACHE_NAME)
    const cached = cache.get(CONTEXT_KEY)
    if (cached) return cached
    const ctx = (await tauriInvoke('get_runeword_context')) as RunewordContext
    cache.set(CONTEXT_KEY, ctx, 'ipc')
    writeContextL2(ctx)
    return ctx
  },

  invalidateContext(): void {
    ensureBridge()
    emitInvalidate({
      cacheName: CACHE_NAME,
      pattern: CONTEXT_KEY,
      reason: 'character-switch',
    })
  },
}
