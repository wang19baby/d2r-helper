/**
 * warehouseStore — 扩展仓库 SQLite collection 的 L1 缓存
 *
 * 数据命令: warehouse_search / warehouse_deposit / warehouse_withdraw /
 *           warehouse_remove / warehouse_update_meta
 */

import { tauriInvoke } from '../tauri.ts'
import { getCache, bridgeCacheToWindow, emitInvalidate, type CacheSource } from './index.ts'
import type { WarehouseItem } from '../types.ts'

export const CACHE_NAME = 'warehouse'

export interface SearchFilters {
  source_character?: string
  item_kind?: string
  equipment_slot?: string
  quality?: string
  search_text?: string
}

export const searchKey = (f: SearchFilters): string => {
  const sorted = Object.keys(f).sort().join(',')
  const sig = sorted
    .split(',')
    .filter(Boolean)
    .map(k => `${k}=${f[k as keyof SearchFilters] ?? ''}`)
    .join('&')
  return `${CACHE_NAME}:search:${sorted}:${sig}`
}

export interface WarehouseStore {
  readonly cacheName: typeof CACHE_NAME
  searchKey: typeof searchKey
  search(filters: SearchFilters, opts?: { force?: boolean }): Promise<WarehouseItem[]>
  setSearch(filters: SearchFilters, data: WarehouseItem[]): void
  invalidateAll(): void
}

let bridged = false
function ensureBridge(): void {
  if (bridged) return
  bridged = true
  bridgeCacheToWindow(getCache<unknown>(CACHE_NAME), CACHE_NAME)
}

export const warehouseStore: WarehouseStore = {
  cacheName: CACHE_NAME,
  searchKey,

  async search(
    filters: SearchFilters,
    opts: { force?: boolean } = {},
  ): Promise<WarehouseItem[]> {
    ensureBridge()
    const cache = getCache<WarehouseItem[]>(CACHE_NAME)
    const key = searchKey(filters)
    const cached = cache.get(key, { force: opts.force })
    if (cached) return cached
    const data = (await tauriInvoke('warehouse_search', filters as Record<string, unknown>)) as WarehouseItem[]
    cache.set(key, data, 'ipc')
    return data
  },

  setSearch(filters: SearchFilters, data: WarehouseItem[]): void {
    ensureBridge()
    getCache<WarehouseItem[]>(CACHE_NAME).set(searchKey(filters), data, 'ipc')
  },

  invalidateAll(): void {
    ensureBridge()
    emitInvalidate({
      cacheName: CACHE_NAME,
      pattern: `${CACHE_NAME}:`,
      reason: 'warehouse-write',
    })
  },
}

// ── Default page (per-code + per-item override) ─────────────
//
// These are write commands; they bypass the cache (no read-then-cache) and
// invalidate the warehouse prefix on success so subsequent `search()` calls
// reflect the new state.

export interface DefaultPageInfo {
  item_code: string
  code_default: string | null
  item_default: string | null
  effective: string | null
}

export async function setCodeDefault(itemCode: string, pageName: string): Promise<void> {
  await tauriInvoke('warehouse_set_code_default', { itemCode, pageName })
  warehouseStore.invalidateAll()
}

export async function clearCodeDefault(itemCode: string): Promise<boolean> {
  const result = (await tauriInvoke('warehouse_clear_code_default', { itemCode })) as boolean
  warehouseStore.invalidateAll()
  return result
}

export async function setItemDefault(itemId: string, pageName: string): Promise<boolean> {
  const result = (await tauriInvoke('warehouse_set_item_default', { itemId, pageName })) as boolean
  warehouseStore.invalidateAll()
  return result
}

export async function clearItemDefault(itemId: string): Promise<boolean> {
  const result = (await tauriInvoke('warehouse_clear_item_default', { itemId })) as boolean
  warehouseStore.invalidateAll()
  return result
}

export async function resolveDefault(
  itemCode: string,
  itemId?: string,
): Promise<DefaultPageInfo> {
  return (await tauriInvoke('warehouse_resolve_default', { itemCode, itemId })) as DefaultPageInfo
}
