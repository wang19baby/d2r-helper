/**
 * BatchDepositModal — 多件聚合存入扩展仓库 (v2 spec §3.7 P0)
 *
 * 替代 v1 占位: 之前 Inventory batch bar 触发单件 DConfirmModal (按 selectedIds[0])
 * 现在对所有 selectedIds 一次性 confirm → 循环 warehouse_deposit,
 * 显示每件结果 (成功 / 失败 msg)。
 */

import { useEffect, useState } from 'react'
import type { JSX } from 'react'
import { tauriInvoke } from '../tauri'
import { showToast } from './Toast'
import type { StashItem } from '../types'

export interface BatchDepositModalProps {
  items: StashItem[]
  stashFile: string | null
  onClose: () => void
  onDone: () => void
}

interface Progress {
  total: number
  done: number
  failed: { name: string; msg: string }[]
}

export default function BatchDepositModal({
  items,
  stashFile,
  onClose,
  onDone,
}: BatchDepositModalProps): JSX.Element | null {
  const [loading, setLoading] = useState(false)
  const [progress, setProgress] = useState<Progress | null>(null)

  useEffect(() => { /* noop */ }, [])

  if (items.length === 0) return null

  const totalQty = items.reduce((s, i) => s + (i.quantity ?? 0), 0)

  const submit = async (): Promise<void> => {
    setLoading(true)
    const failed: Progress['failed'] = []
    let done = 0
    for (const item of items) {
      try {
        await tauriInvoke('warehouse_deposit', {
          stashPath: stashFile,
          itemCode: item.code,
          pageIndex: item.page_index,
          quantity: item.quantity,
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
      showToast(`成功存入 ${done} 件到扩展仓库`, 'success', { position: 'top' })
      onDone()
    } else if (done > 0) {
      showToast(`成功 ${done} 件, 失败 ${failed.length} 件`, 'warning', { position: 'top' })
      onDone()
    } else {
      showToast(`全部 ${failed.length} 件失败,保持选中`, 'error', { position: 'top' })
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center d2emu-modal-overlay"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-label={`批量存入扩展仓库 ${items.length} 件`}>
      <div className="absolute inset-0 bg-black/70" />
      <div className="relative d2emu-modal d2emu-card"
        onClick={e => e.stopPropagation()}
        style={{ borderRadius: 6, maxWidth: 540, padding: '20px 24px', maxHeight: '85vh', overflow: 'auto' }}>
        <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
          onClick={onClose}
          aria-label="关闭"
          style={{ position: 'absolute', top: 8, right: 8, padding: '4px 10px' }}>✕</button>

        <h3 className="font-d2emu-title" style={{ margin: 0, padding: 0, textAlign: 'left', fontSize: 20 }}>
          <i className="fa-solid fa-box-archive" style={{ marginRight: 8, color: 'var(--color-d2emu-gold)' }} />
          批量存入 {items.length} 件
        </h3>
        <p className="d2emu-lede" style={{ marginTop: 6, marginBottom: 14, fontSize: 14, color: 'var(--color-d2emu-bad, #ef5350)' }}>
          <i className="fa-solid fa-triangle-exclamation" style={{ marginRight: 6 }} />
          物品将从共享仓库转出到扩展仓库。
        </p>

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
              <span style={{
                color: i.quality === 'unique' ? 'var(--color-d2emu-gold)' :
                       i.quality === 'set'    ? '#3bbf4f' :
                       i.quality === 'rare'   ? '#ffd700' :
                                                  'var(--color-d2emu-muted)',
                fontFamily: '"JetBrains Mono", monospace', minWidth: 60, textAlign: 'right',
                textTransform: 'uppercase', fontSize: 12,
              }}>{i.quality || 'normal'}</span>
              <span style={{ color: 'var(--color-d2emu-muted)', fontFamily: '"JetBrains Mono", monospace', minWidth: 60, textAlign: 'right' }}>
                ×{i.quantity}
              </span>
            </li>
          ))}
        </ul>

        <div style={{ color: 'var(--color-d2emu-muted)', fontSize: 13, marginBottom: 14 }}>
          共 <strong style={{ color: 'var(--color-d2emu-text)' }}>{items.length}</strong> 项 ·
          总件数 <strong style={{ color: 'var(--color-d2emu-text)' }}>{totalQty}</strong> 件
        </div>

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
          <button className="d2emu-btn d2emu-btn-action flex-1"
            disabled={loading}
            onClick={submit}>
            {loading
              ? `存入中 ${progress?.done ?? 0}/${items.length}...`
              : <><i className="fa-solid fa-check" /> 确认存入 {items.length} 项</>}
          </button>
          <button className="d2emu-btn d2emu-btn-ghost flex-1"
            disabled={loading}
            onClick={onClose}>取消</button>
        </div>
      </div>
    </div>
  )
}
