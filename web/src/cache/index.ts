/** L1 Module Singleton cache layer — public barrel.
 *
 * Stores:
 *   - ClientCache (class) + getCache / dropCache / clearAllCaches
 *   - CACHE_EVENT / emitInvalidate / onInvalidate / bridgeCacheToWindow
 *   - useCached (React hook)
 *   - 5 stores: characters, stash, warehouse, runewords, runes
 *
 * NOTE: CACHE_NAME and fullKey are exported from multiple store modules.
 *       characters.ts is the canonical source for the bare names; other
 *       modules export them with a per-store prefix alias.
 */

export * from './ClientCache.ts'
export * from './events.ts'
export * from './useCache.ts'

// ── characters (canonical CACHE_NAME + fullKey) ──
export * from './characters.ts'

// ── stash (CACHE_NAME → stashCACHE_NAME, fullKey → stashFullKey) ──
export {
  stashStore,
  type StashStore,
  CACHE_NAME as stashCACHE_NAME,
  fullKey as stashFullKey,
} from './stash.ts'

// ── warehouse (CACHE_NAME → warehouseCACHE_NAME) ──
export {
  warehouseStore,
  type WarehouseStore,
  type SearchFilters,
  searchKey,
  CACHE_NAME as warehouseCACHE_NAME,
} from './warehouse.ts'

// ── runewords (CACHE_NAME → runewordCACHE_NAME) ──
export {
  runeWordStore,
  type RuneWordStore,
  type RunewordContext,
  CONTEXT_KEY,
  resultsKey,
  CACHE_NAME as runewordCACHE_NAME,
} from './runewords.ts'

// ── runes (CACHE_NAME → runeCACHE_NAME) ──
export {
  runesStore,
  type RunesStore,
  type RuneLocations,
  ownedKey,
  locationsKey,
  extractRuneCodes,
  CACHE_NAME as runeCACHE_NAME,
} from './runes.ts'
