/**
 * D2 词缀 / 合成公式 / 手工艺品 索引 — 客户端共享类型与导入。
 *
 * 数据来自 extracted_data/d2core/*.json,
 * 这里再聚合成 typed 数组便于前端过滤展示。
 */

import rawPrefix from './magic_prefix.json'
import rawSuffix from './magic_suffix.json'
import rarePrefix from './rare_prefix.json'
import rareSuffix from './rare_suffix.json'
import rawCube from './cube_recipes.json'
import rawCrafted from './crafted_items.json'

export interface AffixRecord {
  name: string
  level?: number
  itype1?: string
  itype2?: string
  itype3?: string
  spawnable?: number
  /** mod1code..mod9code */
  [k: string]: any
}

export interface CubeRecipe {
  id: string
  description: string
  enabled: number
  op: number
  numinputs: number
  inputs: string[]
  output: string
  qty: number
  lvl: number
  plvl: number
  bldtst: number
}

// ── 手工装备 (Crafted Items) ────────────────────────────────────────
export interface CraftedItem {
  id: string
  name: string
  /** D2 物品代码 + 类型标记(mag/rare/upg/gem) */
  inputs: string[]
  spawnable: number
  chance: number
  costmult?: number | null
  costadd?: number | null
  /** mod code + min/max/param */
  props: { code: string; min?: number | null; max?: number | null; param?: number | null }[]
}

export const magicPrefixes: AffixRecord[] = rawPrefix as unknown as AffixRecord[]
export const magicSuffixes: AffixRecord[] = rawSuffix as unknown as AffixRecord[]
export const rarePrefixes: AffixRecord[] = rarePrefix as unknown as AffixRecord[]
export const rareSuffixes: AffixRecord[] = rareSuffix as unknown as AffixRecord[]
export const cubeRecipes: CubeRecipe[] = rawCube as unknown as CubeRecipe[]
export const craftedItems: CraftedItem[] = rawCrafted as unknown as CraftedItem[]

// 输入 token 类型标记
export const CRAFTED_INPUT_HINT: Record<string, { label: string; color: string }> = {
  mag: { label: '魔法物品', color: '#5a82b4' },
  rare: { label: '稀有物品', color: '#c7b377' },
  upg: { label: '升级材料', color: '#a76fb7' },
  gem: { label: '宝石', color: '#e3a76f' },
}

// D2 item code 翻译
const ITEM_CODE_ZH: Record<string, string> = {
  cap: 'Cap', skp: 'Skull Cap', hlm: 'Helm', fhl: 'Full Helm', ghm: 'Great Helm',
  crn: 'Crown', msk: 'Mask', bhm: 'Bone Helm', qui: 'Quilted Armor', lea: 'Leather Armor',
  hla: 'Hard Leather', stu: 'Studded', rng: 'Ring Mail', brs: 'Breast Plate',
  spl: 'Splint Mail', plt: 'Plate Mail', fld: 'Field Plate', gth: 'Gothic Plate',
  ful: 'Full Plate', aar: 'Ancient Armor', buc: 'Buckler', sml: 'Small Shield',
  lrg: 'Large Shield', spk: 'Spiked', kit: 'Kite Shield', gts: 'Gothic Shield',
  umb: 'Ancient Shield', hax: 'Hand Axe', '2ax': 'Double Axe', bax: 'Battle Axe',
  gax: 'Great Axe', wax: 'War Axe', lax: 'Large Axe', ths: 'Two-Hand Axe',
  swd: 'Short Sword', scm: 'Scimitar', sab: 'Saber', fal: 'Falchion',
  brd: 'Broad Sword', crs: 'Crystal Sword', lsw: 'Long Sword', bsw: 'Bastard',
  clb: 'Club', spc: 'Spiked Club', msc: 'Morning Star', whm: 'War Hammer',
  mau: 'Maul', gma: 'Great Maul', stf: 'Short Staff', cst: 'Gnarled Staff',
  wnd: 'Wand', bwn: 'Bone Wand',
}

export function translateInputCode(code: string): string {
  return ITEM_CODE_ZH[code.toLowerCase()] ?? code
}
