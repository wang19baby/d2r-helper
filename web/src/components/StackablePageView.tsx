import type { CSSProperties, DragEvent } from 'react'
import type { StashItem } from '../types'
import ItemTooltip from './ItemTooltip'
import { resolveItemIcon, handleImgError } from '../utils/itemImages'

interface Props {
  items: StashItem[]
  /** 触发拖动 → 写入 dragPayload / dragPayloadRef,后端 deposit 用 */
  beginDragStash?: (item: StashItem, event: DragEvent<HTMLDivElement>) => void
  /** 当前正在被拖的物品 id(用于高亮原位置) */
  draggingItemId?: string | null
  /** 当前选中的物品 id(用于高亮 + 中间栏位联动) */
  selectedItemId?: string | null
  /** 点击格子时回调(父组件写入 selectedKey) */
  onSelectItem?: (item: StashItem) => void
}

const GEM_CODES: string[][] = [
  //         Amethyst   Diamond   Emerald   Ruby      Sapphire  Skull     Topaz
  /* chip */ ['gcv',    'gcw',    'gcg',    'gcr',    'gcb',    'skc',    'gcy'],
  /* flaw */ ['gfv',    'gfw',    'gfg',    'gfr',    'gfb',    'skf',    'gfy'],
  /* norm */ ['gsv',    'gsw',    'gsg',    'gsr',    'gsb',    'sku',    'gsy'],
  /* flawl*/ ['gzv',    'glw',    'glg',    'glr',    'glb',    'skl',    'gly'],
  /* perf*/ ['gpv',    'gpw',    'gpg',    'gpr',    'gpb',    'skz',    'gpy'],
]

/* ════════════════════════════════════════════════════════════
   尺寸表(等比例 ×4/3,相对原值 +33%):
   - 单元格 44→59  · 内部图标 28→37  · gap 4→5  · 容器 padding 12→16
   - 标题字号 14→19 · 标题下间距 8→11 · 容器间 gap 16→21 · 材料 wrap gap 6→8
   - 数量角标字号 10→13 · 数量角标 padding '0 2px'→'0 3px' · right 2→3
   ════════════════════════════════════════════════════════════ */

/**
 * 堆叠页单元格。两种模式:
 * - 有 item + 有 beginDragStash:可拖动 → dragStart 写入全局 dragPayload,
 *   拖动时该 cell 变半透明(原位置视觉提示),全局 storage-drag-ghost 跟随光标。
 * - 有 item + 无 beginDragStash:只读(角色页用)。
 */
const ITEM_CELL = (
  it: StashItem | undefined,
  key: string,
  opts: {
    draggable: boolean
    isDragging: boolean
    selected?: boolean
    onClick?: () => void
    onDragStart?: (event: DragEvent<HTMLDivElement>) => void
  },
) => {
  const baseStyle: CSSProperties = {
    width: 59, height: 59, borderRadius: 5,
    background: 'rgba(0,0,0,0.5)',
    display: 'flex', alignItems: 'center', justifyContent: 'center',
    border: '1px solid rgba(255,255,255,0.06)',
    position: 'relative',
    transition: 'opacity 120ms ease, border-color 120ms ease, box-shadow 120ms ease',
    cursor: opts.draggable && it ? 'grab' : 'default',
  }
  if (opts.isDragging) {
    baseStyle.opacity = 0.18
    baseStyle.borderColor = 'rgba(251, 177, 58, 0.55)'
    baseStyle.boxShadow = 'inset 0 0 0 2px rgba(251, 177, 58, 0.18)'
  } else if (opts.selected && it) {

    // 不修改 borderColor,保持默认边框
    baseStyle.boxShadow = 'inset 0 0 0 2px rgba(251, 177, 58, 0.45)'
  }
  return (
    <div
      key={key}
      style={baseStyle}
      draggable={opts.draggable && !!it}
      onDragStart={opts.draggable && it && opts.onDragStart ? (event) => opts.onDragStart!(event) : undefined}
      onClick={it && opts.onClick ? opts.onClick : undefined}
      data-code={it?.code}
    >
      {it && (
        <>
          <img
            src={resolveItemIcon({ code: it.code, icon: it.icon })}
            alt={it.code}
            data-code={it.code}
            style={{ width: 37, height: 37, objectFit: 'contain', imageRendering: 'pixelated' }}
            onError={handleImgError}
          />
          {(it.quantity ?? 0) > 1 && (
            <span style={{
              position: 'absolute', bottom: 1, right: 3,
              background: 'rgba(0,0,0,0.85)', color: '#fff',
              fontSize: 13, fontWeight: 700, lineHeight: 1.1,
              padding: '0 3px', borderRadius: 1,
            }}>×{it.quantity}</span>
          )}
        </>
      )}
      {it && (
        <ItemTooltip
          tooltipData={it.tooltip}
          quality={it.quality}
          itemCode={it.code}
          nameZh={it.item_name}
          english={it.name_en}
          quantity={it.quantity}
          mode="hover"
          position="top"
        />
      )}
    </div>
  )
}

