import { useState, useRef } from 'react'
import { useFocusTrap } from '../hooks/useFocusTrap'
import { tauriInvoke } from '../tauri'
import { showToast } from './Toast'
import type { StashItem, PriceSuggestion } from '../types'

interface Props {
  item: StashItem | null
  stashFile: string | null
  onClose: () => void
  onListed: () => void
}

export default function SellModal({ item, stashFile, onClose, onListed }: Props) {
  const [qty, setQty] = useState(1)
  const [price, setPrice] = useState(1)
  const [validationError, setValidationError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const modalRef = useRef<HTMLDivElement>(null)
  useFocusTrap(modalRef, onClose, !!item)

  if (!item) return null

  const handlePriceSuggestion = async () => {
    try {
      const res = await tauriInvoke('get_price_suggestion', { itemName: item.item_name, itemKind: item.kind }) as PriceSuggestion
      if (res.suggested_price > 0) setPrice(res.suggested_price)
    } catch { /* ignore */ }
  }

  const submit = async () => {
    setValidationError(null)
    if (!Number.isInteger(qty) || qty < 1) { setValidationError('数量无效。'); return showToast('数量无效。', 'error') }
    if (qty > (item.quantity ?? 0)) { setValidationError('数量超出库存。'); return showToast('数量超出库存。', 'error') }
    if (!Number.isInteger(price) || price < 1) { setValidationError('单价无效。'); return showToast('单价无效。', 'error') }
    setLoading(true)
    try {
      await tauriInvoke('list_item', {
        itemName: item.item_name, itemCode: item.code,
        itemKind: item.kind, quantity: qty, unitPrice: price,
        stashFile: stashFile || null,
      })
      showToast('物品上架成功！', 'success')
      onListed()
    } catch (e: any) {
      showToast(e.message || '上架失败。', 'error')
    } finally { setLoading(false) }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={onClose}>
      <div className="absolute inset-0 bg-black/70" />
      <div ref={modalRef} className="relative d2-panel p-6 min-w-[380px] max-w-[460px] max-h-[80vh] overflow-y-auto" onClick={e => e.stopPropagation()}>
        <h3 className="text-d2-gold font-serif text-lg mb-1">创建上架</h3>
        <p className="text-d2-text-soft text-sm mb-4">设置数量和单价，将物品上架到市场。</p>
        <div className="space-y-3">
          <div><strong>物品：</strong><span>{item.item_name}</span></div>
          <div><strong>库存：</strong><span>{item.quantity}</span></div>
          <div>
            <label className="block text-d2-text-soft text-sm mb-1" htmlFor="sell-qty">数量</label>
            <input id="sell-qty" type="number" min={1} max={item.quantity} value={qty}
              onChange={e => { setQty(Number(e.target.value)); setValidationError(null) }}
              className="d2-input"
              aria-describedby={validationError ? 'sell-validation-msg' : undefined}
              aria-invalid={!!validationError} />
          </div>
          <div>
            <label className="block text-d2-text-soft text-sm mb-1" htmlFor="sell-price">单价（代币）</label>
            <input id="sell-price" type="number" min={1} value={price}
              onChange={e => { setPrice(Number(e.target.value)); setValidationError(null) }}
              className="d2-input"
              aria-describedby={validationError ? 'sell-validation-msg' : undefined}
              aria-invalid={!!validationError} />
            <button onClick={handlePriceSuggestion} className="text-xs text-d2-gold/70 hover:text-d2-gold mt-1">参考建议售价</button>
          </div>
          {validationError && (
            <div id="sell-validation-msg" role="alert" aria-live="polite"
              style={{ color: '#ef5350', fontSize: 14, marginTop: 4 }}>
              <i className="fa-solid fa-circle-exclamation" style={{ marginRight: 6 }} />
              {validationError}
            </div>
          )}
        </div>
        <div className="flex gap-2.5 mt-4">
          <button className="d2-btn flex-1" disabled={loading} onClick={submit}>{loading ? '上架中...' : <><i className="fa-solid fa-check" /> 确认上架</>}</button>
          <button className="d2-btn d2-btn-secondary flex-1" onClick={onClose}>取消</button>
        </div>
      </div>
    </div>
  )
}
