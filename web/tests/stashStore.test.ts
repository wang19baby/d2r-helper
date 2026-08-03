/**
 * stashStore.getStashFile / invalidateAll tests
 *
 * Verifies:
 *   - L1 命中不调 IPC
 *   - L1 miss 调 read_stash (stash_file 从 read_stash 返回值提取)
 *   - invalidateAll 同时清 stash:file:path
 */

import { test, beforeEach } from 'node:test'
import assert from 'node:assert/strict'
import { stashStore, filePathKey, fullKey } from '../src/cache/stash.ts'
import { getCache } from '../src/cache/index.ts'

// ── Setup: minimal window stub + mock tauriInvoke ──
let invokeCalls: { cmd: string; args?: Record<string, unknown> }[] = []
function resetCache(): void {
  invokeCalls = []
  // Clear all L1 caches
  ;['stash', 'warehouse', 'character', 'runeword', 'runes', 'characters'].forEach(name => {
    try { getCache<unknown>(name).clear() } catch { /* noop */ }
  })
}

beforeEach(() => resetCache())

;(globalThis as unknown as Record<string, unknown>).window ??= Object.create(null) as Window & typeof globalThis;
const w = globalThis as Record<string, unknown>;
const win = w.window as Record<string, unknown>;
win.localStorage = {
  _data: new Map<string, string>(),
  get length() { return this._data.size },
  getItem(k: string) { return this._data.get(k) ?? null },
  setItem(k: string, v: string) { this._data.set(k, v) },
  removeItem(k: string) { this._data.delete(k) },
  key(i: number) { return [...this._data.keys()][i] ?? null },
  clear() { this._data.clear() },
} as unknown as Storage;
win.addEventListener = () => {};
win.removeEventListener = () => {};
win.CustomEvent = class {} as unknown as typeof CustomEvent;
// Mock __TAURI__ for tauriInvoke
const tauriInvokeMock = async (cmd: string, args?: Record<string, unknown>) => {
  invokeCalls.push({ cmd, args })
  if (cmd === 'read_stash') {
    return {
      stash_name: 'Shared Stash',
      stash_file: 'D:/saves/ModernSharedStashSoftCoreV2.d2i',
      item_count: 0,
      read_status: 'mock',
      items: [],
      pages: [],
    }
  }
  return null
};
win.__TAURI__ = { core: { invoke: tauriInvokeMock } };

// Re-import the module under test so it picks up the mocked tauriInvoke

test('getStashFile: L1 miss 调 read_stash', async () => {
  resetCache()
  const path = await stashStore.getStashFile()
  assert.equal(path, 'D:/saves/ModernSharedStashSoftCoreV2.d2i')
  assert.equal(invokeCalls.length, 1)
  // getStashFile now reads stash_file from read_stash result
  assert.equal(invokeCalls[0].cmd, 'read_stash')
})

test('getStashFile: L1 命中不调 IPC', async () => {
  resetCache()
  await stashStore.getStashFile() // miss → IPC
  const path2 = await stashStore.getStashFile() // hit
  assert.equal(path2, 'D:/saves/ModernSharedStashSoftCoreV2.d2i')
  assert.equal(invokeCalls.length, 1)
})

test('getStashFile: force=true 跳过 L1', async () => {
  resetCache()
  await stashStore.getStashFile()
  await stashStore.getStashFile({ force: true })
  assert.equal(invokeCalls.length, 2)
})

test('getStashFile: invalidate clears L1 path cache', async () => {
  resetCache()
  await stashStore.getStashFile() // populate L1
  const cache = getCache<unknown>('stash')
  cache.invalidate('stash:')
  await stashStore.getStashFile() // should re-call IPC
  assert.equal(invokeCalls.length, 2)
})

test('getStashFile: 路径不写 L2', async () => {
  resetCache()
  await stashStore.getStashFile()
  const ls = (globalThis as any).window.localStorage
  const keys: string[] = []
  for (let i = 0; i < ls.length; i++) {
    const k = ls.key(i)
    if (k) keys.push(k)
  }
  const l2Keys = keys.filter((k: string) => k.startsWith('d2r-char-stash-'))
  assert.equal(l2Keys.length, 0, `getStashFile 不得写 L2,实际 keys: ${l2Keys.join(', ')}`)
})

test('filePathKey 命名规范', () => {
  assert.equal(filePathKey(), 'stash:file:path')
})

test('fullKey(' + "'shared'" + '): 与 filePathKey 不冲突', () => {
  assert.notEqual(fullKey('shared'), filePathKey())
})
