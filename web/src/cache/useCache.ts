/**
 * useCached — 统一 React 缓存订阅 hook
 *
 * 数据流:
 *   mount → cache.get(key, { maxAgeMs }) 命中即返回
 *   miss / expired → loader() → cache.set()
 *   invalidate 事件 → useSyncExternalStore 触发 re-render
 *
 * 详见 docs/design/main-menu-ui-ux-spec-2026-07-20.md §1.2.3
 */

import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from 'react'
import { ClientCache, type CacheSource, getCache } from './ClientCache.ts'

export interface UseCachedOptions<T> {
  /** 形如 "character:<name>", 前缀决定绑到哪个 ClientCache */
  key: string
  /** 数据加载函数 */
  loader: () => Promise<T>
  /** 默认 60_000 (60s) */
  maxAgeMs?: number
  /** false 时不自动加载,只订阅事件 */
  enabled?: boolean
  /** mount 时强制 fetch,忽略 cache */
  force?: boolean
}

export interface UseCachedResult<T> {
  data: T | null
  loading: boolean
  error: Error | null
  isStale: boolean
  refresh: (force?: boolean) => Promise<void>
  source: CacheSource | null
}

const DEFAULT_TTL_MS = 60_000

export function useCached<T>(opts: UseCachedOptions<T>): UseCachedResult<T> {
  const cacheName = (opts.key.split(':')[0] || 'default') as string
  // L1 instance 共享,singleton 保证各组件订阅同一份
  const cacheRef = useRef<ClientCache<unknown>>(getCache<unknown>(cacheName))

  // 跨组件订阅:cache 任何变化触发本组件 re-render
  const subscribe = useCallback(
    (cb: () => void) => cacheRef.current.subscribe(() => cb()),
    [],
  )
  const getSnapshot = useCallback(() => cacheRef.current.size(), [])
  // eslint-disable-next-line react-hooks/rules-of-hooks
  useSyncExternalStore(subscribe, getSnapshot, getSnapshot)

  const initialEntry = cacheRef.current.peek(opts.key)
  const [data, setData] = useState<T | null>(
    cacheRef.current.get(opts.key, {
      maxAgeMs: opts.maxAgeMs ?? DEFAULT_TTL_MS,
      force: opts.force,
    }) as T | null,
  )
  const [loading, setLoading] = useState<boolean>(
    initialEntry == null && opts.enabled !== false,
  )
  const [error, setError] = useState<Error | null>(null)
  const [isStale, setStale] = useState<boolean>(false)
  const [source, setSource] = useState<CacheSource | null>(
    initialEntry?.source ?? null,
  )

  const load = useCallback(
    async (force: boolean = false) => {
      if (opts.enabled === false) return
      const ttl = opts.maxAgeMs ?? DEFAULT_TTL_MS

      if (!force) {
        const cached = cacheRef.current.get(opts.key, { maxAgeMs: ttl }) as T | null
        if (cached != null) {
          setData(cached)
          const e = cacheRef.current.peek(opts.key)
          setSource(e?.source ?? null)
          setStale(false)
          return
        }
      }

      setLoading(true)
      setError(null)
      try {
        const v = await opts.loader()
        cacheRef.current.set(opts.key, v, 'ipc')
        setData(v)
        setSource('ipc')
        setStale(false)
      } catch (e: unknown) {
        setError(e instanceof Error ? e : new Error(String(e)))
      } finally {
        setLoading(false)
      }
    },
    [opts.key, opts.maxAgeMs, opts.enabled, opts.loader],
  )

  // 自动 mount 时加载
  useEffect(() => {
    void load(opts.force)
    // 故意只依赖 key/enabled;其它变化(loader 闭包)走 refresh() 触发
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [opts.key, opts.enabled])

  // 计算 staleness: age > maxAgeMs 视为 stale
  useEffect(() => {
    if (!data) {
      setStale(false)
      return
    }
    const e = cacheRef.current.peek(opts.key)
    if (!e) {
      setStale(true)
      return
    }
    const ttl = opts.maxAgeMs ?? DEFAULT_TTL_MS
    setStale(Date.now() - e.imported_at > ttl)
  }, [data, opts.key, opts.maxAgeMs])

  const refresh = useCallback(
    async (force: boolean = false) => {
      await load(force)
    },
    [load],
  )

  return { data, loading, error, isStale, refresh, source }
}
