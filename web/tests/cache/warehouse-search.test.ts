/**
 * warehouseStore.search 集成测试
 *
 * 验证 L1 缓存与 IPC 调用的交互:
 *   - L1 miss → 调 warehouse_search IPC
 *   - L1 hit  → 不调 IPC
 *   - force=true → 跳过 L1
 *   - searchKey 生成
 *   - 缓存失效
 */

import { test } from 'node:test'
import assert from 'node:assert/strict'
import { warehouseStore, searchKey } from '../../src/cache/warehouse.ts'
import { getCache, clearAllCaches } from '../../src/cache/index.ts'
import type { WarehouseItem } from '../../src/types.ts'

// ── Setup: window stub with real event dispatch ──
const eventHandlers = new Map<string, ((e: Event) => void)[]>()
;(globalThis as unknown as Record<string, unknown>).window ??= {
  localStorage: {
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {},
    length: 0,
    key: () => null,
  } satisfies Storage,
  __TAURI__: {
    core: {
      invoke: async (_cmd: string, _args?: Record<string, unknown>) => [] as WarehouseItem[],
    },
  },
  addEventListener: (type: string, fn: (e: Event) => void) => {
    const list = eventHandlers.get(type) ?? []
    list.push(fn)
    eventHandlers.set(type, list)
  },
  removeEventListener: (type: string, fn: (e: Event) => void) => {
    const list = eventHandlers.get(type) ?? []
    eventHandlers.set(type, list.filter(f => f !== fn))
  },
  dispatchEvent: (e: Event) => {
    const list = eventHandlers.get(e.type)
    if (list) list.forEach(fn => fn(e))
    return true
  },
}

// Track invoke calls
let invokeCalls: { cmd: string; args?: Record<string, unknown> }[] = []

function mockInvoke(): void {
  invokeCalls = []
  const w = globalThis as unknown as Record<string, unknown>
  w.window.__TAURI__.core.invoke =
    async (_cmd: string, _args?: Record<string, unknown>) => {
      invokeCalls.push({ cmd: _cmd, args: _args })
      return []
    }
}

function resetCache(): void {
  clearAllCaches()
  // force re-ensureBridge on next store access
  const cache = getCache<unknown>('warehouse')
  cache.clear()
}

// ── 测试 ─────────────────────────────────────────────────────────────────

test('warehouseStore.search: L1 miss 调 warehouse_search IPC', async () => {
  resetCache()
  mockInvoke()
  await warehouseStore.search({ quality: 'unique' })
  assert.equal(invokeCalls.length, 1)
  assert.equal(invokeCalls[0].cmd, 'warehouse_search')
})

test('warehouseStore.search: L1 命中不调 IPC', async () => {
  resetCache()
  mockInvoke()
  // First call populates cache
  await warehouseStore.search({ item_kind: 'weapon' })
  assert.equal(invokeCalls.length, 1)
  // Second call should hit L1
  await warehouseStore.search({ item_kind: 'weapon' })
  assert.equal(invokeCalls.length, 1) // no additional IPC
})

test('warehouseStore.search: force=true 跳过 L1', async () => {
  resetCache()
  mockInvoke()
  // First call populates cache
  await warehouseStore.search({ quality: 'set' })
  assert.equal(invokeCalls.length, 1)
  // force=true → skips cache, calls IPC again
  await warehouseStore.search({ quality: 'set' }, { force: true })
  assert.equal(invokeCalls.length, 2)
})

test('warehouseStore.search: 不同 filter 不同 cache key', async () => {
  resetCache()
  mockInvoke()
  await warehouseStore.search({ quality: 'unique' })
  await warehouseStore.search({ quality: 'set' })
  assert.equal(invokeCalls.length, 2)
})

test('searchKey: 空 filter', () => {
  const key = searchKey({})
  assert.ok(key.startsWith('warehouse:search:'))
  assert.ok(key.endsWith(':'))
})

test('searchKey: 单 filter 排序稳定', () => {
  const a = searchKey({ quality: 'unique', equipment_slot: '4' })
  const b = searchKey({ equipment_slot: '4', quality: 'unique' })
  assert.equal(a, b)
})

test('searchKey: 全 filter + 多值', () => {
  const key = searchKey({
    source_character: 'Sorceress',
    item_kind: 'weapon',
    equipment_slot: '4',
    quality: 'unique',
    search_text: 'sword',
  })
  assert.ok(key.includes('source_character=Sorceress'))
  assert.ok(key.includes('item_kind=weapon'))
  assert.ok(key.includes('equipment_slot=4'))
  assert.ok(key.includes('quality=unique'))
  assert.ok(key.includes('search_text=sword'))
})

test('invalidateAll: 清空搜索缓存', async () => {
  resetCache()
  mockInvoke()
  // Populate cache
  await warehouseStore.search({ quality: 'unique' })
  assert.equal(invokeCalls.length, 1)
  // Direct invalidate — event-bus wiring tested in ClientCache test
  const cache = getCache<unknown>('warehouse')
  cache.invalidate('warehouse:')
  // Should miss cache now
  await warehouseStore.search({ quality: 'unique' })
  assert.equal(invokeCalls.length, 2)
})
test('setSearch: 写入后 search 命中 cache', async () => {
  mockInvoke()
  const item: WarehouseItem = {
    id: 1, code: '7ws', name_en: 'sword', quality: 'unique',
    item_kind: 'weapon', item_category: 'sword', stash_item_id: null,
  }
  warehouseStore.setSearch({ quality: 'unique' }, [item])
  // Should hit cache, not call IPC
  const result = await warehouseStore.search({ quality: 'unique' })
  assert.equal(invokeCalls.length, 0)
  assert.equal(result.length, 1)
  assert.equal(result[0].id, 1)
})
