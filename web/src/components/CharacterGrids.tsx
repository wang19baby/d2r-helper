import { resolveItemIcon, handleImgError } from '../utils/itemImages'
import SocketsOverlay from './SocketsOverlay'
import ItemTooltip from './ItemTooltip'
import type { CSSProperties } from 'react'
export interface GridItem {
  code: string; x: number; y: number; amount?: number
  inv_width: number; inv_height: number
  quality?: number | null; tooltip_lines?: string[]
  name_zh?: string | null
  name_en?: string | null
  /** D2S container: undefined=default, page=4=cube, page=5=stash */
  page?: number | null
  /** 镶嵌物品（符文/宝石/珠宝等） */
  socketed_items?: Array<{
    code: string
    amount: number
    quality?: number | null
    name_zh?: string
    name_en?: string
  }>
  /** 结构化 tooltip 数据（后端已分类） */
  tooltipData?: import('../types').TooltipData | null
  /** 原始 stat 分类数据 */
  stats?: import('../types').ItemStats | null
  /** 技能词缀 */
  skillBonuses?: import('../types').SkillBonus[] | null
}

/** 单格尺寸走 CSS var --cell-grid,断点缩放 (index.css);默认 50px fallback */
const CELL = 'var(--cell-grid, 50px)'

const CELL_EMPTY: CSSProperties = {
  width: CELL, height: CELL,
  backgroundColor: 'rgba(20,20,20,0.4)',
  border: '1px solid rgb(34,34,34)',
  boxSizing: 'border-box',
}

const CELL_FILLED: CSSProperties = {
  width: CELL, height: CELL,
  backgroundColor: 'rgba(40,40,40,0.9)',
  // border color set per-item in GridItemContent based on quality
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  boxSizing: 'border-box',
  position: 'absolute',
  cursor: 'grab',
}

const CELL_MASKED: CSSProperties = {
  width: CELL, height: CELL,
  backgroundColor: 'rgba(15,15,15,0.2)',
  border: '1px solid rgb(40,40,40)',
  boxSizing: 'border-box',
}

const GRID_WRAPPER: CSSProperties = {
  backgroundColor: 'rgba(15,15,15,0.85)',
  border: '1px solid rgb(51,51,51)',
  borderRadius: 6,
  padding: 12,
  userSelect: 'none',
  display: 'inline-block',
}

const GRID_LABEL: CSSProperties = {
  fontSize: 14, fontWeight: 600, textTransform: 'uppercase',
  letterSpacing: '0.08em', color: 'var(--color-d2emu-muted, #888)',
}

function GridSlot({ style }: { style?: CSSProperties }) {
  return <div style={style ?? CELL_EMPTY} />
}

/** 品质 → 边框色 */
const QUALITY_BORDER: Record<number, string> = {
  2: '#e8e8e8',   // normal 白
  3: '#e8e8e8',   // superior 同白
  4: '#5d6cff',   // magic 蓝
  5: '#45b84a',   // set 亮绿
  6: '#c4a847',   // rare 亮金
  7: '#a08030',   // unique 暗金
  8: '#c06820',   // crafted 暗橙
}

/** 渲染单个物品（img + tooltip + 数量角标，边框按品质着色） */
function GridItemContent({ cell }: { cell: GridItem }) {
  const borderColor = QUALITY_BORDER[cell.quality ?? 0] ?? '#e8e8e8'
  const borderWidth = cell.quality === 7 || cell.quality === 6 ? 2 : 1
  return (
    <div className="group z-10 hover:z-50 d2emu-cell-grid" style={{
      ...CELL_FILLED,
      border: `${borderWidth}px solid ${borderColor}`,
      left: `calc(${cell.x} * (var(--cell-grid, 50px) + 1px))`,
      top: `calc(${cell.y} * (var(--cell-grid, 50px) + 1px))`,
      width: `calc(${cell.inv_width} * var(--cell-grid, 50px) + ${cell.inv_width - 1}px)`,
      height: `calc(${cell.inv_height} * var(--cell-grid, 50px) + ${cell.inv_height - 1}px)`,
    } as CSSProperties}>
      <div style={{ position: 'relative', width: '100%', height: '100%' }}>
        <img src={resolveItemIcon(cell)} alt={cell.code}
          data-code={cell.code}
          style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'contain', opacity: 1, pointerEvents: 'none' }}
          onError={handleImgError} />
        {cell.tooltipData?.sockets && (
          <SocketsOverlay sockets={cell.tooltipData.sockets} />
        )}
      </div>
      {(cell.amount ?? 0) > 1 && (
        <span style={{
          position: 'absolute', bottom: 0, right: 0,
          background: '#000', color: '#fff', fontSize: 12, fontWeight: 700,
          padding: '1px 4px', lineHeight: 1, borderRadius: 2,
          border: '1px solid rgba(255,255,255,0.3)',
        }}>×{cell.amount ?? 0}</span>
      )}
      <ItemTooltip
        tooltipData={cell.tooltipData}
        tooltipLines={cell.tooltip_lines}
        quality={cell.quality != null ? ['','low','normal','superior','magic','set','rare','unique','crafted'][cell.quality] ?? 'normal' : 'normal'}
        itemCode={cell.code}
        quantity={cell.amount ?? 0}
        nameZh={cell.name_zh}
        english={cell.name_en}
        socketedItems={cell.socketed_items?.map(s => ({
          code: s.code,
          item_name: s.name_zh ?? s.code,
          quality: (s.quality ?? 0).toString(),
          quantity: s.amount,
        }))}
        mode="hover"
        position="top"
      />
    </div>
  )
}

