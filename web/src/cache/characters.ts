/**
 * characterStore — d2s / 角色列表 / 完整 CharacterInfo 的 L1 缓存
 *
 * 详见 docs/design/main-menu-ui-ux-spec-2026-07-20.md §1.2.2
 *
 * loadFull() 使用 event→Promise adapter 封装 Tauri event 驱动的加载流程.
 * 决策记录: docs/design/adr-characters-useCached-2026-07-27.md
 */
 
import { listen } from '@tauri-apps/api/event'
import { tauriInvoke } from '../tauri.ts'
import { getCache, bridgeCacheToWindow, emitInvalidate, type CacheSource } from './index.ts'
import type { CharacterBriefInfo, CharacterInfo } from '../types.ts'

export const CACHE_NAME = 'character'
export const LIST_CACHE = 'characters'  // listKey 命名空间

/** cache key */
export const listKey = `${LIST_CACHE}:list`
export const fullKey = (name: string): string => `${CACHE_NAME}:${name}`
export const classKey = (name: string): string => `${CACHE_NAME}:class:${name}`

let bridged = false
function ensureBridge(): void {
  if (bridged) return
  bridged = true
  bridgeCacheToWindow(getCache<unknown>(CACHE_NAME), CACHE_NAME)
  bridgeCacheToWindow(getCache<unknown>(LIST_CACHE), LIST_CACHE)
}

export interface GetListOpts {
  force?: boolean
  /** save folder */
  dir: string
}

export const LOAD_TIMEOUT_MS = 20_000

export interface CharacterStore {
  readonly cacheName: typeof CACHE_NAME
  readonly listKey: typeof listKey
  fullKey: typeof fullKey
  classKey: typeof classKey

  /** 角色列表 (轻量) — 仅依赖 list_characters_brief */
  getList(opts: GetListOpts): Promise<CharacterBriefInfo[]>

  /** 缓存完整 CharacterInfo;entry point 由 caller 接收 char:stage3 事件后调 setFull */
  getFull(name: string, opts?: { force?: boolean }): CharacterInfo | null
  setFull(name: string, data: CharacterInfo, source?: CacheSource): void

  /**
   * Event→Promise adapter: 加载完整角色数据.
   * 内部注册 Tauri event 监听 (char:stage1/stage3/error),
   * 返回 Promise<CharacterInfo>.  解决后自动写入 L1+L2 缓存.
   */
  loadFull(name: string, saveFolder: string): Promise<CharacterInfo>

  /** 写入成功回执 — Tauri command ok=true 后调,触发 invalidate */
  afterWriteSuccess(name: string, data: CharacterInfo): void

  /** 切角色时失效源 */
  onSwitch(from: string | null, to: string): void
}

// ── L2 helpers ──

function writeL2(name: string, data: CharacterInfo): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(`d2r-char-full-${name}`, JSON.stringify(data))
  } catch {
    // silent: quota / privacy mode
  }
}

/** L2 类缓存 — 用于 chip filter 快速判断职业/死活 */
export function setClassCache(name: string, p: Record<string, unknown>): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(`d2r-char-class-${name}`, JSON.stringify({
      class_en: typeof p.class_en === 'string' ? p.class_en : '',
      class_cn: typeof p.class_cn === 'string' ? p.class_cn : '',
      level: typeof p.level === 'number' ? p.level : 0,
      is_hardcore: !!p.is_hardcore,
      is_expansion: !!p.is_expansion,
      hash: typeof p.file_hash === 'string' ? p.file_hash : '',
    }))
  } catch { /* noop */ }
}

export function getClassCache(name: string): Record<string, unknown> | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = window.localStorage.getItem(`d2r-char-class-${name}`)
    return raw ? JSON.parse(raw) : null
  } catch { return null }
}


function readL2(name: string): CharacterInfo | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = window.localStorage.getItem(`d2r-char-full-${name}`)
    return raw ? (JSON.parse(raw) as CharacterInfo) : null
  } catch {
    return null
  }
}

