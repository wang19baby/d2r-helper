/**
 * stashStore — d2i 共享仓库 (page + items) 的 L1 缓存
 *
 * 与 characterStore.internal-stash 共享同一份 d2i,数据来源 `read_stash`。
 * 跨页 stale 由本地 +60s TTL 治理 (useCached.maxAgeMs)。
 */

import { tauriInvoke } from '../tauri.ts'
import { getCache, bridgeCacheToWindow, emitInvalidate, type CacheSource } from './index.ts'
import type { StashResult } from '../types.ts'

export const CACHE_NAME = 'stash'

export const fullKey = (name: string): string => `${CACHE_NAME}:${name}`

/** L1 cache key for the active stash file path. Separate from `fullKey('shared')`
 * because the path is much cheaper to fetch and changes only when the
 * configured save_folder changes. */
export const filePathKey = (): string => `${CACHE_NAME}:file:path`

let bridged = false
function ensureBridge(): void {
  if (bridged) return
  bridged = true
  bridgeCacheToWindow(getCache<unknown>(CACHE_NAME), CACHE_NAME)
}

function readL2(name: string): StashResult | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = window.localStorage.getItem(`d2r-char-stash-${name}`)
    return raw ? (JSON.parse(raw) as StashResult) : null
  } catch {
    return null
  }
}

function writeL2(name: string, data: StashResult): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(`d2r-char-stash-${name}`, JSON.stringify(data))
  } catch {
    // 配额 / 隐私模式都可能 silent failure,此处仅 best-effort
  }
}

export interface StashStore {
  readonly cacheName: typeof CACHE_NAME
  fullKey: typeof fullKey
  get(name: string, opts?: { force?: boolean }): StashResult | null
  fetch(name: string, opts?: { force?: boolean }): Promise<StashResult>
  set(name: string, data: StashResult, source?: CacheSource): void
  afterWriteSuccess(name: string, data: StashResult): void
  /**
   * Lightweight: fetch only the active stash file path, without parsing items.
   * Cached in L1 under `stash:file:path`; never written to L2.
   */
  getStashFile(opts?: { force?: boolean }): Promise<string | null>
  /**
   * 跨页 stale 信号 — 任意页发现 d2i 变化时调,
   * invalidate character:* 的 stash 缓存 + 广播到全局。
   */
  invalidateAll(): void
}

export const stashStore: StashStore = {
  cacheName: CACHE_NAME,
  fullKey,

  get(name: string, opts: { force?: boolean } = {}): StashResult | null {
    ensureBridge()
    const cache = getCache<StashResult>(CACHE_NAME)
    const cached = cache.get(fullKey(name), { force: opts.force })
    if (cached) return cached
    return readL2(name)
  },

  async fetch(name: string, opts: { force?: boolean } = {}): Promise<StashResult> {
    ensureBridge()
    const cache = getCache<StashResult>(CACHE_NAME)
    const key = fullKey(name)
    const cached = cache.get(key, { force: opts.force })
    if (cached) return cached
    // tauri read_stash 不需要参数(走默认 save folder)
    const data = (await tauriInvoke('read_stash')) as StashResult
    cache.set(key, data, 'ipc')
    writeL2(name, data)
    return data
  },

  set(name: string, data: StashResult, source: CacheSource = 'ipc'): void {
    ensureBridge()
    getCache<StashResult>(CACHE_NAME).set(fullKey(name), data, source)
  },

  afterWriteSuccess(name: string, data: StashResult): void {
    ensureBridge()
    getCache<StashResult>(CACHE_NAME).set(fullKey(name), data, 'ipc')
    writeL2(name, data)
    emitInvalidate({
      cacheName: CACHE_NAME,
      pattern: fullKey(name),
      reason: 'stash-write-success',
    })
  },

  async getStashFile(opts: { force?: boolean } = {}): Promise<string | null> {
    ensureBridge()
    // Use separate lightweight cache key (path is cheap to re-fetch)
    const pathCache = getCache<string>(CACHE_NAME)
    const pathKey = filePathKey()
    // First check the path-only cache
    const cachedPath = pathCache.get(pathKey, { force: opts.force })
    if (cachedPath) return cachedPath
    // Check shared stash cache for a stash_file too
    const stashCache = getCache<StashResult>(CACHE_NAME)
    const sharedKey = fullKey('shared')
    const cached = stashCache.get(sharedKey, { force: opts.force })
    if (cached?.stash_file) {
      pathCache.set(pathKey, cached.stash_file, 'ipc')
      return cached.stash_file
    }
    // Full IPC call as last resort
    const data = (await tauriInvoke('read_stash')) as StashResult
    if (data.stash_file) pathCache.set(pathKey, data.stash_file, 'ipc')
    return data.stash_file ?? null
  },

  invalidateAll(): void {
    ensureBridge()
    emitInvalidate({
      cacheName: CACHE_NAME,
      pattern: `${CACHE_NAME}:`,
      reason: 'cross-page-stale',
    })
  },
}
