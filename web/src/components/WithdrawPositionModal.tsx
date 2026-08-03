/**
 * WithdrawPositionModal — 取回扩展仓库物品时选 d2i 页 + 坐标 (v2 P0)
 *
 * 沿用 d2emu-card / d2emu-modal 装饰 (index.css 已存在)。
 *
 * 数据命令: warehouse_withdraw(itemId, stashPath, pageIndex, positionX, positionY)
 *
 * 用户主动选位 (v1 = 自动堆 page=0, x=0, y=0 经常撞车;现在可控可追溯)
 */

import { useEffect, useState } from 'react'
import type { JSX } from 'react'
import { tauriInvoke } from '../tauri'
import { stashStore } from '../cache/stash'
import { showToast } from './Toast'
import type { WarehouseItem } from '../types'

export interface WithdrawPositionModalProps {
  item: WarehouseItem | null
  stashFile: string | null
  onClose: () => void
  onWithdrawn: () => void
}

interface PageInfo {
  index: number
  label: string
  item_count: number
  grid_width: number
  grid_height: number
}

const DEFAULT_PAGE = { index: 0, label: '高级页·堆叠物品', item_count: 0, grid_width: 16, grid_height: 16 }

export default function WithdrawPositionModal({
  item,
  stashFile,
  onClose,
  onWithdrawn,
}: WithdrawPositionModalProps): JSX.Element | null {
  const [pages, setPages] = useState<PageInfo[]>([])
  const [pageIndex, setPageIndex] = useState<number>(0)
  const [x, setX] = useState<number>(0)
  const [y, setY] = useState<number>(0)
  const [loading, setLoading] = useState<boolean>(false)

  // 进入 modal 时拉 d2i 页列表,默认选中 item_count 最少的页
  useEffect(() => {
    if (!item) return
    let cancelled = false
    stashStore.fetch('shared')
      .then(stash => {
        if (cancelled) return
        setPages(stash.pages)
        // 自动选 item_count 最少的非空页 (期望最有空位)
        const sorted = [...stash.pages].sort((a, b) => a.item_count - b.item_count)
        setPageIndex(sorted[0]?.index ?? 0)
        setX(0)
        setY(0)
      })
      .catch(() => {
        if (cancelled) return
        // fallback: 假定 1 页 16×16
        setPages([DEFAULT_PAGE])
      })
    return () => { cancelled = true }
  }, [item?.id])

  if (!item) return null

  const currentPage = pages.find(p => p.index === pageIndex) ?? DEFAULT_PAGE
  const maxX = Math.max(0, currentPage.grid_width - 1)
  const maxY = Math.max(0, currentPage.grid_height - 1)

  const submit = async (): Promise<void> => {
    if (!stashFile) {
      showToast('未找到游戏仓库文件，请先配置存档路径', 'error', { position: 'top' })
      return
    }
    if (x < 0 || x > maxX || y < 0 || y > maxY) {
      showToast(`坐标越界: x ∈ [0, ${maxX}], y ∈ [0, ${maxY}]`, 'error', { position: 'top' })
      return
    }
    setLoading(true)
    try {
      await tauriInvoke('warehouse_withdraw', {
        itemId: item.id,
        stashPath: stashFile,
        pageIndex,
        positionX: x,
        positionY: y,
      })
      showToast(`已取出 ${item.item_name ?? item.item_code}`, 'success', { position: 'top' })
      onWithdrawn()
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      showToast(msg || '取出失败', 'error', { position: 'top' })
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center d2emu-modal-overlay"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-label={`取出 ${item.item_name ?? item.item_code}`}>
      <div className="absolute inset-0 bg-black/70" />
      <div className="relative d2emu-modal d2emu-card"
        onClick={e => e.stopPropagation()}
        style={{ borderRadius: 6, maxWidth: 520, padding: '20px 24px', maxHeight: '85vh', overflow: 'auto' }}>
        <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
          onClick={onClose}
          aria-label="关闭"
          style={{ position: 'absolute', top: 8, right: 8, padding: '4px 10px' }}>✕</button>
        <h3 className="font-d2emu-title" style={{ margin: 0, padding: 0, textAlign: 'left', fontSize: 20 }}>
          <i className="fa-solid fa-box-archive" style={{ marginRight: 8, color: 'var(--color-d2emu-gold)' }} />
          取回 {item.item_name ?? item.item_code} ×{item.quantity}
        </h3>
        <p className="d2emu-lede" style={{ marginTop: 6, marginBottom: 16, fontSize: 14 }}>
          选择 d2i 页与坐标。脚本会把物品从扩展仓库写回游戏仓库的对应位置。
        </p>

        <div className="d2emu-field" style={{ marginBottom: 12 }}>
          <label htmlFor="wpm-page">目标页</label>
          <select id="wpm-page" value={pageIndex} onChange={e => setPageIndex(Number(e.target.value))}>
            {pages.length === 0 && <option value={0}>加载页列表...</option>}
            {pages.map(p => (
              <option key={p.index} value={p.index}>
                第 {p.index + 1} 页 · {p.label} · {p.item_count}/{p.grid_width * p.grid_height}
              </option>
            ))}
          </select>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, marginBottom: 12 }}>
          <div className="d2emu-field">
            <label htmlFor="wpm-x">X 坐标 (0-{maxX})</label>
            <input id="wpm-x" type="number" min={0} max={maxX} value={x}
              onChange={e => setX(Math.max(0, Math.min(maxX, Number(e.target.value))))} />
          </div>
          <div className="d2emu-field">
            <label htmlFor="wpm-y">Y 坐标 (0-{maxY})</label>
            <input id="wpm-y" type="number" min={0} max={maxY} value={y}
              onChange={e => setY(Math.max(0, Math.min(maxY, Number(e.target.value))))} />
          </div>
        </div>

        {item.inv_width && item.inv_height && (
          <div style={{
            padding: '8px 12px',
            border: '1px dashed var(--color-d2emu-line)',
            borderRadius: 4,
            color: 'var(--color-d2emu-muted)',
            fontSize: 13,
            marginBottom: 12,
            background: 'rgba(0,0,0,0.3)',
          }}>
            <i className="fa-solid fa-info-circle" style={{ marginRight: 6, color: 'var(--color-d2emu-gold)' }} />
            物品占 <strong style={{ color: 'var(--color-d2emu-text)' }}>{item.inv_width}×{item.inv_height}</strong> 格
            ——
            自动占位会从 <code style={{ color: 'var(--color-d2emu-gold-bright)' }}>({x},{y})</code> 起向右下扩展到
            <code style={{ color: 'var(--color-d2emu-gold-bright)' }}> ({x + (item.inv_width ?? 1) - 1},{y + (item.inv_height ?? 1) - 1})</code>。
            请确保这块区域是空的。
          </div>
        )}

        <div className="flex gap-2.5 mt-4">
          <button className="d2emu-btn d2emu-btn-primary flex-1"
            disabled={loading}
            onClick={submit}>
            {loading ? '取出中...' : (<><i className="fa-solid fa-check" /> 确认取出</>)}
          </button>
          <button className="d2emu-btn d2emu-btn-ghost flex-1"
            onClick={onClose}>取消</button>
        </div>
      </div>
    </div>
  )
}
