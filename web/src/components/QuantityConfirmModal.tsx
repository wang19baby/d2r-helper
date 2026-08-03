import { useEffect, useState } from 'react'
import D2ConfirmModal from './D2ConfirmModal'
import { handleImgError, resolveItemIcon } from '../utils/itemImages'

interface ItemSummary {
  item_name: string
  code: string
  icon?: string
  quantity: number
  /** 副标识(deposit 显示坐标,withdraw 显示入仓时间) */
  subtitle?: string
}

interface Props {
  /** 触发操作的源物品摘要(deposit 来自 stash,withdraw 来自 warehouse) */
  item: ItemSummary
  /** 操作模式 */
  mode: 'deposit' | 'withdraw'
  /** 人类可读的 page 标签(deposit 用收藏页名 / 默认页,withdraw 用堆叠页 label) */
  pageLabel: string
  /** deposit:undefined = 后端 fallback 到 per-code default(显示「默认收藏页(后端解析)」) */
  pageName?: string | null
  /** 确认时回调,返回用户选的数量(已 clamp 到 [1, maxQty]) */
  onConfirm: (quantity: number) => void
  /** 取消 / 点遮罩关闭 */
  onClose: () => void
  /** 后端正在执行,禁用按钮 + loading 状态 */
  loading?: boolean
}

/**
 * QuantityConfirmModal — 数量确认弹窗(deposit / withdraw 共用)
 *
 * 任意"将物品从 X 移动到 Y"的操作都先打开这个弹窗,让用户:
 *  1. 看到目标物品(图标 + 中文名 + code)
 *  2. 看到目标位置(收藏页 / 堆叠页)
 *  3. 调整数量(number input + -/+ 按钮 + 全量快捷)
 *  4. 点确认才真正调后端 IPC
 *
 * 单件物品(quantity <= 1)不渲染数量输入,直接显示提示文案。
 */
export default function QuantityConfirmModal({
  item,
  mode,
  pageLabel,
  pageName,
  onConfirm,
  onClose,
  loading = false,
}: Props) {
  const maxQty = Math.max(1, item.quantity ?? 1)
  const [pending, setPending] = useState<number>(maxQty)

  // item 切换时重置 pending 为新 maxQty(用 id 替代 — 这里没有 id, 用 code 兜底)
  const itemKey = `${item.code}:${item.quantity}`
  useEffect(() => { setPending(maxQty) }, [maxQty, itemKey])

  const isFullOnly = maxQty <= 1
  const isFallback = mode === 'deposit' && pageName == null
  const isDeposit = mode === 'deposit'

  const titleIcon = isDeposit ? 'fa-box-open' : 'fa-inbox-out'
  const actionVerb = isDeposit ? '存入' : '取回'
  const confirmText = loading ? '处理中…' : (isDeposit ? '确认存入' : '确认取回')
  const targetLabel = isDeposit ? '目标收藏页' : '目标堆叠页'
  const targetIcon = isDeposit ? 'fa-folder-tree' : 'fa-inbox'

  return (
    <D2ConfirmModal
      title={
        <span>
          <i className={`fa-solid ${titleIcon}`} style={{ marginRight: 8 }} />
          {actionVerb} {item.item_name}
          {isFullOnly ? '' : ` × ${pending}`}
        </span>
      }
      confirmText={confirmText}
      cancelText="取消"
      loading={loading}
      onConfirm={() => onConfirm(pending)}
      onClose={onClose}
    >
      <div style={{ display: 'flex', gap: 12, alignItems: 'center', marginBottom: 12 }}>
        <img
          src={resolveItemIcon({ code: item.code, icon: item.icon })}
          alt={item.item_name}
          data-code={item.code}
          onError={handleImgError}
          style={{ width: 48, height: 48, objectFit: 'contain', imageRendering: 'pixelated' }}
        />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontWeight: 600, fontSize: 15 }}>{item.item_name}</div>
          <div style={{ fontSize: 12, opacity: 0.7 }}>
            code: <code>{item.code}</code>
            {item.subtitle ? ` · ${item.subtitle}` : ''}
          </div>
        </div>
      </div>

      <div style={{
        padding: '8px 10px',
        borderRadius: 4,
        background: 'rgba(251, 177, 58, 0.08)',
        border: '1px solid rgba(251, 177, 58, 0.25)',
        fontSize: 13,
        marginBottom: 12,
      }}>
        <i className={`fa-solid ${targetIcon}`} style={{ marginRight: 6 }} />
        {targetLabel}:
        <strong style={{ marginLeft: 4 }}>
          {isFallback ? '默认收藏页(后端自动解析)' : pageLabel}
        </strong>
        {isFallback && (
          <div style={{ fontSize: 11, opacity: 0.75, marginTop: 4 }}>
            由 per-code 默认收藏页配置决定;若未设置则落到「默认收藏」系统页。
          </div>
        )}
      </div>

      {isFullOnly ? (
        <div style={{ fontSize: 13, opacity: 0.85 }}>
          <i className="fa-solid fa-info-circle" style={{ marginRight: 6 }} />
          这件物品数量为 1,无需选择数量。
        </div>
      ) : (
        <>
          <div style={{ fontSize: 12, fontWeight: 600, opacity: 0.85, marginBottom: 6 }}>
            {actionVerb}数量(最大 <span style={{ color: 'var(--color-d2emu-gold-bright)' }}>{maxQty}</span>)
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <button
              type="button"
              className="d2emu-btn d2emu-btn-ghost"
              style={{ flex: '0 0 auto', minWidth: 36, padding: '0 10px' }}
              disabled={loading || pending <= 1}
              onClick={() => setPending(q => Math.max(1, q - Math.max(1, Math.floor(maxQty / 20))))}
            >−</button>
            <input
              type="number"
              min={1}
              max={maxQty}
              step={1}
              value={pending}
              disabled={loading}
              onChange={(event) => {
                const raw = Number(event.target.value)
                if (Number.isNaN(raw)) return
                setPending(Math.min(maxQty, Math.max(1, Math.floor(raw))))
              }}
              style={{
                flex: 1, minWidth: 0, height: 36,
                textAlign: 'center', fontSize: 16, fontWeight: 700,
                borderRadius: 4,
                border: '1px solid var(--color-d2emu-line)',
                background: 'rgba(0,0,0,0.5)', color: 'inherit',
              }}
            />
            <button
              type="button"
              className="d2emu-btn d2emu-btn-ghost"
              style={{ flex: '0 0 auto', minWidth: 36, padding: '0 10px' }}
              disabled={loading || pending >= maxQty}
              onClick={() => setPending(q => Math.min(maxQty, q + Math.max(1, Math.floor(maxQty / 20))))}
            >+</button>
            <button
              type="button"
              className="d2emu-btn d2emu-btn-ghost"
              style={{ flex: '0 0 auto', padding: '0 10px' }}
              disabled={loading || pending >= maxQty}
              onClick={() => setPending(maxQty)}
            >全量</button>
          </div>
          <div style={{ fontSize: 11, opacity: 0.7, marginTop: 6 }}>
            {isDeposit
              ? `剩余将留在共享仓库(本次写入后源物品 amount 减 ${pending})`
              : `剩余 ${maxQty - pending} 个将留在收藏仓库(本次写入 stash 的新 item amount = ${pending})`}
          </div>
        </>
      )}
    </D2ConfirmModal>
  )
}