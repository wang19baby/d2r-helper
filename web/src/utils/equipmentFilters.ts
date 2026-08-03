/**
 * EquipmentFilters 纯函数工具
 *
 * 抽 Inventory.tsx 中的过滤逻辑为独立函数,便于测试 + 复用。
 * spec §3.7 P0 装备类目二级过滤 (quality / equipment_slot / base_type),AND 组合。
 *
 * 设计原则:
 *   - 纯函数:输入 items + filters,返回过滤后的 items。无副作用。
 *   - undefined 字段:当作"不匹配"(除非 filter 为 null = 不过滤)。
 *     理由:老数据 / game_data 未加载时,字段缺失,严格匹配会让它们消失。
 *   - 三个维度独立:全部 null 时返回 items 本身(等价 "全部")。
 */

import type { StashItem } from '../types'

export interface EquipmentFilters {
  /** null = 不过滤 */
  quality: string | null
  /** null = 不过滤 */
  equipment_slot: number | null
  /** null = 不过滤 */
  base_type: string | null
}

export const EMPTY_EQUIPMENT_FILTERS: EquipmentFilters = {
  quality: null,
  equipment_slot: null,
  base_type: null,
}

/**
 * 应用装备二级过滤到 items。
 * 任何字段为 null → 该维度不过滤。
 * 字段 undefined(后端未透传 / 老数据)→ 不匹配(被过滤)。
 */
export function applyEquipmentFilters(
  items: StashItem[],
  filters: EquipmentFilters,
): StashItem[] {
  if (
    filters.quality === null &&
    filters.equipment_slot === null &&
    filters.base_type === null
  ) {
    return items
  }
  return items.filter(item => {
    if (filters.quality !== null) {
      if (item.quality !== filters.quality) return false
    }
    if (filters.equipment_slot !== null) {
      if (item.equipment_slot !== filters.equipment_slot) return false
    }
    if (filters.base_type !== null) {
      if (item.base_type !== filters.base_type) return false
    }
    return true
  })
}

/**
 * 检查是否有任何过滤条件被激活。
 */
export function isEquipmentFiltersActive(filters: EquipmentFilters): boolean {
  return (
    filters.quality !== null ||
    filters.equipment_slot !== null ||
    filters.base_type !== null
  )
}
