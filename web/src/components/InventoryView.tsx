import { useMemo } from 'react'
import { PositionedItemGrid, type GridItem } from './CharacterGrids'
import type { TooltipData, ItemStats, SkillBonus } from '../types'

interface BackpackBeltItem {
  code: string; x: number; y: number; amount: number
  inv_width: number; inv_height: number
  quality?: number | null
  tooltip_lines: string[]
  name_zh?: string | null; name_en?: string | null
  page?: number | null
  /** 后端字段名为 tooltip（非 tooltipData） */
  tooltip?: TooltipData | null
  stats?: ItemStats | null
  /** 后端字段名为 skill_bonuses（蛇形） */
  skill_bonuses?: SkillBonus[] | null
}

interface InventoryViewProps {
  backpackItems: BackpackBeltItem[]
  beltItems: BackpackBeltItem[]
  backpackCols: number; backpackRows: number
  cubeCols: number; cubeRows: number
}

function toGridItem(item: BackpackBeltItem): GridItem {
  return {
    code: item.code, x: item.x, y: item.y,
    amount: item.amount,
    inv_width: item.inv_width, inv_height: item.inv_height,
    quality: item.quality,
    tooltip_lines: item.tooltip_lines,
    name_zh: item.name_zh, name_en: item.name_en,
    page: item.page,
    tooltipData: item.tooltip,
    stats: item.stats,
    skillBonuses: item.skill_bonuses,
  }
}

export default function InventoryView({
  backpackItems, beltItems,
  backpackCols = 10, backpackRows = 4,
  cubeCols = 10, cubeRows = 10,
}: InventoryViewProps) {
  const bpItems = useMemo(
    () => backpackItems.filter(item => !item.page || item.page === 1).map(toGridItem),
    [backpackItems],
  )
  const cubeItems = useMemo(
    () => backpackItems.filter(item => item.page === 4).map(toGridItem),
    [backpackItems],
  )

  const { belt, beltRows } = useMemo(() => {
    const sorted = [...beltItems].sort((a, b) => a.y - b.y || a.x - b.x)
    if (sorted.length === 0) return { belt: [] as GridItem[], beltRows: 4 }

    // 腰带始终归一化到 4 列网格
    // x = 原始坐标取模 4（保留空隙），y = 每 4 个一行
    const items4Col: GridItem[] = sorted.map((item, i) => ({
      ...toGridItem(item), x: item.x % 4, y: Math.floor(i / 4),
    }))

    const maxY = items4Col.reduce((m, i) => Math.max(m, i.y), 0)
    // 翻转 y（游戏数据 y=0 是最下行）
    return {
      belt: items4Col.map(item => ({ ...item, y: maxY - item.y })),
      beltRows: maxY + 1,
    }
  }, [beltItems])

  return (
    <div className="inventory-view-wrap">
      <div>
        <PositionedItemGrid
          items={bpItems}
          label="背包"
          cols={backpackCols}
          rows={backpackRows}
        />
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
        <PositionedItemGrid
          items={belt}
          label="腰带"
          cols={4}
          rows={beltRows}
          fixed
        />
        <PositionedItemGrid
          items={cubeItems}
          label="赫拉迪克方块"
          cols={cubeCols}
          rows={cubeRows}
        />
      </div>
    </div>
  )
}
