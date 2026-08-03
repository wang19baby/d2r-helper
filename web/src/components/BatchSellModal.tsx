/**
 * BatchSellModal — 多件聚合上架 (v2 spec §3.7 P0)
 *
 * 替代 v1 占位: 之前 Inventory batch bar 触发单件 SellModal (按 selectedIds[0])
 * 现在按 selectedIds 全部 item 列出 + 单一单价 + 批量确认 → 循环 invoke list_item。
 *
 * 沿用 d2emu-card / d2emu-modal 装饰 (index.css 已存在)。
 */

import { useEffect, useRef, useState } from 'react'
import type { JSX } from 'react'
import { tauriInvoke } from '../tauri'
import { showToast } from './Toast'
import type { StashItem } from '../types'

export interface BatchSellModalProps {
  items: StashItem[]
  stashFile: string | null
  onClose: () => void
  onDone: () => void  // 完成后回调 (含全成功/部分失败)
}

interface Progress {
  total: number
  done: number
  failed: { name: string; msg: string }[]
}

export default function BatchSellModal({
  items,
  stashFile,
  onClose,
  onDone,
}: BatchSellModalProps): JSX.Element | null {
  const [unitPrice, setUnitPrice] = useState<number>(1)
  const [loading, setLoading] = useState(false)
  const [progress, setProgress] = useState<Progress | null>(null)
  const modalRef = useRef<HTMLDivElement>(null)

  useEffect(() => { /* focus trap omitted; footer button focuses on mount via autoFocus */ }, [])

  if (items.length === 0) return null

  const totalQty = items.reduce((s, i) => s + (i.quantity ?? 0), 0)
  const totalValue = totalQty * unitPrice

  const submit = async (): Promise<void> => {
    if (!Number.isInteger(unitPrice) || unitPrice < 1) {
      showToast('单价必须是正整数', 'error', { position: 'top' })
      return
    }
    setLoading(true)
    const failed: Progress['failed'] = []
    let done = 0
    for (const item of items) {
      try {
        await tauriInvoke('list_item', {
          stashFile,
          itemName: item.item_name,
          itemCode: item.code,
          itemKind: item.kind,
          quantity: item.quantity,
          unitPrice,
        })
        done++
      } catch (e: unknown) {
        failed.push({
          name: item.item_name,
          msg: e instanceof Error ? e.message : String(e),
        })
      }
      setProgress({ total: items.length, done, failed })
    }
    setLoading(false)
    if (failed.length === 0) {
      showToast(`批量上架 ${done} 件成功`, 'success', { position: 'top' })
      onDone()
    } else if (done > 0) {
      showToast(`成功 ${done} 件, 失败 ${failed.length} 件`, 'warning', { position: 'top' })
      onDone()  // 部分成功也回调 (让父级清理选中 + 刷新)
    } else {
      showToast(`全部 ${failed.length} 件失败,保持选中`, 'error', { position: 'top' })
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center d2emu-modal-overlay"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-label={`批量上架 ${items.length} 件`}>
      <div className="absolute inset-0 bg-black/70" />
      <div ref={modalRef} className="relative d2emu-modal d2emu-card"
        onClick={e => e.stopPropagation()}
        style={{ borderRadius: 6, maxWidth: 540, padding: '20px 24px', maxHeight: '85vh', overflow: 'auto' }}>
        <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
          onClick={onClose}
          aria-label="关闭"
          style={{ position: 'absolute', top: 8, right: 8, padding: '4px 10px' }}>✕</button>

        <h3 className="font-d2emu-title" style={{ margin: 0, padding: 0, textAlign: 'left', fontSize: 20 }}>
          <i className="fa-solid fa-tag" style={{ marginRight: 8, color: 'var(--color-d2emu-gold)' }} />
          批量上架 {items.length} 件
        </h3>
        <p className="d2emu-lede" style={{ marginTop: 6, marginBottom: 14, fontSize: 14 }}>
          所有选中物品使用同一单价上架。上架后会自动从共享仓库转出 (写在 listing 数据)。
        </p>

        {/* ── 选中物品预览 ── */}
        <ul style={{
          listStyle: 'none', padding: '8px 12px', margin: '0 0 12px',
          maxHeight: 220, overflowY: 'auto',
          border: '1px solid var(--color-d2emu-line)', borderRadius: 4,
          background: 'rgba(0,0,0,0.30)',
        }}>
          {items.map(i => (
            <li key={i.id}
              style={{ display: 'flex', alignItems: 'center', padding: '4px 0', borderBottom: '1px dotted rgba(255,255,255,0.06)', fontSize: 13 }}>
              <span style={{ flex: 1, color: 'var(--color-d2emu-text)', fontWeight: 600 }}>{i.item_name}</span>
              <span style={{ color: 'var(--color-d2emu-muted)', fontFamily: '"JetBrains Mono", monospace', minWidth: 50, textAlign: 'right' }}>
                ×{i.quantity ?? 0}
              </span>
              <span style={{ color: 'var(--color-d2emu-gold)', fontFamily: '"JetBrains Mono", monospace', minWidth: 80, textAlign: 'right' }}>
                × {unitPrice} = {(i.quantity ?? 0) * unitPrice}
              </span>
            </li>
          ))}
        </ul>

        {/* ── 单价表单 ── */}
        <div className="d2emu-field" style={{ marginBottom: 8 }}>
          <label htmlFor="bsm-price">单价 (代币)</label>
          <input id="bsm-price" type="number" min={1} value={unitPrice}
            autoFocus
            onChange={e => setUnitPrice(Math.max(1, Number(e.target.value)))} />
        </div>

        <div style={{ color: 'var(--color-d2emu-muted)', fontSize: 13, marginBottom: 14 }}>
          总件数 <strong style={{ color: 'var(--color-d2emu-text)' }}>{totalQty}</strong> 件 ·
          估算总价 <strong style={{ color: 'var(--color-d2emu-gold)' }}>{totalValue}</strong> 代币
        </div>

        {/* ── 进度 ── */}
        {progress && (
          <div role="status" aria-live="polite"
            style={{
              padding: '8px 12px', borderRadius: 4,
              background: progress.failed.length > 0
                ? 'linear-gradient(180deg, rgba(184,134,11,0.12), rgba(184,134,11,0.04))'
                : 'rgba(76,175,80,0.10)',
              border: '1px solid var(--color-d2emu-line)',
              marginBottom: 12, fontSize: 13,
            }}>
            进度 {progress.done}/{progress.total}
            {progress.failed.length > 0 && ` · 失败 ${progress.failed.length}`}
          </div>
        )}

        <div className="flex gap-2.5 mt-4">
          <button className="d2emu-btn d2emu-btn-primary flex-1"
            disabled={loading}
            onClick={submit}>
            {loading
              ? `上架中 ${progress?.done ?? 0}/${items.length}...`
              : <><i className="fa-solid fa-check" /> 确认上架 {items.length} 件</>}
          </button>
          <button className="d2emu-btn d2emu-btn-ghost flex-1"
            disabled={loading}
            onClick={onClose}>取消</button>
        </div>
      </div>
    </div>
  )
}
