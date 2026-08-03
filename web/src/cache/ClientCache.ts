/**
 * ClientCache<T> — L1 Module Singleton cache
 *
 * 详见 docs/design/main-menu-ui-ux-spec-2026-07-20.md §1.2
 *
 * 4 级缓存层级:
 *   L0  Build-in       代码常量, 编入 bundle
 *   L1  Module         in-memory 单例, 全 app 一份 (this file)
 *   L2  localStorage   跨 tab / 跨刷新
 *   L3  Tauri/Rust     server-of-record (.d2s/.d2i/SQLite)
 */

export type CacheSource = 'local' | 'ipc' | 'hybrid' | 'fallback'

export interface CacheEntry<T> {
  /** 真实数据 */
  data: T
  /** 写入时刻 ms timestamp */
  imported_at: number
  /** 数据来源 */
  source: CacheSource
}

export interface GetOptions {
  /** TTL 过期判定. 超过该 ms 视为过期返回 null */
  maxAgeMs?: number
  /** 强制返回 null,触发外层重新加载 */
  force?: boolean
}

/**
 * Type-safe TTL cache with pub/sub.
 *
 * 跨组件订阅用 subscribe();
 * store 之间共享 instance 用 getCache(name);
 * 跨页面失效经 window CustomEvent (events.ts) 中转。
 */
export class ClientCache<T = unknown> {
  private readonly map = new Map<string, CacheEntry<T>>()
  private readonly listeners = new Set<(pattern: string) => void>()
  private readonly name: string

  constructor(name: string = 'default') {
    this.name = name
  }

  getName(): string {
    return this.name
  }

  /** 命中且未过期返回 data,否则返回 null */
  get(key: string, opts: GetOptions = {}): T | null {
    const e = this.map.get(key)
    if (!e) return null
    if (opts.force) return null
    if (opts.maxAgeMs != null && (Date.now() - e.imported_at) > opts.maxAgeMs) {
      return null
    }
    return e.data
  }

  peek(key: string): CacheEntry<T> | null {
    return this.map.get(key) ?? null
  }

  set(key: string, data: T, source: CacheSource = 'ipc'): void {
    this.map.set(key, { data, imported_at: Date.now(), source })
    this.notify(key)
  }

  /**
   * 失效匹配 key:
   *   invalidate('character:foo')         - 精确
   *   invalidate('character:')             - startsWith 前缀
   *   invalidate(/^character:/)            - 正则
   *   invalidate('*')                      - 全部
   * 返回被移除的 key 数
   */
  invalidate(pattern: '*' | string | RegExp): number {
    const matcher = makeMatcher(pattern)
    let removed = 0
    for (const k of [...this.map.keys()]) {
      if (matcher(k)) {
        this.map.delete(k)
        removed++
      }
    }
    if (removed > 0) {
      this.notify(typeof pattern === 'string' ? pattern : '*')
    }
    return removed
  }

  size(): number {
    return this.map.size
  }

  keys(): string[] {
    return [...this.map.keys()]
  }

  has(key: string): boolean {
    return this.map.has(key)
  }

  clear(): number {
    const n = this.map.size
    this.map.clear()
    if (n > 0) this.notify('*')
    return n
  }

  subscribe(fn: (pattern: string) => void): () => void {
    this.listeners.add(fn)
    return () => {
      this.listeners.delete(fn)
    }
  }

  private notify(pattern: string): void {
    for (const fn of this.listeners) {
      try {
        fn(pattern)
      } catch {
        // listener 抛错不破坏 set/invalidate 主流程
      }
    }
  }
}

/**
 * 把 pattern 编译成一个谓词。规则:
 *   '*'            → 始终 true
 *   RegExp         → regex.test
 *   "<prefix>:"    → startsWith(prefix:) (前缀批量失效)
 *   其他 string     → 精确相等
 */
function makeMatcher(pattern: '*' | string | RegExp): (key: string) => boolean {
  if (pattern === '*') return () => true
  if (pattern instanceof RegExp) return k => pattern.test(k)
  if (pattern.endsWith(':')) return k => k.startsWith(pattern)
  return k => k === pattern
}

// ─────────────────────────────────────────────────────────────────
// 模块级 registry — 每个 cacheName 一份 instance (module singleton)
// ─────────────────────────────────────────────────────────────────

export const cacheRegistry = new Map<string, ClientCache<unknown>>()

/** 取出或创建 cacheName 命名的 ClientCache */
export function getCache<T>(name: string): ClientCache<T> {
  let c = cacheRegistry.get(name)
  if (!c) {
    c = new ClientCache<unknown>(name)
    cacheRegistry.set(name, c)
  }
  return c as ClientCache<T>
}

/** 强制删除 registry 中的 cache */
export function dropCache(name: string): boolean {
  const c = cacheRegistry.get(name)
  if (!c) return false
  c.clear()
  return cacheRegistry.delete(name)
}

/** 清空所有 cache (debug/test 用) */
export function clearAllCaches(): void {
  for (const c of cacheRegistry.values()) c.clear()
  cacheRegistry.clear()
}

/** 当前 registry 大小 (debug/test 用) */
export function registrySize(): number {
  return cacheRegistry.size
}
