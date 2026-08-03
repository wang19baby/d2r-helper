import { useEffect, useState, useMemo, useRef } from 'react'
import { tauriInvoke, fmt } from '../tauri'
import { useToast } from '../hooks/useToast'
import { useFocusTrap } from '../hooks/useFocusTrap'
import { QUALITY_COLOR, type QualityKey } from './QualityLegend'
import type { ListedItem } from '../types'

interface Props {
  item: ListedItem | null
  currentBalance: number | null
  onClose: () => void
  onBought: () => void
}

/**
 * BuyModal — Catalog 点击物品后的购买确认弹窗
 *
 * 沿用 d2emu 单系设计:
 *  - 顶部 d2emu-card + 烫金标题
 *  - 数量 d2emu-stepper
 *  - 总价 / 余额预览
 *  - 余额不足时禁用确认按钮 + 警告色
 *
 * 调 `buy_item` 后:
 *  - 顶栏 balance 刷新(通过 balance-update 事件)
 *  - onBought 回调(让 Catalog reload 列表)
 */
export default function BuyModal({ item, currentBalance, onClose, onBought }: Props) {
  const [qty, setQty] = useState(1)
  const [loading, setLoading] = useState(false)
  const { wrap } = useToast()

  useEffect(() => {
    if (item) setQty(1)
  }, [item?.id])

  const modalRef = useRef<HTMLDivElement>(null)
  useFocusTrap(modalRef, onClose, !!item)

  if (!item) return null

  const quality: QualityKey = (item.quality as QualityKey) || 'normal'
  const qc = QUALITY_COLOR[quality] || QUALITY_COLOR.normal
  const unitPrice = item.unit_price
  const totalPrice = unitPrice * qty
  const balance = currentBalance ?? 0
  const insufficient = balance < totalPrice
  const overLimit = qty > item.quantity

  const kindLabel = useMemo(() => {
    switch (item.item_kind) {
      case 'rune': return '符文'
      case 'gem': return '宝石'
      case 'potion': return '药水'
      case 'key': return '钥匙'
      case 'essence': return '精华'
      case 'shard': return '碎片'
      case 'armor': return '护甲'
      case 'weapon': return '武器'
      case 'shield': return '盾牌'
      case 'jewelry': return '珠宝'
      case 'misc': return '杂项'
      default: return item.item_kind || '其他'
    }
  }, [item.item_kind])

  const submit = async () => {
    if (overLimit) return
    if (insufficient) return
    setLoading(true)
    const res = await wrap(
      () => tauriInvoke('buy_item', {
        itemName: item.name,
        itemKind: item.item_kind || 'rune',
        tokenPrice: unitPrice,
        qty,
      }) as Promise<{ new_balance: number; item_id: string; stash_path: string }>,
      {
        success: `成功购买 ${qty}× ${item.name}`,
        error: (e) => `购买失败: ${e?.message || '未知错误'}`,
      },
    )
    setLoading(false)
    if (res) {
      // 显式派发 balance-update(buy_item 已返回 new_balance)
      window.dispatchEvent(new CustomEvent('balance-update', { detail: res.new_balance }))
      onBought()
      onClose()
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={onClose}>
      <div className="absolute inset-0" style={{ background: 'rgba(0,0,0,0.75)' }} />
      <div ref={modalRef} className="relative d2emu-card d2emu-modal"
        onClick={e => e.stopPropagation()}>

        <div style={{ borderBottom: '1px solid var(--color-d2emu-line)', paddingBottom: 12, marginBottom: 14 }}>
          <p className="d2emu-kicker">确认购买</p>
          <h3 className="font-d2emu-title" style={{
            textAlign: 'left', padding: 0,
            fontSize: 22, letterSpacing: 2, margin: '4px 0 0',
            color: 'var(--color-d2emu-gold-bright, #fff)',
          }}>购买物品</h3>
        </div>

        {/* 物品卡片 */}
        <div style={{
          display: 'flex', gap: 14, alignItems: 'flex-start',
          padding: 12, marginBottom: 14,
          border: `2px solid ${qc}`,
          borderRadius: 4,
          background: 'rgba(0,0,0,0.4)',
          boxShadow: `0 0 14px ${qc}33`,
        }}>
          <div style={{
            width: 56, height: 56, flexShrink: 0,
            background: '#0a0806', border: `1px solid ${qc}55`,
            display: 'grid', placeItems: 'center', borderRadius: 3,
          }}>
            <i className="fa-solid fa-gem"
              style={{ color: qc, fontSize: 28, filter: `drop-shadow(0 0 8px ${qc}66)` }} />
          </div>
          <div className="min-w-0 flex-1">
            <div style={{
              fontSize: 16, fontWeight: 700, color: qc,
              marginBottom: 4,
              textShadow: `0 0 8px ${qc}44`,
            }}>{item.name}</div>
            <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 6 }}>
              {kindLabel}
              {item.item_code && <> · {item.item_code}</>}
            </div>
            <div className="d2emu-tags" style={{ marginTop: 0 }}>
              <span className="d2emu-tag">{quality}</span>
              <span className="d2emu-tag" style={{ color: qc }}>库存 {item.quantity}</span>
              <span className="d2emu-tag">{fmt(unitPrice)} 代币 / 个</span>
            </div>
          </div>
        </div>

        {/* 数量 + 步进器 */}
        <div className="d2emu-field" style={{ marginBottom: 12 }}>
          <label>购买数量</label>
          <div className="d2emu-stepper" role="group" aria-label="购买数量调节">
            <button type="button" disabled={loading || qty <= 1}
              onClick={() => setQty(q => Math.max(1, q - 1))}
              aria-label="减少" aria-controls="buy-qty-input">−</button>
            <input id="buy-qty-input" type="number" min={1} max={item.quantity} value={qty} disabled={loading}
              role="spinbutton"
              aria-valuemin={1}
              aria-valuemax={item.quantity}
              aria-valuenow={qty}
              onChange={e => setQty(Math.max(1, Math.min(item.quantity, Number(e.target.value) || 1)))} />
            <button type="button" disabled={loading || qty >= item.quantity}
              onClick={() => setQty(q => Math.min(item.quantity, q + 1))}
              aria-label="增加" aria-controls="buy-qty-input">+</button>
        </div>
          </div>

        {/* 价格预览 */}
        <div style={{
          display: 'grid', gridTemplateColumns: '1fr 1fr',
          gap: 10, marginBottom: 12,
          padding: 12,
          background: 'rgba(0,0,0,0.35)',
          border: '1px solid var(--color-d2emu-line)',
          borderRadius: 4,
        }}>
          <div>
            <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 2 }}>总价</div>
            <div style={{
              font: '600 20px/1 "Cinzel", serif',
              color: insufficient ? 'var(--color-d2emu-bad, #ef5350)' : 'var(--color-d2emu-gold-bright, #fff)',
              letterSpacing: 1,
            }}>{fmt(totalPrice)}</div>
            <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', marginTop: 2 }}>代币</div>
          </div>
          <div style={{ textAlign: 'right' }}>
            <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 2 }}>你的余额</div>
            <div style={{
              font: '600 20px/1 "Roboto Mono", monospace',
              color: insufficient ? 'var(--color-d2emu-bad, #ef5350)' : 'var(--color-d2emu-text, #e8e8e8)',
            }}>{fmt(balance)}</div>
            <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', marginTop: 2 }}>
              {insufficient
                ? <span style={{ color: 'var(--color-d2emu-bad, #ef5350)' }}>不足 {fmt(totalPrice - balance)}</span>
                : <span style={{ color: 'var(--color-d2emu-good, #4caf50)' }}>剩余 {fmt(balance - totalPrice)}</span>}
            </div>
          </div>
        </div>

        {item.listed_by && (
          <div style={{
            fontSize: 14, color: 'var(--color-d2emu-muted)',
            textTransform: 'uppercase', letterSpacing: '0.08em',
            marginBottom: 12,
          }}>
            <i className="fa-solid fa-user-tag" style={{ marginRight: 6 }} />
            上架人: {item.listed_by.replace(/\.d2s$/i, '')}
          </div>
        )}

        {/* 操作按钮 */}
        <div style={{ display: 'flex', gap: 10 }}>
          <button className="d2emu-btn d2emu-btn-action" style={{ flex: 1 }}
            disabled={loading || insufficient || overLimit}
            onClick={submit}>
            <i className="fa-solid fa-coins" />
            {loading ? '购买中…' : insufficient ? '余额不足' : `确认购买 ${fmt(totalPrice)} 代币`}
          </button>
          <button className="d2emu-btn d2emu-btn-ghost" onClick={onClose} disabled={loading}>
            取消
          </button>
        </div>

        {/* 风险提示 */}
        <div style={{
          marginTop: 14, padding: '8px 10px',
          background: 'rgba(242,184,75,0.06)',
          border: '1px solid rgba(242,184,75,0.3)',
          borderRadius: 4,
          fontSize: 14, color: '#ddd', lineHeight: 1.5,
        }}>
          <i className="fa-solid fa-circle-info" style={{ color: '#f2b84b', marginRight: 6 }} />
          购买后物品将直接写入你的共享仓库(优先堆叠页)。请确保已配置存档路径。
        </div>
      </div>
    </div>
  )
}