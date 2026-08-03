/**
 * cache events — 跨 store / 跨组件失效事件总线
 *
 * 用 window.CustomEvent,简单、跨页面、调试方便(可在 DevTools 中转储监听)。
 * store 可通过 bridgeCacheToWindow 自动同步自身 instance 失效。
 */

export const CACHE_EVENT = 'cache:invalidate'

export interface CacheInvalidateDetail {
  /** 影响的 cache 域, '*' 表示全部 */
  cacheName: string | '*'
  /** 失效 pattern */
  pattern: string | RegExp
  /** ms timestamp */
  ts: number
  /** 可选原因(写命令名 / 角色名),便于日志 */
  reason?: string
}

function isBrowser(): boolean {
  return typeof window !== 'undefined'
}

/** 发送失效事件 */
export function emitInvalidate(detail: Omit<CacheInvalidateDetail, 'ts'>): void {
  if (!isBrowser()) return
  window.dispatchEvent(
    new CustomEvent<CacheInvalidateDetail>(CACHE_EVENT, {
      detail: { ...detail, ts: Date.now() },
    }),
  )
}

/** 订阅失效事件,返回 unsubscribe */
export function onInvalidate(fn: (d: CacheInvalidateDetail) => void): () => void {
  if (!isBrowser()) return () => {}
  const handler = (e: Event) => {
    const ce = e as CustomEvent<CacheInvalidateDetail>
    fn(ce.detail)
  }
  window.addEventListener(CACHE_EVENT, handler)
  return () => window.removeEventListener(CACHE_EVENT, handler)
}

export interface BridgeableCache {
  invalidate: (p: string | RegExp) => number | void
}

/**
 * 把 ClientCache 接上全局事件总线。
 * 事件 cacheName 与 bridge 的 cacheName 一致时自动转发 invalidate。
 */
export function bridgeCacheToWindow(
  cache: BridgeableCache,
  cacheName: string,
): () => void {
  return onInvalidate(d => {
    if (d.cacheName === cacheName || d.cacheName === '*') {
      cache.invalidate(d.pattern)
    }
  })
}
