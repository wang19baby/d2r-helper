/**
 * 解析 D2 物品 tooltip_lines 数组为结构化字段。
 *
 * D2 tooltip 格式约定(参考 d2s 中文 tooltip):
 *   Line 0:  物品中文名 / 暗金套装名(主标题)
 *   Line 1?: 物品英文名(仅部分物品)
 *   Line N:  Code: xxx / Type: xxx / Quality: xxx / Page: N
 *   Line N:  +X 防御 / +50 攻击准确率 等属性行
 * 从 StashManager.tsx 提取到独立模块，Inventory + 未来组件共用。
 */
export interface ParsedTip {
  title: string
  english: string
  stats: string[]
  meta: string[]
}

/** 分类后的 tooltip 数据结构，匹配后端 TooltipData。 */
export interface ClassifiedTooltip {
  base_info: string[]
  stats: string[]
  hidden_info: string[]
  set_info: string[]
}

/**
 * 将 flat tooltip_lines 按规则分类，匹配 Rust classify_tooltip()。
 * 后端将来会直接返回 tooltip 字段，前端优先使用。
 */
export function classifyTooltip(lines: string[] | null | undefined): ClassifiedTooltip {
  const result: ClassifiedTooltip = { base_info: [], stats: [], hidden_info: [], set_info: [] }
  if (!lines?.length) return result
  for (const l of lines) {
    if (/^(耐久度|Durability)[：:]/.test(l)) {
      result.base_info.push(l)
    } else if (/^(代码|Code|类型|Type|品质|Quality|Page|槽位|Slot):/.test(l)) {
      result.base_info.push(l)
    } else if (/^(物品等级|ItemLevel|暗金 ID|Unique ID|套装 ID|Set ID|符文之语 ID|Runeword ID):/.test(l)) {
      result.hidden_info.push(l)
    } else if (/Set ID|套装 ID/.test(l)) {
      result.set_info.push(l)
    } else if (/[+%]/.test(l) || /\d/.test(l)) {
      result.stats.push(l)
    } else {
      result.base_info.push(l)
    }
  }
  return result
}

export function parseTip(lines: string[] | null | undefined): ParsedTip {
  if (!lines?.length) return { title: '', english: '', stats: [], meta: [] }
  const title = lines[0]
  const rest = lines.slice(1)
  const en = rest.length > 0 && /^[A-Z]/.test(rest[0]) ? rest[0] : ''
  const remainder = en ? rest.slice(1) : rest
  const stats: string[] = []
  const meta: string[] = []
  for (const l of remainder) {
    if (/^(耐久度|Durability)[：:]/.test(l)) meta.push(l)
    else if (/^(代码|Code|类型|Type|品质|Quality|Page|物品等级|ItemLevel):/.test(l)) meta.push(l)
    else stats.push(l)
  }
  return { title, english: en, stats, meta }
}