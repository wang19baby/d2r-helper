import { useState, useMemo, useRef, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { tauriInvoke, fmt } from '../tauri'
import { showToast } from '../components/Toast'
import EmptyState from '../components/EmptyState'
import D2EmuLoading from '../components/D2EmuLoading'
import KpiRow from '../components/KpiRow'
import type { ListedItem } from '../types'
import D2ConfirmModal from '../components/D2ConfirmModal'
import { useFocusTrap } from '../hooks/useFocusTrap'
const GOLD = 'var(--color-d2emu-gold, #FBB13A)'




export default function Listings() {
  const [items, setItems] = useState<ListedItem[]>([])
  const [loading, setLoading] = useState(true)
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null)
  const [confirmQuickSellItem, setConfirmQuickSellItem] = useState<ListedItem | null>(null)
  const [repriceItem, setRepriceItem] = useState<ListedItem | null>(null)
  const [newPrice, setNewPrice] = useState<number>(0)
  const navigate = useNavigate()

  const repriceModalRef = useRef<HTMLDivElement>(null)
  useFocusTrap(repriceModalRef, () => setRepriceItem(null), !!repriceItem)
  const load = async () => {
    setLoading(true)
    try {
      const res = await tauriInvoke('get_listed_items') as ListedItem[]
      setItems(res)
    } catch { setItems([]) }
    finally { setLoading(false) }
  }

  useEffect(() => { load() }, [])
  const executeCancel = async (id: string) => {
    try {
      await tauriInvoke('cancel_listing', { listingId: id })
      showToast('上架已取消,物品已归还仓库。', 'success')
      setConfirmDeleteId(null)
      load()
    } catch (e: any) { showToast(e.message || '取消失败。', 'error') }
  }

  const cancel = async (id: string) => {
    setConfirmDeleteId(id)
  }

  /**
   * 立即卖出换 token (止损)
   */
  const executeQuickSell = async (item: ListedItem) => {
    try {
      const newBalance = await tauriInvoke('sell_item', { itemId: item.id }) as number
      window.dispatchEvent(new CustomEvent('balance-update', { detail: newBalance }))
      showToast(`已卖出 ${item.name}, 余额: ${fmt(newBalance)} 代币`, 'success')
      setConfirmQuickSellItem(null)
      load()
    } catch (e: any) {
      showToast(e.message || '卖出失败。', 'error')
    }
  }

  const quickSell = (item: ListedItem) => {
    setConfirmQuickSellItem(item)
  }

  /** 打开改价 modal, 默认填当前价格 */
  const openReprice = (item: ListedItem) => {
    setRepriceItem(item)
    setNewPrice(item.unit_price)
  }

  /** 提交新价格 */
  const submitReprice = async () => {
    if (!repriceItem) return
    if (!Number.isInteger(newPrice) || newPrice < 1) {
      showToast('价格必须为正整数', 'error')
      return
    }
    if (newPrice === repriceItem.unit_price) {
      showToast('价格未变, 无需改价', 'info')
      setRepriceItem(null)
      return
    }
    try {
      const ok = await tauriInvoke('update_listing_price', {
        itemId: repriceItem.id,
        newUnitPrice: newPrice,
      }) as boolean
      if (ok) {
        showToast(`已改价: ${repriceItem.name} → ${fmt(newPrice)} 代币/个`, 'success')
        setRepriceItem(null)
        load()
      } else {
        showToast('改价失败: 物品可能已下架', 'error')
      }
    } catch (e: any) {
      showToast(e.message || '改价失败', 'error')
    }
  }

  /** 跳转到 History 并预填 item_id 过滤(查谁买了这件) */
  const viewHistory = (item: ListedItem) => {
    if (!item.id) {
      showToast('该上架缺少 ID,无法定位交易记录。', 'warning')
      return
    }
    navigate(`/history?itemId=${encodeURIComponent(item.id)}&itemName=${encodeURIComponent(item.name)}`)
  }

  const totals = useMemo(() => {
    const totalQty = items.reduce((s, i) => s + i.quantity, 0)
    const totalValue = items.reduce((s, i) => s + i.quantity * i.unit_price, 0)
    const avgPrice = items.length ? items.reduce((s, i) => s + i.unit_price, 0) / items.length : 0
    return { totalQty, totalValue, avgPrice }
  }, [items])

  return (
    <div className="font-d2emu-ui flex flex-col" style={{ gap: 12 }}>
      {/* Hero 段 */}
      <section className="d2emu-card">
        <div className="flex items-start gap-4 flex-wrap">
          <img className="d2emu-portrait" alt="marketplace"
            src="data:image/svg+xml;utf8,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 64 64%27%3E%3Crect width=%2764%27 height=%2764%27 fill=%27%23100a05%27/%3E%3Cpath d=%27M16 24h32v18a4 4 0 01-4 4H20a4 4 0 01-4-4V24z%27 fill=%27none%27 stroke=%27%23FBB13A%27 stroke-width=%271.5%27/%3E%3Cpath d=%27M22 24v-4a10 10 0 0120 0v4%27 stroke=%27%23FBB13A%27 stroke-width=%271.5%27 fill=%27none%27/%3E%3Ccircle cx=%2732%27 cy=%2734%27 r=%273%27 fill=%27%23FBB13A%27/%3E%3C/svg%3E" />
          <div className="flex-1 min-w-0">
            <p className="d2emu-kicker">我的上架</p>
            <h1 className="font-d2emu-title" style={{ textAlign: 'left', padding: 0 }}>拍卖行</h1>
            <p className="d2emu-lede" style={{ marginTop: 6 }}>
              管理你的活跃上架,追踪已完成交易与累计收益。
            </p>
            <div className="d2emu-tags">
              <span className="d2emu-tag">{items.length} 件在售</span>
              <span className="d2emu-tag d2emu-tag-active">本地数据库</span>
              <span className="d2emu-tag">实时刷新</span>
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={load} disabled={loading}>
              <i className="fa-solid fa-rotate-right" /> 刷新
            </button>
          </div>
        </div>
      </section>

      {/* KPI 摘要 */}
      <KpiRow
        items={[
          {
            label: '活跃上架',
            value: items.length,
            delta: items.length ? '出售中' : '待上架',
            trend: items.length ? 'up' : 'neutral',
          },
          {
            label: '总数量',
            value: totals.totalQty,
            delta: `平均每单 ${items.length ? (totals.totalQty / items.length).toFixed(1) : '0'} 件`,
            trend: 'neutral',
          },
          {
            label: '总价值',
            value: `${fmt(totals.totalValue)} 代币`,
            delta: '累计在售金额',
            trend: totals.totalValue > 0 ? 'up' : 'neutral',
            gold: true,
          },
          {
            label: '平均单价',
            value: items.length ? `${fmt(totals.avgPrice)} 代币` : '—',
            delta: '所有上架均价',
            trend: 'neutral',
          },
        ]}
      />

      {/* 活跃上架列表 */}
      <section className="d2emu-card">
        <header className="d2emu-card-header">
          <div className="min-w-0">
            <p className="d2emu-kicker">活跃中</p>
            <h2 className="d2emu-card-title">活跃上架</h2>
          </div>
          <div className="d2emu-card-actions flex items-center gap-2 flex-shrink-0">
            <span className="d2emu-tag">{items.length} 个上架</span>
          </div>
        </header>
        {loading ? (
          <div style={{ display: "flex", alignItems: "center", justifyContent: "center", minHeight: 240 }}><D2EmuLoading text="加载中…" /></div>
        ) : !items.length ? (
          <EmptyState
            icon="tag"
            title="暂无活跃上架"
            hint="从共享仓库选中物品后点击上架按钮,物品会出现在这里。"
            compact
          />
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
            {items.map(item => {
              const lineTotal = item.quantity * item.unit_price
              return (
                <div key={item.id} className="d2emu-card-quiet"
                  style={{
                    padding: 14,
                    borderLeft: `3px solid ${GOLD}`,
                  }}>
                  <div className="flex justify-between items-start mb-2" style={{ gap: 8 }}>
                    <div className="min-w-0">
                      <div className="font-semibold truncate" style={{ color: GOLD, fontSize: 14 }}>
                        {item.name}
                      </div>
                      <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted, #888)', textTransform: 'uppercase', letterSpacing: '0.05em', marginTop: 2 }}>
                        已上架到市场
                      </div>
                    </div>
                    <span className="d2emu-tag d2emu-tag-active" style={{ flexShrink: 0 }}>出售中</span>
                  </div>

                  <div className="space-y-1 text-sm"
                    style={{ borderTop: '1px solid var(--color-d2emu-line, #252525)', paddingTop: 10, marginTop: 10 }}>
                    <div className="flex justify-between">
                      <span style={{ color: 'var(--color-d2emu-muted, #888)' }}>数量</span>
                      <strong>{item.quantity}</strong>
                    </div>
                    <div className="flex justify-between">
                      <span style={{ color: 'var(--color-d2emu-muted, #888)' }}>单价</span>
                      <strong>{fmt(item.unit_price)} 代币</strong>
                    </div>
                    <div className="flex justify-between"
                      style={{ borderTop: '1px solid var(--color-d2emu-line, #252525)', paddingTop: 6, marginTop: 6 }}>
                      <span style={{ color: 'var(--color-d2emu-muted, #888)' }}>总价</span>
                      <strong style={{ color: GOLD }}>{fmt(lineTotal)} 代币</strong>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 sm:flex sm:gap-1.5" style={{ marginTop: 12, gap: 4 }}>
                    <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
                      style={{ flex: 1, padding: '4px 6px' }}
                      onClick={() => viewHistory(item)}
                      title="查看该上架的成交记录">
                      <i className="fa-solid fa-clock-rotate-left" /> 成交记录
                    </button>
                    <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
                      style={{ flex: 1, padding: '4px 6px', color: 'var(--color-d2emu-gold-bright)' }}
                      onClick={() => openReprice(item)}
                      title="修改单价 (无需取消重上架)">
                      <i className="fa-solid fa-pen-to-square" /> 改价
                    </button>
                    <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
                      style={{ flex: 1, padding: '4px 6px' }}
                      onClick={() => quickSell(item)}
                      title="按市场价折算 token 立即卖出">
                      <i className="fa-solid fa-hand-holding-dollar" /> 卖出
                    </button>
                    <button className="d2emu-btn d2emu-btn-danger d2emu-btn-sm"
                      style={{ padding: '4px 8px' }}
                      onClick={() => cancel(item.id)}
                      title="取消上架, 物品归还仓库">
                      <i className="fa-solid fa-circle-xmark" />
                    </button>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </section>

      {/* 改价 Modal */}
      {repriceItem && (
        <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={() => setRepriceItem(null)}>
          <div className="absolute inset-0" style={{ background: 'rgba(0,0,0,0.75)' }} />
          <div ref={repriceModalRef} className="relative d2emu-card d2emu-modal"
            onClick={e => e.stopPropagation()}>
            <div style={{ borderBottom: '1px solid var(--color-d2emu-line)', paddingBottom: 12, marginBottom: 14 }}>
              <p className="d2emu-kicker">修改单价</p>
              <h3 className="font-d2emu-title" style={{
                textAlign: 'left', padding: 0,
                fontSize: 22, letterSpacing: 2, margin: '4px 0 0',
                color: 'var(--color-d2emu-gold-bright, #fff)',
              }}>修改单价</h3>
            </div>

            <div style={{
              padding: '10px 12px', marginBottom: 14,
              background: 'rgba(0,0,0,0.4)',
              border: '1px solid var(--color-d2emu-line)',
              borderRadius: 4, fontSize: 14,
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
                <span style={{ color: 'var(--color-d2emu-muted)' }}>物品</span>
                <strong style={{ color: 'var(--color-d2emu-gold-bright)' }}>{repriceItem.name}</strong>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
                <span style={{ color: 'var(--color-d2emu-muted)' }}>数量</span>
                <strong>×{repriceItem.quantity}</strong>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--color-d2emu-muted)' }}>当前单价</span>
                <strong style={{ fontFamily: 'monospace', color: 'var(--color-d2emu-text)' }}>
                  {fmt(repriceItem.unit_price)} 代币
                </strong>
              </div>
            </div>

            <div className="d2emu-field" style={{ marginBottom: 12 }}>
              <label>新单价 (代币 / 个)</label>
              <div className="d2emu-stepper" role="group" aria-label="新价格调节">
                <button type="button"
                  onClick={() => setNewPrice(p => Math.max(1, p - Math.max(1, Math.floor(p / 10))))}
                  aria-label="降低价格">−</button>
                <input id="reprice-input" type="number" min={1} value={newPrice}
                  role="spinbutton" aria-valuemin={1} aria-valuenow={newPrice}
                  onChange={e => setNewPrice(Math.max(1, Number(e.target.value) || 1))}
                  autoFocus />
                <button type="button"
                  onClick={() => setNewPrice(p => p + Math.max(1, Math.floor(p / 10)))}
                  aria-label="增加价格">+</button>
            </div>
            </div>

            {/* 预览对比 */}
            {newPrice !== repriceItem.unit_price && (
              <div style={{
                display: 'grid', gridTemplateColumns: '1fr 1fr',
                gap: 10, marginBottom: 14,
                padding: 10,
                background: 'rgba(0,0,0,0.3)',
                border: '1px solid var(--color-d2emu-line)',
                borderRadius: 4,
                fontSize: 14,
              }}>
                <div>
                  <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 2 }}>旧总价</div>
                  <div style={{ font: '600 16px/1 Roboto Mono, monospace', color: 'var(--color-d2emu-muted)', textDecoration: 'line-through' }}>
                    {fmt(repriceItem.unit_price * repriceItem.quantity)}
                  </div>
                </div>
                <div style={{ textAlign: 'right' }}>
                  <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 2 }}>新总价</div>
                  <div style={{
                    font: '700 18px/1 Cinzel, serif',
                    color: newPrice > repriceItem.unit_price ? 'var(--color-d2emu-good, #4caf50)' : 'var(--color-d2emu-bad, #ef5350)',
                    letterSpacing: 1,
                  }}>
                    {fmt(newPrice * repriceItem.quantity)}
                  </div>
                </div>
              </div>
            )}

            <div style={{ display: 'flex', gap: 10 }}>
              <button className="d2emu-btn d2emu-btn-action" style={{ flex: 1 }}
                onClick={submitReprice}>
                <i className="fa-solid fa-check" /> 确认改价
              </button>
              <button className="d2emu-btn d2emu-btn-ghost" onClick={() => setRepriceItem(null)}>
                取消
              </button>
            </div>
          </div>
        </div>
      )}

      {confirmDeleteId && (
        <D2ConfirmModal
          title="取消上架"
          danger
          confirmText="确认取消"
          onConfirm={() => executeCancel(confirmDeleteId)}
          onClose={() => setConfirmDeleteId(null)}>
          <p>确定取消此上架？物品将归还到共享仓库。</p>
          <p style={{ marginTop: 8, color: 'var(--color-d2emu-bad, #ef5350)', fontWeight: 600 }}>
            <i className="fa-solid fa-circle-info" /> 此操作不可撤销。
          </p>
        </D2ConfirmModal>
      )}

      {confirmQuickSellItem && (
        <D2ConfirmModal
          title="快速卖出"
          danger
          confirmText="确认卖出"
          onConfirm={() => executeQuickSell(confirmQuickSellItem)}
          onClose={() => setConfirmQuickSellItem(null)}>
          <p>立即卖出 <strong>{confirmQuickSellItem.name}</strong> ×{confirmQuickSellItem.quantity}？</p>
          <p style={{ marginTop: 4 }}>
            将按市场价折算成代币（通常低于上架价 20-40%）。
          </p>
          <p style={{ marginTop: 8, color: 'var(--color-d2emu-bad, #ef5350)', fontWeight: 600 }}>
            <i className="fa-solid fa-circle-info" /> 此操作不可撤销。
          </p>
        </D2ConfirmModal>
      )}
    </div>
  )
}