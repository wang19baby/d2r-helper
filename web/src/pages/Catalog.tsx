import { useEffect, useState, useMemo, useDeferredValue } from 'react'
import { tauriInvoke, fmt } from '../tauri'
import { showToast } from '../components/Toast'
import D2EmuCard from '../components/D2EmuCard'
import D2EmuLoading from '../components/D2EmuLoading'
import KpiRow from '../components/KpiRow'
import QualityLegend, { type QualityKey } from '../components/QualityLegend'
import EmptyState from '../components/EmptyState'
import BuyModal from '../components/BuyModal'
import { QUALITY_COLOR } from '../components/QualityLegend'
import type { ListedItem } from '../types'

const KIND_LABEL: Record<string, string> = {
  rune: '符文', gem: '宝石', potion: '药水', key: '钥匙',
  essence: '精华', shard: '碎片', charm: '护符', jewelry: '珠宝',
  armor: '护甲', weapon: '武器', shield: '盾牌', misc: '杂项',
}

const KIND_ICON: Record<string, string> = {
  rune: 'fa-diamond', gem: 'fa-gem', potion: 'fa-flask', key: 'fa-key',
  essence: 'fa-droplet', shard: 'fa-bolt', charm: 'fa-star', jewelry: 'fa-ring',
  armor: 'fa-shield-halved', weapon: 'fa-crosshairs', shield: 'fa-shield', misc: 'fa-cube',
}

const KIND_TABS = [
  { key: 'all', label: '全部' },
  { key: 'rune', label: '符文' },
  { key: 'gem', label: '宝石' },
  { key: 'potion', label: '药水' },
  { key: 'key', label: '钥匙' },
  { key: 'essence', label: '精华' },
] as const

const QUALITY_CHIPS: { key: QualityKey; label: string }[] = [
  { key: 'unique', label: '暗金' },
  { key: 'set', label: '绿色' },
  { key: 'rare', label: '稀有' },
  { key: 'magic', label: '魔法' },
  { key: 'normal', label: '普通' },
]

type SortKey = 'tier' | 'price-asc' | 'price-desc' | 'time-newest' | 'time-oldest'
const SORT_OPTIONS: { key: SortKey; label: string; icon: string }[] = [
  { key: 'tier',        label: '符文等级', icon: 'fa-layer-group' },
  { key: 'price-asc',   label: '价格 ↑',   icon: 'fa-arrow-up-wide-short' },
  { key: 'price-desc',  label: '价格 ↓',   icon: 'fa-arrow-down-wide-short' },
  { key: 'time-newest', label: '最新上架', icon: 'fa-clock' },
  { key: 'time-oldest', label: '最早上架', icon: 'fa-clock-rotate-left' },
]

/** rune code "r01"→1、"r33"→33;非 rune 返回 null */
function runeTierFromCode(code: string | null | undefined): number | null {
  if (!code) return null
  const m = code.toLowerCase().match(/^r(\d{1,2})$/)
  if (!m) return null
  const n = parseInt(m[1], 10)
  return n >= 1 && n <= 33 ? n : null
}

