import type { EquipSlot, EquippedItem } from './EquipmentPanel'
import { SLOT_LABEL } from './EquipmentPanel'
import ItemTooltip from './ItemTooltip'
import { resolveItemIcon, handleImgError } from '../utils/itemImages'

export interface EquipmentDetailModalProps {
  /** 当前选中的装备;null 时不渲染 */
  selected: { slot: EquipSlot; item: EquippedItem } | null
  onClose: () => void
}

/**
 * EquipmentDetailModal — 装备 12 槽点击后弹出的详情面板
 *
 * 模式参考 d2emu 单系 detail panel (不打开新页面,直接 modal):
 *  - 暗背景 overlay,click 外部关闭
 *  - 居中 d2emu-card 容器 (min-w 380 / max-w 460)
 *  - 左侧 64×64 物品图标 + 右侧 ItemTooltip inline (含 stats / meta / 镶嵌)
 *  - 顶右 ✕ 关闭
 *
 * 不用 SellModal 因为:
 *  SellModal 是"上架到市场"操作(有数量/价格表单),
 *  本组件是"查看详情"(只读),语义不同,共用会引入死代码。
 */
export default function EquipmentDetailModal({ selected, onClose }: EquipmentDetailModalProps) {
  if (!selected) return null
  const { slot, item } = selected

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-label={`装备详情 — ${SLOT_LABEL[slot]}`}
    >
      <div className="absolute inset-0 bg-black/70" />

      <div
        className="relative d2-panel p-6 max-h-[80vh] overflow-y-auto d2emu-modal"
        onClick={e => e.stopPropagation()}
        style={{ borderRadius: 6 }}
      >
        {/* 顶右 ✕ */}
        <button
          type="button"
          onClick={onClose}
          aria-label="关闭"
          style={{
            position: 'absolute', top: 8, right: 8,
            background: 'transparent', border: 'none', cursor: 'pointer',
            color: 'var(--color-d2emu-muted, #888)', fontSize: 18, lineHeight: 1,
            padding: '4px 8px', borderRadius: 3,
          }}
          onMouseEnter={e => { e.currentTarget.style.color = 'var(--color-d2emu-gold, #FBB13A)' }}
          onMouseLeave={e => { e.currentTarget.style.color = 'var(--color-d2emu-muted, #888)' }}
        >
          ✕
        </button>

        {/* 标题行:槽位名 + 品质 chip */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 12 }}>
          <h3 style={{
            margin: 0,
            fontFamily: '"Cinzel", "Trajan Pro", serif',
            fontSize: 16, fontWeight: 700, letterSpacing: '0.06em',
            color: 'var(--color-d2emu-gold-bright, #fff)',
            textTransform: 'uppercase',
          }}>
            {SLOT_LABEL[slot]}
          </h3>
          <span style={{
            padding: '2px 8px',
            border: '1px solid var(--color-d2emu-line, #252525)',
            borderRadius: 999,
            color: 'var(--color-d2emu-muted, #888)',
            font: '600 14px/1 "Source Sans 3", sans-serif',
            textTransform: 'uppercase', letterSpacing: '0.06em',
          }}>
            {item.quality}
          </span>
        </div>

        {/* 主体:左侧图标,右侧 tooltip */}
        <div style={{ display: 'flex', gap: 14, alignItems: 'flex-start' }}>
          <div style={{
            flexShrink: 0,
            minWidth: 56, minHeight: 56,
            padding: 4,
            border: '1px solid var(--color-d2emu-line, #252525)',
            borderRadius: 4,
            background: 'rgba(0,0,0,0.4)',
            display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
            position: 'relative',
          }}>
            <img
              src={resolveItemIcon(item)}
              alt={item.code}
              style={{ width: 'auto', height: 'auto', maxWidth: 'none', maxHeight: 'none', imageRendering: 'pixelated' }}
              onError={handleImgError}
            />
            {item.socketed && (
              <span style={{
                position: 'absolute', top: 3, right: 3,
                color: '#8a6aaa', fontSize: 14, fontWeight: 700, lineHeight: 1,
              }}>◈</span>
            )}
          </div>

          <div style={{ flex: 1, minWidth: 0 }}>
            <ItemTooltip
              tooltipLines={(item as any).tooltip_lines ?? null}
              tooltipData={item.tooltipData ?? null}
              quality={item.quality}
              itemCode={item.code}
              nameZh={item.name_zh}
              english={item.name_en}
              stats={item.stats ?? null}
              mode="inline"
            />
          </div>
        </div>
      </div>
    </div>
  )
}