function GemGrid({
  items,
  beginDragStash,
  draggingItemId,
  selectedItemId,
  onSelectItem,
}: {
  items: StashItem[]
  beginDragStash?: Props['beginDragStash']
  draggingItemId?: string | null
  selectedItemId?: string | null
  onSelectItem?: (item: StashItem) => void
}) {
  const byCode = new Map(items.map(i => [i.code, i]))
  const draggable = !!beginDragStash
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 59px)', gap: 5 }}>
      {GEM_CODES.flat().map(code => {
        const item = byCode.get(code)
        return ITEM_CELL(item, code, {
          draggable,
          isDragging: !!item && item.id === draggingItemId,
          selected: !!item && item.id === selectedItemId,
          onClick: item && onSelectItem ? () => onSelectItem(item) : undefined,
          onDragStart: beginDragStash && item ? (event) => beginDragStash(item, event) : undefined,
        })
      })}
    </div>
  )
}

function RuneGrid({
  items,
  beginDragStash,
  draggingItemId,
  selectedItemId,
  onSelectItem,
}: {
  items: StashItem[]
  beginDragStash?: Props['beginDragStash']
  draggingItemId?: string | null
  selectedItemId?: string | null
  onSelectItem?: (item: StashItem) => void
}) {
  const byCode = new Map(items.map(i => [i.code, i]))
  const draggable = !!beginDragStash
  const codes: string[] = []
  for (let i = 1; i <= 33; i++) codes.push(`r${i.toString().padStart(2, '0')}`)
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 59px)', gap: 5 }}>
      {codes.map(code => {
        const item = byCode.get(code)
        return ITEM_CELL(item, code, {
          draggable,
          isDragging: !!item && item.id === draggingItemId,
          selected: !!item && item.id === selectedItemId,
          onClick: item && onSelectItem ? () => onSelectItem(item) : undefined,
          onDragStart: beginDragStash && item ? (event) => beginDragStash(item, event) : undefined,
        })
      })}
    </div>
  )
}

export default function StackablePageView({
  items,
  beginDragStash,
  draggingItemId,
  selectedItemId,
  onSelectItem,
}: Props) {
  const groups: Record<string, { label: string; bg: string; items: StashItem[] }> = {
    gem: { label: '宝石', bg: '#1a2a4a', items: [] },
    rune: { label: '符文', bg: '#3a2a0a', items: [] },
    mat: { label: '材料', bg: '#2a2a2a', items: [] },
  }
  for (const it of items) {
    if (it.quantity !== undefined && it.quantity === 0) continue  // zero-amount items are effectively empty
    const kind = it.kind || ''
    if (kind === 'gem' || it.code.startsWith('g')) groups.gem.items.push(it)
    else if (kind === 'rune' || it.code.startsWith('r')) groups.rune.items.push(it)
    else groups.mat.items.push(it)
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 21 }}>
      {/* 第一行: 宝石 + 符文 并行 */}
      <div style={{ display: 'flex', gap: 21, flexWrap: 'wrap', alignItems: 'flex-start' }}>
        {groups.gem.items.length > 0 && (
          <div style={{ background: '#1a2a4a', borderRadius: 10, padding: 16, border: '1px solid rgba(255,255,255,0.08)' }}>
            <div style={{ fontSize: 19, fontWeight: 600, color: '#c7b377', textTransform: 'uppercase', letterSpacing: '0.12em', marginBottom: 11 }}>宝石 ({groups.gem.items.length})</div>
            <GemGrid items={groups.gem.items} beginDragStash={beginDragStash} draggingItemId={draggingItemId} selectedItemId={selectedItemId} onSelectItem={onSelectItem} />
          </div>
        )}
        {groups.rune.items.length > 0 && (
          <div style={{ background: '#3a2a0a', borderRadius: 10, padding: 16, border: '1px solid rgba(255,255,255,0.08)' }}>
            <div style={{ fontSize: 19, fontWeight: 600, color: '#c7b377', textTransform: 'uppercase', letterSpacing: '0.12em', marginBottom: 11 }}>符文 ({groups.rune.items.length})</div>
            <RuneGrid items={groups.rune.items} beginDragStash={beginDragStash} draggingItemId={draggingItemId} selectedItemId={selectedItemId} onSelectItem={onSelectItem} />
          </div>
        )}
      </div>
      {/* 第二行: 材料区 */}
      {groups.mat.items.length > 0 && (
        <div style={{ maxWidth: 970, background: '#2a2a2a', borderRadius: 10, padding: 16, border: '1px solid rgba(255,255,255,0.08)' }}>
          <div style={{ fontSize: 19, fontWeight: 600, color: '#c7b377', textTransform: 'uppercase', letterSpacing: '0.12em', marginBottom: 11 }}>材料 ({groups.mat.items.length})</div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
            {groups.mat.items.map(it => ITEM_CELL(it, it.id, {
              draggable: !!beginDragStash,
              isDragging: it.id === draggingItemId,
              selected: it.id === selectedItemId,
              onClick: onSelectItem ? () => onSelectItem(it) : undefined,
              onDragStart: beginDragStash ? (event) => beginDragStash(it, event) : undefined,
            }))}
          </div>
        </div>
      )}
    </div>
  )
}