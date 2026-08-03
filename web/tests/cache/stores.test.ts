/**
 * stores smoke test — 验证 5 个 store 的 key 模式 + bridge 副作用。
 * 不调 loader(避免 node:test 环境下 tauriInvoke 抛错)。
 */

import test from 'node:test'
import assert from 'node:assert/strict'

import { clearAllCaches, getCache, registrySize } from '../../src/cache/ClientCache.ts'
import * as charactersMod from '../../src/cache/characters.ts'
import * as stashMod from '../../src/cache/stash.ts'
import * as warehouseMod from '../../src/cache/warehouse.ts'
import * as runewordsMod from '../../src/cache/runewords.ts'
import * as runesMod from '../../src/cache/runes.ts'

// ───── characters ─────

test('characterStore: 暴露正确的 key 与 cacheName', () => {
  assert.equal(charactersMod.characterStore.cacheName, 'character')
  assert.equal(charactersMod.characterStore.listKey, 'characters:list')
  assert.equal(charactersMod.characterStore.fullKey('EchoingStrike'), 'character:EchoingStrike')
  assert.equal(charactersMod.characterStore.classKey('happy_librarian'), 'character:class:happy_librarian')
})

test('characterStore: getFull L1 miss 时 node 环境返回 null', () => {
  clearAllCaches()
  // 不走 tauri invoke,只测 L1+L2 hydration 路径
  const r = charactersMod.characterStore.getFull('nonexistent')
  assert.equal(r, null)
})

test('characterStore: setFull 写入后 get 命中', () => {
  clearAllCaches()
  charactersMod.characterStore.setFull('test', {
    name: 'test', class: '', class_en: '', class_cn: '', class_zh_tw: '',
    level: 1, experience: 0, strength: 0, energy: 0, dexterity: 0, vitality: 0,
    current_hp: 1, max_hp: 1, current_mana: 1, max_mana: 1,
    is_hardcore: false, is_expansion: true, last_played: 0,
    gold: 0, gold_bank: 0, stat_points: 0, new_skills: 0,
    source_path: '', equipment: [], binary_structure: {} as any,
  })
  const got = getCache(charactersMod.CACHE_NAME).peek(charactersMod.characterStore.fullKey('test'))
  assert.ok(got)
  assert.equal(got!.source, 'ipc')
})

// ───── stash ─────

test('stashStore: fullKey 正确', () => {
  assert.equal(stashMod.stashStore.fullKey('EchoingStrike'), 'stash:EchoingStrike')
})

test('stashStore: get L1 miss,L2 miss 返回 null', () => {
  clearAllCaches()
  assert.equal(stashMod.stashStore.get('nobody'), null)
})

// ───── warehouse ─────

test('warehouseStore: searchKey 含签名且 sorted', () => {
  const k1 = warehouseMod.searchKey({ source_character: 'A', quality: 'unique' })
  const k2 = warehouseMod.searchKey({ quality: 'unique', source_character: 'A' })
  assert.equal(k1, k2, 'filter 顺序不影响 key')
  assert.match(k1, /^warehouse:search:[^:]+:/)
  assert.ok(k1.includes('source_character=A'))
  assert.ok(k1.includes('quality=unique'))
})

test('warehouseStore: searchKey 多维 filter', () => {
  const k = warehouseMod.searchKey({
    source_character: 'echo',
    item_kind: 'rune',
    equipment_slot: 'amulet',
    quality: 'set',
    search_text: 'xx',
  })
  for (const seg of ['source_character=echo', 'item_kind=rune', 'equipment_slot=amulet', 'quality=set', 'search_text=xx']) {
    assert.ok(k.includes(seg), `missing ${seg} in ${k}`)
  }
})

test('warehouseStore: setSearch 后 cache 命中', () => {
  clearAllCaches()
  warehouseMod.warehouseStore.setSearch({ quality: 'unique' }, [
    { id: 'w1', item_code: 'r01', item_name: 'El Rune', item_kind: 'rune', quality: 'unique', quantity: 5, page_name: '默认', imported_at: '2026', tags: '', notes: '', tooltip_lines: [] } as any,
  ])
  const got = getCache(warehouseMod.CACHE_NAME).peek(
    warehouseMod.searchKey({ quality: 'unique' }),
  )
  assert.ok(got)
  assert.equal(got!.source, 'ipc')
})

// ───── runewords ─────

test('runeWordStore: resultsKey 排序稳定', () => {
  const k1 = runewordsMod.resultsKey(['r10', 'r01', 'r33'])
  const k2 = runewordsMod.resultsKey(['r33', 'r01', 'r10'])
  assert.equal(k1, k2)
  assert.equal(k1, 'runeword:r01,r10,r33')
})

test('runeWordStore: contextKey 固定', () => {
  assert.equal(runewordsMod.runeWordStore.contextKey, 'runeword:context')
})

test('runeWordStore: L2 hydrate context (用 fakeStorage stub?) 跳过 — L2 read 依赖 window.localStorage', () => {
  // node 无 window,getContext 返回 null
  clearAllCaches()
  assert.equal(runewordsMod.runeWordStore.getContext(), null)
})

// ───── runes ─────

test('runesStore: ownedKey / locationsKey 命名一致', () => {
  assert.equal(runesMod.runesStore.ownedKey('EchoingStrike'), 'runes:owned:EchoingStrike')
  assert.equal(runesMod.runesStore.locationsKey('EchoingStrike'), 'runes:locations:EchoingStrike')
})

test('runesStore: extractRuneCodes 仅 r01-r33', () => {
  const codes = runesMod.extractRuneCodes([
    {
      items: [
        { code: 'r01' },
        { code: 'r22' },
        { code: 'r33' },
        { code: 'gcv' },
        { code: 'INVALID' },
        { code: 'r01' },  // dup
        { code: 'r00' },  // out of range
        { code: 'r34' },  // out of range
        {
          code: 'baa',
          socketed_items: [
            { code: 'r15' },
            { code: 'r25' },
          ],
        },
      ],
    },
  ])
  assert.deepEqual(codes, ['r01', 'r15', 'r22', 'r25', 'r33'])
})

test('runesStore: extractRuneCodes 空输入返回空', () => {
  assert.deepEqual(runesMod.extractRuneCodes([]), [])
  assert.deepEqual(runesMod.extractRuneCodes([{ items: [] }]), [])
  assert.deepEqual(runesMod.extractRuneCodes([{}]), [])
})

test('runesStore: setOwned + getOwned roundtrip + dedup + sort', () => {
  clearAllCaches()
  runesMod.runesStore.setOwned('echo', ['R33', 'r01', 'r22', 'r01', 'gcv', 'r15'])
  const got = runesMod.runesStore.getOwned('echo')
  assert.deepEqual(got, ['r01', 'r15', 'r22', 'r33'])  // dedup + sort + filter invalid
})

// ───── bridge 副作用 ─────

test('5 stores 调用 ensureBridge 后不增加 registry size 异常', () => {
  clearAllCaches()
  charactersMod.characterStore.setFull('a', {} as any)
  stashMod.stashStore.set('a', {} as any)
  warehouseMod.warehouseStore.setSearch({}, [])
  runewordsMod.runeWordStore.setContext({ owned_runes: [], socketed_base_types: [] })
  runesMod.runesStore.setOwned('a', [])
  // 5 stores 最终各自占一个 cacheName (含 characters+stash+warehouse+runeword+runes = 5)
  // characters store 还额外 register characters:list cache
  assert.ok(registrySize() >= 5, `registrySize=${registrySize()}`)
})