interface PositionedItemGridProps {
  items: GridItem[]
  label: string
  cols?: number
  rows?: number
  /** 空单元格样式（默认 CELL_EMPTY） */
  emptyCellStyle?: CSSProperties
  /** 固定网格尺寸，不根据物品自动扩展 */
  fixed?: boolean
}

/**
 * PositionedItemGrid — 统一的坐标定位物品网格
 *
 * 根据物品的 x/y/inv_width/inv_height 在网格中绝对定位渲染，
 * 可代替原有的 BackpackGrid / BeltRow / CubeGrid。
 */
export function PositionedItemGrid({
  items, label, cols = 10, rows = 4,
  emptyCellStyle, fixed = false,
}: PositionedItemGridProps) {
  const itemMap = new Map<string, GridItem>()
  for (const it of items) {
    const key = `${it.x}-${it.y}`
    if (!itemMap.has(key)) itemMap.set(key, it)
  }

  // 自动计算网格尺寸（除非 fixed）
  const maxX = !fixed ? items.reduce((m, i) => Math.max(m, i.x + i.inv_width - 1), -1) : -1
  const maxY = !fixed ? items.reduce((m, i) => Math.max(m, i.y + i.inv_height - 1), -1) : -1
  const c = !fixed ? Math.max(cols, maxX + 1) : cols
  const r = !fixed ? Math.max(rows, maxY + 1) : rows
  const total = c * r

  // 多格物品的占用格
  const occupied = new Set<string>()
  for (const it of itemMap.values()) {
    for (let dx = 0; dx < it.inv_width; dx++) {
      for (let dy = 0; dy < it.inv_height; dy++) {
        if (dx > 0 || dy > 0) occupied.add(`${it.x + dx}-${it.y + dy}`)
      }
    }
  }

  const gap = 1
  const slotStyle = emptyCellStyle ?? CELL_EMPTY

  return (
    <div>
      <div style={{ ...GRID_LABEL, marginTop: 10, borderTop: '1px solid rgb(51,51,51)', paddingTop: 10 }}>
        {label} {c}×{r} ({items.length})
      </div>
      <div style={GRID_WRAPPER}>
        <div style={{
          display: 'grid',
          gridTemplateColumns: `repeat(${c}, var(--cell-grid, 50px))`,
          gridTemplateRows: `repeat(${r}, var(--cell-grid, 50px))`,
          gap: `${gap}px`,
          position: 'relative',
          userSelect: 'none',
        }}>
          {Array.from({ length: total }, (_, i) => {
            const x = i % c
            const y = Math.floor(i / c)
            return occupied.has(`${x}-${y}`)
              ? <div key={i} style={CELL_MASKED} />
              : <GridSlot key={i} style={slotStyle} />
          })}
          {Array.from(itemMap.values()).map((cell, i) => (
            <GridItemContent key={`item-${i}`} cell={cell} />
          ))}
        </div>
      </div>
    </div>
  )
}

/* ── 薄包装器（向后兼容，实际内容极少） ── */

export function BeltRow({ items, rows }: { items: GridItem[]; rows?: number }) {
  return (
    <PositionedItemGrid
      items={items}
      label="腰带"
      cols={4}
      rows={rows ?? 1}
      fixed
    />
  )
}

export function BackpackGrid({ items, label, cols, rows }: {
  items: GridItem[]
  label: string
  cols?: number
  rows?: number
}) {
  return (
    <PositionedItemGrid
      items={items}
      label={label}
      cols={cols}
      rows={rows}
    />
  )
}

export function CubeGrid({ items, cols, rows }: { items?: GridItem[]; cols?: number; rows?: number }) {
  return (
    <PositionedItemGrid
      items={items ?? []}
      label="赫拉迪克方块"
      cols={cols ?? 3}
      rows={rows ?? 4}
      emptyCellStyle={{
        ...CELL_EMPTY,
        cursor: 'copy',
      } as CSSProperties}
    />
  )
}

// 腰带行数推导
const BELT_ROWS_BY_CODE: Record<string, number> = {
  s9s: 2, z9s: 2,
  s8l: 3, z8l: 3,
  s7b: 4, z7b: 4,
  s6h: 4, z6h: 4,
}
export function beltRowsFromEquip(equipment: Array<{ slot: string; occupied: boolean; code?: string }>): number {
  const belt = equipment?.find(s => s.slot === 'belt')
  if (!belt || !belt.occupied || !belt.code) return 1
  return Math.min(BELT_ROWS_BY_CODE[belt.code] ?? 4, 4)
}
