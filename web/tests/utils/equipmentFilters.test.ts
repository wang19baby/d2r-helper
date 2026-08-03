/**
 * applyEquipmentFilters 纯函数测试 (spec §3.7 P0)
 *
 * 覆盖范围:
 *   - EMPTY_EQUIPMENT_FILTERS 全部 null
 *   过滤条件:        quality/equipment_slot/base_type
 *   组合:            AND (3 维叠加)
 *   边界:            undefined 字段(老数据)/空数组/非空过滤器但不匹配
 *   不可变性:        输入 items 不被修改
 */

import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
  applyEquipmentFilters,
  isEquipmentFiltersActive,
  EMPTY_EQUIPMENT_FILTERS,
} from '../../src/utils/equipmentFilters.ts'
import type { StashItem } from '../../src/types.ts'

// ── 测试 fixtures ─────────────────────────────────────────────────────────

function mkItem(overrides: Partial<StashItem> = {}): StashItem {
  const base: StashItem = {
    code: '7ws',
    name: 'test',
    quality: 'magic',
    equipment_slot: 4,
    base_type: 'sword',
    kind: 'weapon',
    position_x: 0, position_y: 0, width: 1, height: 3,
    page: 0, counted: false, amount: 0, item_id: 0,
    level: 0, identified: false, socketed: false,
    name_en: 'test',
    ...overrides,
  }
  return base as StashItem
}

/** 创建一个缺少指定字段的副本(模拟老数据/未透传)。 */
function withoutField<T, K extends keyof T>(obj: T, key: K): Omit<T, K> {
  const { [key]: _omit, ...rest } = obj
  void _omit
  return rest
}

const sword = mkItem({ quality: 'unique', equipment_slot: 4, base_type: 'sword' })
const axe = mkItem({ quality: 'unique', equipment_slot: 4, base_type: 'axe' })
const helm = mkItem({ quality: 'set', equipment_slot: 1, base_type: 'helm' })
const armor = mkItem({ quality: 'set', equipment_slot: 3, base_type: 'armor' })
const superiorSword = mkItem({ quality: 'superior', equipment_slot: 4, base_type: 'sword' })
/** 删掉 base_type + equipment_slot 模拟老数据中字段未透传 */
const noMeta = withoutField(withoutField(mkItem({ quality: 'magic' }), 'base_type'), 'equipment_slot') as unknown as StashItem

const ITEMS = [sword, axe, helm, armor, noMeta, superiorSword]

test('EMPTY_EQUIPMENT_FILTERS 全是 null', () => {
  assert.equal(EMPTY_EQUIPMENT_FILTERS.quality, null)
  assert.equal(EMPTY_EQUIPMENT_FILTERS.equipment_slot, null)
  assert.equal(EMPTY_EQUIPMENT_FILTERS.base_type, null)
})

test('空 filters → 返回原 items 引用(零拷贝优化)', () => {
  const EMPTY = { quality: null, equipment_slot: null, base_type: null }
  const result = applyEquipmentFilters(ITEMS, EMPTY)
  assert.equal(result, ITEMS) // same reference
})

test('isEquipmentFiltersActive: 空 → false', () => {
  assert.equal(isEquipmentFiltersActive(EMPTY_EQUIPMENT_FILTERS), false)
})

test('isEquipmentFiltersActive: 任何非 null → true', () => {
  assert.equal(isEquipmentFiltersActive({ quality: 'unique', equipment_slot: null, base_type: null }), true)
  assert.equal(isEquipmentFiltersActive({ quality: null, equipment_slot: 4, base_type: null }), true)
  assert.equal(isEquipmentFiltersActive({ quality: null, equipment_slot: null, base_type: 'sword' }), true)
})

// ── 单维度过滤 ─────────────────────────────────────────────────────────────

test('quality=unique → 过滤出 sword + axe(noMeta 是 magic)', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: 'unique', equipment_slot: null, base_type: null })
  assert.equal(result.length, 2)
  assert.equal(result[0], sword)
  assert.equal(result[1], axe)
})

test('quality=superior → 仅 superiorSword', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: 'superior', equipment_slot: null, base_type: null })
  assert.equal(result.length, 1)
  assert.equal(result[0], superiorSword)
})

