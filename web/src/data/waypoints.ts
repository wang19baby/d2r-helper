//! D2R 小站（Waypoint）基础数据。
//!
//! 数据来源:
//! - 39 个小站分布在 5 个 Act（A4 只有 3 个）
//! - 每个小站有 3 个难度副本（普通 / 恶梦 / 地狱）= 117 bits
//! - 名称来自 d2s 协议 + d2emu hero editor 内部命名
//!
//! 字段说明:
//! - `id`     : 0-based 全局索引（与 d2s jf 段 bit 顺序一致）
//! - `act`    : 1-5
//! - `name`   : 英文显示名（前端根据 itemLanguage 翻译）
//! - `area`   : 对应游戏内 area code（参考 d2emu act map）
//! - `d2rAct` : "A1"-"A5" 短码（与 d2emu 一致）

export type D2RAct = 1 | 2 | 3 | 4 | 5

export interface WaypointDef {
  id: number
  act: D2RAct
  d2rAct: 'A1' | 'A2' | 'A3' | 'A4' | 'A5'
  name: string
  name_cn?: string
  /** d2s 协议层 0-based, 39 个小站的全局 index */
  index: number
}

export const TOTAL_WAYPOINTS = 39

/**
 * 5 个 Act 的边界 (lo, hi, d2rAct) — 与 CharacterPanel ACT_BOUNDS 同语义。
 * 用来在扁平数组里切片得到各 act 的小站。
 */
export const ACT_WAYPOINT_BOUNDS: [number, number, D2RAct, 'A1' | 'A2' | 'A3' | 'A4' | 'A5'][] = [
  [0, 9, 1, 'A1'],
  [9, 18, 2, 'A2'],
  [18, 27, 3, 'A3'],
  [27, 30, 4, 'A4'],
  [30, 39, 5, 'A5'],
]

/**
 * 39 个小站完整数据。顺序与 d2s jf 段 bit 顺序严格一致 —
 * 不可重排,否则存档编辑会写错位。
 *
 * 数据参照:
 *   https://github.com/d2emu/d2s-editor (Area Level Waypoint mapping)
 *   d2s jf 段 120 bits = 3 difficulty × 40 waypoints (含 A4 0-index 末尾填充)
 */
