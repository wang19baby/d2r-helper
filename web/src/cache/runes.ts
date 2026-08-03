/**
 * runesStore — 玩家持有的符文集合 (跨 menu 共享)
 *
 * 派生自:
 *   - characterStore.getFull(name).equipment[*].socketed_items[*].code (r01..r33)
 *   - characterStore.getFull(name).backpack_items[*]
 *   - stashStore.get(name).items[*]
 *   - warehouseStore.search().results[*] (compact socketed view)
 *
 * 提供统一 selectOwnedRunes() 让 RunewordCalc 自动勾选。
 */

import { getCache, bridgeCacheToWindow } from './index.ts'

export const CACHE_NAME = 'runes'

export interface RuneLocations {
  /** r01..r33 编码 */
  byCode: Record<string, { count: number; sources: string[] }>
}

/** 玩家拥有的符文 code 集合 (按字母序) */
export const ownedKey = (characterName: string): string =>
  `${CACHE_NAME}:owned:${characterName}`

/** 符文位置追溯 (P1 特性) */
export const locationsKey = (characterName: string): string =>
  `${CACHE_NAME}:locations:${characterName}`

let bridged = false
function ensureBridge(): void {
  if (bridged) return
  bridged = true
  bridgeCacheToWindow(getCache<unknown>(CACHE_NAME), CACHE_NAME)
}

const RUNE_CODE_RE = /^r(0[1-9]|[12][0-9]|3[0-3])$/

/**
 * 从多种数据源抽取符文 code:
 *   - equipment / backpack / belt / personal_stash (character d2s)
 *   - shared stash items
 *
 * 接受已经 fetch 好的 sources,避免 runesStore 越界依赖其它 store (单向) 。
 */
export function extractRuneCodes(sources: Array<{
  items?: Array<{
    code?: string
    socketed_items?: Array<{ code?: string }>
  }>
}>): string[] {
  const codes: string[] = []
  const push = (code?: string) => {
    if (!code) return
    const c = code.toLowerCase()
    if (RUNE_CODE_RE.test(c)) codes.push(c)
  }
  for (const src of sources) {
    for (const item of src.items ?? []) {
      push(item.code)
      for (const s of item.socketed_items ?? []) push(s.code)
    }
  }
  return Array.from(new Set(codes)).sort()
}

export interface RunesStore {
  readonly cacheName: typeof CACHE_NAME
  ownedKey: typeof ownedKey
  locationsKey: typeof locationsKey
  getOwned(characterName: string): string[] | null
  setOwned(characterName: string, codes: string[]): void
  invalidateFor(characterName: string): void
}

export const runesStore: RunesStore = {
  cacheName: CACHE_NAME,
  ownedKey,
  locationsKey,

  getOwned(characterName: string): string[] | null {
    ensureBridge()
    const cache = getCache<string[]>(CACHE_NAME)
    return cache.get(ownedKey(characterName)) ?? null
  },

  setOwned(characterName: string, codes: string[]): void {
    ensureBridge()
    const dedup = Array.from(new Set(codes.map(c => c.toLowerCase())))
      .filter(c => RUNE_CODE_RE.test(c))
      .sort()
    getCache<string[]>(CACHE_NAME).set(ownedKey(characterName), dedup, 'ipc')
  },

  invalidateFor(characterName: string): void {
    ensureBridge()
    const cache = getCache<string[]>(CACHE_NAME)
    cache.invalidate(ownedKey(characterName))
    cache.invalidate(locationsKey(characterName))
  },
}