export default function Catalog() {
  const [items, setItems] = useState<ListedItem[]>([])
  const [balance, setBalance] = useState<number | null>(null)
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')
  const [kindTab, setKindTab] = useState<string>('all')
  const [qualityChip, setQualityChip] = useState<QualityKey | null>(null)
  const [sortBy, setSortBy] = useState<SortKey>('tier')
  const [expiringOnly, setExpiringOnly] = useState(false)
  const [priceMax, setPriceMax] = useState<number | null>(null) // null = 无限
  const deferredPriceMax = useDeferredValue(priceMax)
  const [buyItem, setBuyItem] = useState<ListedItem | null>(null)

  const load = async () => {
    setLoading(true)
    try {
      const [listed, bal] = await Promise.all([
        tauriInvoke('get_listed_items') as Promise<ListedItem[]>,
        tauriInvoke('get_balance') as Promise<number>,
      ])
      setItems(listed || [])
      setBalance(bal)
    } catch (e: any) {
      showToast(e?.message || '加载失败', 'error', { position: 'top' })
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  // 监听顶栏 balance 刷新
  useEffect(() => {
    const h = (e: CustomEvent) => setBalance(e.detail)
    window.addEventListener('balance-update', h as EventListener)
    return () => window.removeEventListener('balance-update', h as EventListener)
  }, [])

  // 计算 KPI
  const stats = useMemo(() => {
    const total = items.length
    const totalQty = items.reduce((s, i) => s + i.quantity, 0)
    const totalValue = items.reduce((s, i) => s + i.quantity * i.unit_price, 0)
    const avg = total > 0 ? Math.round(totalValue / totalQty) : 0
    return { total, totalQty, totalValue, avg }
  }, [items])

  // 品类计数
  const kindCounts = useMemo(() => {
    const m: Record<string, number> = { all: items.length }
    for (const i of items) {
      const k = i.item_kind || 'misc'
      m[k] = (m[k] || 0) + 1
    }
    return m
  }, [items])

  // 价格区间上限(用于 slider,基于当前可见物品的最大单价)
  const priceCeiling = useMemo(() => {
    if (!items.length) return 100000
    return Math.max(1000, Math.ceil(Math.max(...items.map(i => i.unit_price)) / 1000) * 1000)
  }, [items])

  // 即将过期数量(sell_after_seconds < 3600s 且 status === 'listed')
  const expiringCount = useMemo(() => items.filter(i =>
    (i.status === 'listed' || i.status == null) && i.sell_after_seconds > 0 && i.sell_after_seconds < 3600
  ).length, [items])

  // 过滤 + 排序
  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase()
    const list = items.filter(i => {
      if (kindTab !== 'all' && (i.item_kind || 'misc') !== kindTab) return false
      if (qualityChip && (i.quality || 'normal') !== qualityChip) return false
      if (expiringOnly && !(i.sell_after_seconds > 0 && i.sell_after_seconds < 3600)) return false
      if (deferredPriceMax != null && i.unit_price > deferredPriceMax) return false
      if (q) {
        const name = i.name.toLowerCase()
        const code = (i.item_code || '').toLowerCase()
        if (!name.includes(q) && !code.includes(q) && !(i.listed_by || '').toLowerCase().includes(q)) return false
      }
      return true
    })

    list.sort((a, b) => {
      switch (sortBy) {
        case 'price-asc':  return a.unit_price - b.unit_price
        case 'price-desc': return b.unit_price - a.unit_price
        case 'time-newest': {
          const at = a.listed_at ? Date.parse(a.listed_at) : 0
          const bt = b.listed_at ? Date.parse(b.listed_at) : 0
          return bt - at
        }
        case 'time-oldest': {
          const at = a.listed_at ? Date.parse(a.listed_at) : 0
          const bt = b.listed_at ? Date.parse(b.listed_at) : 0
          return at - bt
        }
        case 'tier':
        default: {
          const at = runeTierFromCode(a.item_code) ?? 999
          const bt = runeTierFromCode(b.item_code) ?? 999
          if (at !== bt) return bt - at
          return a.unit_price - b.unit_price
        }
      }
    })
    return list
  }, [items, search, kindTab, qualityChip, sortBy, expiringOnly, deferredPriceMax])

  const onBuyClick = (item: ListedItem) => {
    if ((item.item_kind || 'misc') !== 'rune') {
      showToast(`${item.name} 暂不支持直接购买,目前仅支持符文。`, 'warning', { position: 'top' })
      return
    }
    setBuyItem(item)
  }

  return (
    <div className="font-d2emu-ui flex flex-col" style={{ gap: 12 }}>

      {/* Hero 段 */}
      <section className="d2emu-card">
        <div className="flex items-start gap-4 flex-wrap">
          <img className="d2emu-portrait" alt="market"
            src="data:image/svg+xml;utf8,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 64 64%27%3E%3Crect width=%2764%27 height=%2764%27 fill=%27%23100a05%27/%3E%3Cpath d=%27M32 8l4 14h14l-11 8 4 14-11-8-11 8 4-14-11-8h14z%27 fill=%27%23FBB13A%27 opacity=%270.85%27/%3E%3C/svg%3E" />
          <div className="flex-1 min-w-0">
            <p className="d2emu-kicker">物品市场</p>
            <h1 className="font-d2emu-title" style={{ textAlign: 'left', padding: 0 }}>市场</h1>
            <p className="d2emu-lede" style={{ marginTop: 6 }}>
              浏览其他玩家在售的物品,选中后用代币购买;购买后物品直接写入你的共享仓库。
            </p>
            <div className="d2emu-tags">
              <span className="d2emu-tag">{stats.total} 个上架</span>
              <span className="d2emu-tag">{fmt(stats.totalValue)} 代币总价值</span>
              <span className="d2emu-tag d2emu-tag-active">{fmt(balance ?? 0)} 代币余额</span>
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
          { label: '在售物品', value: stats.total, delta: `${stats.totalQty} 件总数`, trend: 'neutral' },
          { label: '总价值', value: fmt(stats.totalValue), delta: '代币', trend: 'neutral', gold: true },
          { label: '平均单价', value: fmt(stats.avg), delta: '代币 / 件', trend: 'neutral' },
          { label: '我的余额', value: fmt(balance ?? 0), delta: '代币', trend: balance && balance > 0 ? 'up' : 'neutral' },
        ]}
      />

      {/* Quality 图例 + 搜索 + 类别 tab */}
      <D2EmuCard
        kicker="Browse"
        title="在售物品"
        actions={
          <div className="d2emu-field" style={{ minWidth: 220 }}>
            <input type="text" placeholder="搜索 名称 / code / 上架人"
              value={search} onChange={e => setSearch(e.target.value)} />
          </div>
        }
      >
        {/* 类别 sub-tab */}
        <div className="d2emu-tags" style={{ marginBottom: 10 }}>
          {KIND_TABS.map(t => (
            <button key={t.key}
              onClick={() => setKindTab(t.key)}
              className={`d2emu-tag ${kindTab === t.key ? 'd2emu-tag-active' : ''}`}
              style={{ cursor: 'pointer', borderStyle: 'solid' }}>
              {t.label}
              <span style={{ marginLeft: 4, color: 'var(--color-d2emu-muted)' }}>
                ({kindCounts[t.key] || 0})
              </span>
            </button>
          ))}
        </div>

        {/* Quality 过滤 chip */}
        <QualityLegend
          items={QUALITY_CHIPS.map(c => c.key)}
          mode="compact"
        />
        <div style={{ marginTop: 8 }}>
          {QUALITY_CHIPS.map(c => (
            <button key={c.key}
              onClick={() => setQualityChip(qualityChip === c.key ? null : c.key)}
              className={`d2emu-tag ${qualityChip === c.key ? 'd2emu-tag-active' : ''}`}
              style={{
                marginRight: 6,
                cursor: 'pointer',
                borderLeft: `3px solid ${QUALITY_COLOR[c.key]}`,
                borderStyle: 'solid',
                borderWidth: '0 0 0 3px',
              }}>
              {c.label}
            </button>
          ))}
          {qualityChip && (
            <button onClick={() => setQualityChip(null)}
              className="d2emu-tag" style={{ cursor: 'pointer', color: 'var(--color-d2emu-bad)' }}>
              <i className="fa-solid fa-xmark" /> 清除
            </button>
          )}
        </div>

        {/* 排序 + 价格 + 即将过期 */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap',
          padding: '10px 12px',
          background: 'rgba(0,0,0,0.3)',
          border: '1px solid var(--color-d2emu-line)',
          borderRadius: 4,
        }}>
          {/* 排序 */}
          <div className="d2emu-field" style={{ minWidth: 160, flex: '0 0 auto' }}>
            <label><i className="fa-solid fa-sort" /> 排序</label>
            <select value={sortBy} onChange={e => setSortBy(e.target.value as SortKey)}>
              {SORT_OPTIONS.map(o => (
                <option key={o.key} value={o.key}>{o.label}</option>
              ))}
            </select>
          </div>

          {/* 价格上限 slider */}
          <div className="d2emu-field" style={{ minWidth: 220, flex: '1 1 220px' }}>
            <label>
              <i className="fa-solid fa-coins" />
              价格上限
              {priceMax != null && (
                <span style={{ marginLeft: 8, color: 'var(--color-d2emu-gold-bright)' }}>
                  ≤ {fmt(priceMax)} 代币
                </span>
              )}
            </label>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <input
                type="range"
                min={0}
                max={priceCeiling}
                step={(() => {
                  const c = priceCeiling
                  if (c <= 500) return 5
                  if (c <= 2000) return 10
                  if (c <= 10000) return 50
                  if (c <= 50000) return 100
                  return 500
                })()}
                value={priceMax ?? priceCeiling}
              />
              {priceMax != null && (
                <button onClick={() => setPriceMax(null)}
                  className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
                  style={{ padding: '2px 8px', fontSize: 14 }}
                  title="清除价格限制">
                  <i className="fa-solid fa-xmark" />
                </button>
              )}
            </div>
          </div>

          {/* 即将过期 toggle */}
          <button
            onClick={() => setExpiringOnly(!expiringOnly)}
            className={`d2emu-btn ${expiringOnly ? 'd2emu-btn-action' : 'd2emu-btn-ghost'} d2emu-btn-sm`}
            title="1 小时内即将过期的上架"
            style={{ flexShrink: 0 }}>
            <i className="fa-regular fa-clock" />
            即将过期
            {expiringCount > 0 && (
              <span style={{
                marginLeft: 6,
                padding: '1px 6px',
                background: expiringOnly ? 'rgba(255,255,255,0.25)' : 'rgba(251,177,58,0.2)',
                color: expiringOnly ? '#fff' : 'var(--color-d2emu-gold-bright)',
                borderRadius: 999, fontSize: 14, fontWeight: 700,
                fontFamily: '"Roboto Mono", monospace',
              }}>{expiringCount}</span>
            )}
          </button>
        </div>
      </D2EmuCard>

      {/* 物品 grid */}
      <D2EmuCard
        kicker="Listings"
        title={loading ? '加载中…' : `共 ${filtered.length} 件`}
        actions={
          <span className="d2emu-tag" style={{ fontSize: 14 }}>
            <i className="fa-solid fa-clock-rotate-left" />&nbsp;
            实时数据
          </span>
        }
      >
        {loading ? (
          <div style={{ display: "flex", alignItems: "center", justifyContent: "center", minHeight: 240 }}><D2EmuLoading text="Loading marketplace" /></div>
        ) : items.length === 0 ? (
          <EmptyState
            icon="store"
            title="市场暂无在售物品"
            hint="去共享仓库选一个物品上架,或者等待其他玩家挂出商品。"
          />
        ) : filtered.length === 0 ? (
          <EmptyState
            icon="magnifying-glass"
            title="没有匹配的物品"
            hint="试试清除筛选条件,或换个关键词。"
            compact
          />
        ) : (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
            {filtered.map(item => {
              const q = (item.quality as QualityKey) || 'normal'
              const qc = QUALITY_COLOR[q] || QUALITY_COLOR.normal
              const tier = runeTierFromCode(item.item_code)
              const kindLabel = KIND_LABEL[item.item_kind || 'misc'] || item.item_kind || '其他'
              const listedBy = (item.listed_by || '').replace(/\.d2s$/i, '')
              const isRune = (item.item_kind || '') === 'rune'
              const affordable = balance != null && balance >= item.unit_price
              return (
                <article key={item.id}
                  className="d2emu-card-quiet group flex flex-col d2emu-card-hover-item"
                  style={{
                    padding: 12, position: 'relative',
                    '--d2emu-card-qc-border': `${qc}55`,
                    '--d2emu-card-qc-hover-border': qc,
                    '--d2emu-card-qc-hover-shadow': `0 4px 18px ${qc}33`,
                  } as any}>
                  {/* 品质条 */}
                  <div style={{
                    position: 'absolute', top: 0, left: 0, right: 0, height: 2,
                    background: qc,
                    opacity: 0.8,
                  }} />

                  {/* 头部: icon + tier/badge */}
                  <div style={{
                    display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10,
                  }}>
                    <div style={{
                      width: 44, height: 44, flexShrink: 0,
                      background: '#0a0806', border: `1px solid ${qc}66`,
                      display: 'grid', placeItems: 'center', borderRadius: 3,
                      boxShadow: `inset 0 0 10px ${qc}22`,
                    }}>
                      <i className={`fa-solid ${KIND_ICON[item.item_kind || 'misc']}`}
                        style={{ color: qc, fontSize: 22, filter: `drop-shadow(0 0 6px ${qc}66)` }} />
                    </div>
                    <div className="min-w-0 flex-1">
                      <div style={{
                        fontSize: 14, fontWeight: 700,
                        color: qc,
                        textShadow: `0 0 8px ${qc}44`,
                        whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
                      }}>{item.name}</div>
                      <div style={{
                        fontSize: 14, color: 'var(--color-d2emu-muted)',
                        textTransform: 'uppercase', letterSpacing: '0.06em', marginTop: 2,
                      }}>
                        {kindLabel}
                        {item.item_code && <> · {item.item_code.toUpperCase()}</>}
                      </div>
                    </div>
                    {tier && (
                      <div style={{
                        font: '700 14px/1 "Roboto Mono", monospace',
                        color: qc, padding: '3px 7px',
                        border: `1px solid ${qc}`, borderRadius: 2,
                        background: `${qc}15`,
                      }}>#{String(tier).padStart(2, '0')}</div>
                    )}
                  </div>

                  {/* 价格 + 数量 */}
                  <div style={{
                    display: 'grid', gridTemplateColumns: '1fr 1fr',
                    gap: 6, marginBottom: 8,
                    padding: '6px 8px',
                    background: 'rgba(0,0,0,0.35)',
                    border: '1px solid var(--color-d2emu-line)',
                    borderRadius: 3,
                  }}>
                    <div>
                      <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', textTransform: 'uppercase', letterSpacing: '0.08em' }}>单价</div>
                      <div style={{
                        font: '600 14px/1 "Roboto Mono", monospace',
                        color: 'var(--color-d2emu-gold-bright)',
                      }}>{fmt(item.unit_price)}</div>
                    </div>
                    <div style={{ textAlign: 'right' }}>
                      <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', textTransform: 'uppercase', letterSpacing: '0.08em' }}>库存</div>
                      <div style={{
                        font: '600 14px/1 "Roboto Mono", monospace',
                        color: 'var(--color-d2emu-text)',
                      }}>×{item.quantity}</div>
                    </div>
                  </div>

                  {/* 上架人 */}
                  {listedBy && (
                    <div style={{
                      fontSize: 14, color: 'var(--color-d2emu-muted)',
                      textTransform: 'uppercase', letterSpacing: '0.05em',
                      marginBottom: 8,
                      whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
                    }}>
                      <i className="fa-solid fa-user-tag" style={{ marginRight: 4 }} />
                      {listedBy}
                    </div>
                  )}

                  {/* 总价 + 购买 */}
                  <div style={{
                    display: 'flex', alignItems: 'center', gap: 8,
                    marginTop: 'auto',
                    paddingTop: 8, borderTop: '1px solid var(--color-d2emu-line)',
                  }}>
                    <div className="min-w-0">
                      <div style={{ fontSize: 14, color: 'var(--color-d2emu-muted)', textTransform: 'uppercase', letterSpacing: '0.08em' }}>总价</div>
                      <div style={{
                        font: '700 16px/1 "Cinzel", serif',
                        color: affordable ? 'var(--color-d2emu-gold-bright)' : 'var(--color-d2emu-bad)',
                        letterSpacing: 1,
                      }}>{fmt(item.unit_price * item.quantity)}</div>
                    </div>
                    <button
                      className={`d2emu-btn ${isRune && affordable ? 'd2emu-btn-action' : 'd2emu-btn-ghost'} d2emu-btn-sm`}
                      style={{ flex: 1, minWidth: 0 }}
                      disabled={!isRune || !affordable}
                      onClick={() => onBuyClick(item)}>
                      <i className={`fa-solid ${isRune ? 'fa-coins' : 'fa-lock'}`} />
                      {!isRune ? '敬请期待' : !affordable ? '余额不足' : '购买'}
                    </button>
                  </div>
                </article>
              )
            })}
          </div>
        )}
      </D2EmuCard>

      {/* Buy Modal */}
      {buyItem && (
        <BuyModal item={buyItem} currentBalance={balance}
          onClose={() => setBuyItem(null)}
          onBought={load} />
      )}
    </div>
  )
}