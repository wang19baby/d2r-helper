import test from 'node:test'
import assert from 'node:assert/strict'

import {
  ClientCache,
  cacheRegistry,
  clearAllCaches,
  dropCache,
  getCache,
  registrySize,
} from '../../src/cache/ClientCache.ts'

// ───── 基础路径 ─────

test('ClientCache: set/get 基础路径返回 data', () => {
  const c = new ClientCache<number>('t1')
  c.set('x', 42)
  assert.equal(c.get('x'), 42)
  assert.equal(c.size(), 1)
  assert.equal(c.has('x'), true)
})

test('ClientCache: miss 返回 null', () => {
  const c = new ClientCache<number>('t2')
  assert.equal(c.get('y'), null)
  assert.equal(c.size(), 0)
})

test('ClientCache: peek 返回完整 entry', () => {
  const c = new ClientCache<{ v: number }>('t3')
  c.set('a', { v: 1 }, 'local')
  const e = c.peek('a')
  assert.ok(e !== null)
  assert.equal(e!.data.v, 1)
  assert.equal(e!.source, 'local')
  assert.ok(typeof e!.imported_at === 'number' && e!.imported_at > 0)
})

test('ClientCache: peek miss 返回 null', () => {
  const c = new ClientCache<number>('t4')
  assert.equal(c.peek('nope'), null)
})

// ───── force 选项 ─────

test('ClientCache: force=true 跳过命中', () => {
  const c = new ClientCache<number>('t5')
  c.set('x', 1)
  assert.equal(c.get('x'), 1)
  assert.equal(c.get('x', { force: true }), null)
})

// ───── invalidate 模式 ─────

test('ClientCache: invalidate(string) 精确匹配', () => {
  const c = new ClientCache<number>('t6')
  c.set('a', 1)
  c.set('b', 2)
  const removed = c.invalidate('a')
  assert.equal(removed, 1)
  assert.equal(c.get('a'), null)
  assert.equal(c.get('b'), 2)
})

test('ClientCache: invalidate("prefix:") 走 startsWith', () => {
  const c = new ClientCache<number>('t7')
  c.set('character:A', 1)
  c.set('character:B', 2)
  c.set('stash:full', 3)
  const removed = c.invalidate('character:')
  assert.equal(removed, 2)
  assert.deepEqual(c.keys().sort(), ['stash:full'])
})

test('ClientCache: invalidate(regex) 正则匹配', () => {
  const c = new ClientCache<number>('t8')
  c.set('runeword:1', 1)
  c.set('runeword:2', 2)
  c.set('context:other', 3)
  const removed = c.invalidate(/^runeword:/)
  assert.equal(removed, 2)
  assert.deepEqual(c.keys(), ['context:other'])
})

test('ClientCache: invalidate("*") 清空', () => {
  const c = new ClientCache<number>('t9')
  c.set('a', 1)
  c.set('b', 2)
  const removed = c.invalidate('*')
  assert.equal(removed, 2)
  assert.equal(c.size(), 0)
})

test('ClientCache: invalidate 无匹配时返回 0 且不通知', () => {
  const c = new ClientCache<number>('t10')
  c.set('a', 1)
  const calls: string[] = []
  c.subscribe(p => calls.push(p))
  const removed = c.invalidate('zzz')
  assert.equal(removed, 0)
  assert.equal(calls.length, 0)
})

// ───── clear ─────

test('ClientCache: clear 返回受影响数 + 通知', () => {
  const c = new ClientCache<number>('t11')
  c.set('a', 1)
  c.set('b', 2)
  const calls: string[] = []
  c.subscribe(p => calls.push(p))
  const n = c.clear()
  assert.equal(n, 2)
  assert.equal(c.size(), 0)
  assert.deepEqual(calls, ['*'])
})

// ───── pub/sub ─────

test('ClientCache: subscribe 收到 set/invalidate/clear', () => {
  const c = new ClientCache<number>('t12')
  const calls: string[] = []
  c.subscribe(p => calls.push(p))

  c.set('x', 1)
  assert.deepEqual(calls, ['x'])

  c.invalidate('x')
  assert.deepEqual(calls, ['x', 'x'])

  c.set('y', 2)
  c.invalidate('*')
  assert.deepEqual(calls, ['x', 'x', 'y', '*'])
})

test('ClientCache: subscribe unsubscribe 生效', () => {
  const c = new ClientCache<number>('t13')
  const calls: string[] = []
  const unsub = c.subscribe(p => calls.push(p))
  c.set('x', 1)
  unsub()
  c.set('y', 2)
  assert.deepEqual(calls, ['x'])
})

test('ClientCache: subscribe 单 listener 抛错不影响他人', () => {
  const c = new ClientCache<number>('t14')
  const ok: string[] = []
  c.subscribe(() => { throw new Error('boom') })
  c.subscribe(p => ok.push(p))
  c.set('x', 1)
  assert.deepEqual(ok, ['x'])
})

// ───── registry ─────

test('registry: getCache 同 name 复用 instance', () => {
  clearAllCaches()
  const a = getCache<number>('reg-a')
  const b = getCache<number>('reg-a')
  assert.equal(a, b)
  assert.equal(registrySize(), 1)
})

test('registry: 不同 name 不同 instance', () => {
  clearAllCaches()
  const a = getCache<number>('reg-x')
  const b = getCache<number>('reg-y')
  assert.notEqual(a, b)
})

test('registry: dropCache 存在时返回 true,二次返回 false', () => {
  clearAllCaches()
  getCache<number>('drop-test')
  assert.equal(dropCache('drop-test'), true)
  assert.equal(dropCache('drop-test'), false)
  assert.equal(cacheRegistry.has('drop-test'), false)
})

test('registry: clearAllCaches 清空并 keep registry 调用安全', () => {
  getCache<number>('keep-1')
  getCache<number>('keep-2')
  clearAllCaches()
  assert.equal(registrySize(), 0)
})