export const WAYPOINTS: WaypointDef[] = [
  // ── Act 1 (Rogue Encampment) — 9 个 ──
  { id: 0,  act: 1, d2rAct: 'A1', index: 0,  name: 'Rogue Encampment', name_cn: '萝格营地' },
  { id: 1,  act: 1, d2rAct: 'A1', index: 1,  name: 'Cold Plains', name_cn: '冰冷之原' },
  { id: 2,  act: 1, d2rAct: 'A1', index: 2,  name: 'Stony Field', name_cn: '石块旷野' },
  { id: 3,  act: 1, d2rAct: 'A1', index: 3,  name: 'Dark Wood', name_cn: '黑暗森林' },
  { id: 4,  act: 1, d2rAct: 'A1', index: 4,  name: 'Black Marsh', name_cn: '黑色荒地' },
  { id: 5,  act: 1, d2rAct: 'A1', index: 5,  name: 'Outer Cloister', name_cn: '外侧回廊' },
  { id: 6,  act: 1, d2rAct: 'A1', index: 6,  name: 'Jail Level 1', name_cn: '监牢第一层' },
  { id: 7,  act: 1, d2rAct: 'A1', index: 7,  name: 'Inner Cloister', name_cn: '内侧回廊' },
  { id: 8,  act: 1, d2rAct: 'A1', index: 8,  name: 'Catacombs Level 2', name_cn: '地下墓穴第二层' },

  // ── Act 2 (Lut Gholein) — 9 个 ──
  { id: 9,  act: 2, d2rAct: 'A2', index: 9,  name: 'Lut Gholein', name_cn: '鲁·高因' },
  { id: 10, act: 2, d2rAct: 'A2', index: 10, name: 'Sewers Level 2', name_cn: '下水道第二层' },
  { id: 11, act: 2, d2rAct: 'A2', index: 11, name: 'Dry Hills', name_cn: '干燥高地' },
  { id: 12, act: 2, d2rAct: 'A2', index: 12, name: 'Halls of the Dead Level 2', name_cn: '死亡之殿第二层' },
  { id: 13, act: 2, d2rAct: 'A2', index: 13, name: 'Far Oasis', name_cn: '遥远的绿洲' },
  { id: 14, act: 2, d2rAct: 'A2', index: 14, name: 'Lost City', name_cn: '遗失的城市' },
  { id: 15, act: 2, d2rAct: 'A2', index: 15, name: 'Palace Cellar Level 1', name_cn: '皇宫监牢第一层' },
  { id: 16, act: 2, d2rAct: 'A2', index: 16, name: 'Arcane Sanctuary', name_cn: '神秘庇护所' },
  { id: 17, act: 2, d2rAct: 'A2', index: 17, name: 'Canyon of the Magi', name_cn: '术士的峡谷' },

  // ── Act 3 (Kurast Docks) — 9 个 ──
  { id: 18, act: 3, d2rAct: 'A3', index: 18, name: 'Kurast Docks', name_cn: '库拉斯特海港' },
  { id: 19, act: 3, d2rAct: 'A3', index: 19, name: 'Spider Forest', name_cn: '蜘蛛森林' },
  { id: 20, act: 3, d2rAct: 'A3', index: 20, name: 'Great Marsh', name_cn: '庞大湿地' },
  { id: 21, act: 3, d2rAct: 'A3', index: 21, name: 'Flayer Jungle', name_cn: '剥皮丛林' },
  { id: 22, act: 3, d2rAct: 'A3', index: 22, name: 'Lower Kurast', name_cn: '库拉斯特下层' },
  { id: 23, act: 3, d2rAct: 'A3', index: 23, name: 'Kurast Bazaar', name_cn: '库拉斯特商场' },
  { id: 24, act: 3, d2rAct: 'A3', index: 24, name: 'Upper Kurast', name_cn: '库拉斯特上层' },
  { id: 25, act: 3, d2rAct: 'A3', index: 25, name: 'Travincal', name_cn: '崔凡克' },
  { id: 26, act: 3, d2rAct: 'A3', index: 26, name: 'Durance of Hate Level 2', name_cn: '憎恨的囚牢第二层' },

  // ── Act 4 (The Pandemonium Fortress) — 3 个 ──
  { id: 27, act: 4, d2rAct: 'A4', index: 27, name: 'The Pandemonium Fortress', name_cn: '群魔堡垒' },
  { id: 28, act: 4, d2rAct: 'A4', index: 28, name: 'City of the Damned', name_cn: '诅咒之城' },
  { id: 29, act: 4, d2rAct: 'A4', index: 29, name: 'River of Flame', name_cn: '火焰之河' },

  // ── Act 5 (Harrogath) — 9 个 ──
  { id: 30, act: 5, d2rAct: 'A5', index: 30, name: 'Harrogath', name_cn: '哈洛加斯' },
  { id: 31, act: 5, d2rAct: 'A5', index: 31, name: 'Frigid Highlands', name_cn: '冰冻高地' },
  { id: 32, act: 5, d2rAct: 'A5', index: 32, name: 'Arreat Plateau', name_cn: '亚瑞特高原' },
  { id: 33, act: 5, d2rAct: 'A5', index: 33, name: 'Crystalline Passage', name_cn: '水晶通道' },
  { id: 34, act: 5, d2rAct: 'A5', index: 34, name: 'Halls of Pain', name_cn: '痛苦之厅' },
  { id: 35, act: 5, d2rAct: 'A5', index: 35, name: 'Glacial Trail', name_cn: '冰河路径' },
  { id: 36, act: 5, d2rAct: 'A5', index: 36, name: 'Frozen Tundra', name_cn: '冰冻苔原' },
  { id: 37, act: 5, d2rAct: 'A5', index: 37, name: "The Ancients' Way", name_cn: '先祖之路' },
  { id: 38, act: 5, d2rAct: 'A5', index: 38, name: 'Worldstone Keep Level 2', name_cn: '世界之石要塞第二层' },
]

/** 按 act 取小站（保持顺序） */
export function waypointsByAct(act: D2RAct): WaypointDef[] {
  return WAYPOINTS.filter(w => w.act === act)
}

/** 校验: id 序列 0..38 连续, 不重不漏 */
export function validateWaypoints(): { ok: boolean; missing: number[]; duplicate: number[] } {
  const seen = new Set<number>()
  const missing: number[] = []
  for (let i = 0; i < TOTAL_WAYPOINTS; i++) {
    if (!WAYPOINTS.find(w => w.id === i)) missing.push(i)
  }
  for (const w of WAYPOINTS) seen.add(w.id)
  return { ok: missing.length === 0 && seen.size === TOTAL_WAYPOINTS, missing, duplicate: [] }
}