test('equipment_slot=4 → 过滤出 sword + axe + superiorSword', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: null, equipment_slot: 4, base_type: null })
  assert.equal(result.length, 3)
})

test('equipment_slot=1 → 只有 helm', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: null, equipment_slot: 1, base_type: null })
  assert.equal(result.length, 1)
  assert.equal(result[0], helm)
})

test('equipment_slot=3 → 只有 armor', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: null, equipment_slot: 3, base_type: null })
  assert.equal(result.length, 1)
  assert.equal(result[0], armor)
})

test('base_type=sword → 只有 sword + superiorSword', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: null, equipment_slot: null, base_type: 'sword' })
  assert.equal(result.length, 2)
  assert.equal(result[0], sword)
  assert.equal(result[1], superiorSword)
})

test('base_type=axe → 只有 axe', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: null, equipment_slot: null, base_type: 'axe' })
  assert.equal(result.length, 1)
  assert.equal(result[0], axe)
})

// ── AND 组合 ──────────────────────────────────────────────────────────────

test('AND: quality=unique + base_type=sword → 只有 sword', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: 'unique', equipment_slot: null, base_type: 'sword' })
  assert.equal(result.length, 1)
  assert.equal(result[0], sword)
})

test('AND: quality=unique + equipment_slot=4 → sword + axe', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: 'unique', equipment_slot: 4, base_type: null })
  assert.equal(result.length, 2)
  assert.equal(result[0], sword)
  assert.equal(result[1], axe)
})

test('AND 三维: quality=set + equipment_slot=3 + base_type=armor → 只有 armor', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: 'set', equipment_slot: 3, base_type: 'armor' })
  assert.equal(result.length, 1)
  assert.equal(result[0], armor)
})

// ── 边界 ──────────────────────────────────────────────────────────────────

test('不匹配的组合(rare) → 返回空数组', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: 'rare', equipment_slot: null, base_type: null })
  assert.equal(result.length, 0)
})

test('quality=unique + base_type=armor(无匹配) → 空', () => {
  const result = applyEquipmentFilters(ITEMS, { quality: 'unique', equipment_slot: null, base_type: 'armor' })
  assert.equal(result.length, 0)
})

test('quality 字段缺失项被过滤', () => {
  const noQuality = withoutField(mkItem({}), 'quality') as unknown as StashItem
  const list = [noQuality, sword]
  const result = applyEquipmentFilters(list, { quality: 'unique', equipment_slot: null, base_type: null })
  assert.equal(result.length, 1)
  assert.equal(result[0], sword)
})

test('equipment_slot 字段缺失项被过滤', () => {
  const noSlot = withoutField(mkItem({}), 'equipment_slot') as unknown as StashItem
  const list = [noSlot, sword]
  const result = applyEquipmentFilters(list, { quality: null, equipment_slot: 4, base_type: null })
  assert.equal(result.length, 1)
  assert.equal(result[0], sword)
})

test('base_type 字段缺失项被过滤', () => {
  const noBt = withoutField(mkItem({}), 'base_type') as unknown as StashItem
  const list = [noBt, sword]
  const result = applyEquipmentFilters(list, { quality: null, equipment_slot: null, base_type: 'sword' })
  assert.equal(result.length, 1)
  assert.equal(result[0], sword)
})

test('空数组 → 返回空数组', () => {
  const result = applyEquipmentFilters([], { quality: 'unique', equipment_slot: 4, base_type: 'sword' })
  assert.deepEqual(result, [])
})

test('空数组 + 空过滤器 → 返回空数组', () => {
  const result = applyEquipmentFilters([], EMPTY_EQUIPMENT_FILTERS)
  assert.deepEqual(result, [])
})

test('filters 是 EMPTY 的拷贝但全 null → 同样返回原引用', () => {
  const copy = { ...EMPTY_EQUIPMENT_FILTERS }
  const result = applyEquipmentFilters(ITEMS, copy)
  assert.equal(result, ITEMS)
})

// ── 不可变性 ──────────────────────────────────────────────────────────────

test('applyEquipmentFilters 不修改输入数组', () => {
  const original = [...ITEMS]
  applyEquipmentFilters(ITEMS, { quality: 'unique', equipment_slot: null, base_type: null })
  assert.deepEqual(ITEMS, original)
})