export const characterStore: CharacterStore = {
  cacheName: CACHE_NAME,
  listKey,
  fullKey,
  classKey,

  async getList(opts: GetListOpts): Promise<CharacterBriefInfo[]> {
    ensureBridge()
    const cache = getCache<CharacterBriefInfo[]>(LIST_CACHE)
    const key = listKey
    const cached = cache.get(key, { force: opts.force })
    if (cached) return cached
    const dir = opts?.dir || ''
    const data = (await tauriInvoke('list_characters_brief', { dir })) as CharacterBriefInfo[]
    cache.set(key, data, 'ipc')
    return data
  },

  getFull(name: string, opts: { force?: boolean } = {}): CharacterInfo | null {
    ensureBridge()
    const cache = getCache<CharacterInfo>(CACHE_NAME)
    const key = fullKey(name)
    const cached = cache.get(key, { force: opts.force })
    if (cached) return cached
    // L1 miss → 读 L2 (localStorage d2r-char-full-<name>)
    return readL2(name)
  },

  setFull(name: string, data: CharacterInfo, source: CacheSource = 'ipc'): void {
    ensureBridge()
    getCache<CharacterInfo>(CACHE_NAME).set(fullKey(name), data, source)
    emitInvalidate({
      cacheName: CACHE_NAME,
      pattern: fullKey(name),
      reason: 'setFull',
    })
  },

  async loadFull(name: string, saveFolder: string): Promise<CharacterInfo> {
    ensureBridge()
    const d2sPath = `${saveFolder}\\${name}.d2s`
    return new Promise<CharacterInfo>((resolve, reject) => {
      let settled = false
      const unsubs: (() => void)[] = []
      const timeout = setTimeout(() => {
        if (settled) return
        settled = true
        unsubs.forEach(u => u())
        reject(new Error(`加载角色超时 (${name})`))
      }, LOAD_TIMEOUT_MS)
      const cleanup = () => {
        if (settled) return
        settled = true
        clearTimeout(timeout)
        unsubs.forEach(u => u())
      }
      // Register all listeners before invoking backend to avoid race
      Promise.all([
        listen<unknown>('char:stage1', (e) => {
          setClassCache(name, (e.payload ?? {}) as Record<string, unknown>)
        }),
        listen<CharacterInfo>('char:stage3', (e) => {
          const data = e.payload
          cleanup()
          writeL2(name, data)
          const cache = getCache<CharacterInfo>(CACHE_NAME)
          cache.set(fullKey(name), data, 'ipc')
          emitInvalidate({
            cacheName: CACHE_NAME,
            pattern: fullKey(name),
            reason: 'loadFull',
          })
          resolve(data)
        }),
        listen<string>('char:error', (e) => {
          cleanup()
          reject(new Error(String(e.payload)))
        }),
      ]).then((results) => {
        results.forEach(u => unsubs.push(u))
        tauriInvoke('load_character_background', { path: d2sPath }).catch((err) => {
          cleanup()
          reject(err instanceof Error ? err : new Error(String(err)))
        })
      }).catch((err) => {
        cleanup()
        reject(new Error(`注册事件监听失败: ${err}`))
      })
    })
  },

  afterWriteSuccess(name: string, data: CharacterInfo): void {
    ensureBridge()
    getCache<CharacterInfo>(CACHE_NAME).set(fullKey(name), data, 'ipc')
    emitInvalidate({
      cacheName: CACHE_NAME,
      pattern: fullKey(name),
      reason: 'write-success',
    })
  },

  onSwitch(from: string | null, to: string): void {
    ensureBridge()
    if (from && from !== to) {
      emitInvalidate({
        cacheName: CACHE_NAME,
        pattern: fullKey(from),
        reason: `switch:${from}->${to}`,
      })
    }
    // 目标角色下一次 getFull 会读 L2
  },
}
