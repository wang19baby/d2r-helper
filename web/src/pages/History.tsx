import { useEffect, useState, useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'
import { tauriInvoke, fmt } from '../tauri'
import { showToast } from '../components/Toast'
import D2EmuCard from '../components/D2EmuCard'
import D2EmuLoading from '../components/D2EmuLoading'
import KpiRow from '../components/KpiRow'
import EmptyState from '../components/EmptyState'
import type { HistoryEntry } from '../types'
import { useLocale } from '../locales/context'

/**
 * 交易类型 → (中文标签, FA 图标, 颜色)
 *
 * 与 marketplace.rs / warehouse.rs 中的 add_transaction 类型保持一致:
 *   buy_import | list | sell | export | import | cancel | warehouse_*
 */
const TX_META: Record<string, { label: string; icon: string; color: string }> = {
  buy_import:        { label: 'history.tx_buy',  icon: 'fa-bag-shopping',     color: 'var(--color-d2emu-bad, #ef5350)' },
  list:              { label: 'history.tx_list',  icon: 'fa-tag',              color: 'var(--color-d2emu-gold-bright)' },
  sell:              { label: 'history.tx_sell',  icon: 'fa-coins',            color: 'var(--color-d2emu-good, #4caf50)' },
  export:            { label: 'history.tx_export', icon: 'fa-arrow-up-from-bracket', color: 'var(--color-d2emu-blue)' },
  import:            { label: 'history.tx_import', icon: 'fa-arrow-down-to-bracket', color: 'var(--color-d2emu-blue)' },
  warehouse_deposit: { label: 'history.tx_deposit',  icon: 'fa-box-archive',      color: 'var(--color-d2emu-gold-bright)' },
  warehouse_withdraw:{ label: 'history.tx_withdraw',  icon: 'fa-box-open',         color: 'var(--color-d2emu-good)' },
  warehouse_remove:  { label: 'history.tx_remove',  icon: 'fa-trash',            color: 'var(--color-d2emu-bad)' },
}
const DEFAULT_META = { label: 'history.tx_other', icon: 'fa-circle-info', color: 'var(--color-d2emu-muted)' }

import { fmtTime } from '../utils/time'



export default function History() {
  const [entries, setEntries] = useState<HistoryEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [typeFilter, setTypeFilter] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [pendingItemId, setPendingItemId] = useState<string | null>(null)
  const [focusedItemName, setFocusedItemName] = useState<string | null>(null)
  const [searchParams] = useSearchParams()
  const { t } = useLocale()
  const txMeta = (type: string) => {
    const m = TX_META[type] || DEFAULT_META
    return { label: t(m.label), icon: m.icon, color: m.color }
  }

  // 从 URL search params 读取 Listings 跳转过来的 item 过滤
  useEffect(() => {
    const itemId = searchParams.get('itemId')
    const itemName = searchParams.get('itemName')
    if (itemId) {
      setPendingItemId(itemId)
      setFocusedItemName(itemName)
      setSearch(itemId)
    }
  }, [searchParams])

  const load = async () => {
    setLoading(true)
    try {
      const args: { limit: number; txType?: string } = { limit: 500 }
      if (typeFilter) args.txType = typeFilter
      const res = await tauriInvoke('get_transactions', args) as HistoryEntry[]
      setEntries(res || [])
    } catch (e: any) {
      showToast(e?.message || t('history.load_fail'), 'error', { position: 'top' })
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [typeFilter])

  // 切换 typeFilter 时清除 pendingItemId 以免覆盖筛选
  useEffect(() => {
    if (pendingItemId) {
      setPendingItemId(null)
      setFocusedItemName(null)
    }
  }, [typeFilter])

  // 统计 KPI(基于当前可见 entries)
  const stats = useMemo(() => {
    let income = 0  // 收入(卖出 / 取回 等正数)
    let spend = 0   // 支出(购买 等负数)
    for (const e of entries) {
      if (e.token_amount > 0) income += e.token_amount
      else if (e.token_amount < 0) spend += Math.abs(e.token_amount)
    }
    return {
      count: entries.length,
      income,
      spend,
      net: income - spend,
    }
  }, [entries])

  // 过滤后的列表(本地 search 过滤 + pendingItemId 强匹配)
  const filtered = useMemo(() => {
    let list = entries
    // pendingItemId 优先(从 Listings 跳转过来)
    if (pendingItemId) {
      list = list.filter(e => e.item_id === pendingItemId)
      return list
    }
    const q = search.trim().toLowerCase()
    if (!q) return list
    return list.filter(e => {
      const desc = (e.description || '').toLowerCase()
      const type = (e.tx_type || '').toLowerCase()
      const id = (e.item_id || '').toLowerCase()
      return desc.includes(q) || type.includes(q) || id.includes(q)
    })
  }, [entries, search, pendingItemId])

  const clearItemFilter = () => {
    setPendingItemId(null)
    setFocusedItemName(null)
    setSearch('')
  }

  // 汇总每种 tx_type 的数量(用于顶部 chip)
  const typeCounts = useMemo(() => {
    const m: Record<string, number> = {}
    for (const e of entries) {
      m[e.tx_type] = (m[e.tx_type] || 0) + 1
    }
    return m
  }, [entries])

  const [csvExporting, setCsvExporting] = useState(false)
  // 导出 CSV
  const exportCsv = () => {
    if (!entries.length) return
    setCsvExporting(true)
    // setTimeout 让 React state 更新渲染 + 避免大数据量时卡 UI
    setTimeout(() => {
      try {
        const header = ['id', 'date', 'tx_type', 'token_amount', 'description']
        const lines = [header.join(',')]
        for (const e of entries) {
          const row = [
            e.id,
            `"${(e.date || '').replace(/"/g, '""')}"`,
            e.tx_type,
            e.token_amount,
            `"${(e.description || '').replace(/"/g, '""')}"`,
          ]
          lines.push(row.join(','))
        }
        const blob = new Blob([lines.join('\n')], { type: 'text/csv;charset=utf-8;' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = `d2r-transactions-${new Date().toISOString().slice(0, 10)}.csv`
        a.click()
        URL.revokeObjectURL(url)
        showToast(`已导出 ${entries.length} 条记录`, 'success', { position: 'top' })
      } finally { setCsvExporting(false) }
    }, 50)
  }

  // 当前选中的 type meta(用于 chip 高亮)
  const selectedMeta = typeFilter ? txMeta(typeFilter) : null

  return (
    <div className="font-d2emu-ui flex flex-col" style={{ gap: 12 }}>

      {/* Hero 段 */}
      <section className="d2emu-card">
        <div className="flex items-start gap-4 flex-wrap">
          <img className="d2emu-portrait" alt="history"
            src="data:image/svg+xml;utf8,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 64 64%27%3E%3Crect width=%2764%27 height=%2764%27 fill=%27%23100a05%27/%3E%3Ccircle cx=%2732%27 cy=%2732%27 r=%2722%27 fill=%27none%27 stroke=%27%23FBB13A%27 stroke-width=%272%27/%3E%3Cpath d=%27M32 18v14l10 6%27 stroke=%27%23FBB13A%27 stroke-width=%272.5%27 fill=%27none%27 stroke-linecap=%27round%27/%3E%3C/svg%3E" />
          <div className="flex-1 min-w-0">
            <p className="d2emu-kicker">{t('history.kicker')}</p>
            <h1 className="font-d2emu-title" style={{ textAlign: 'left', padding: 0 }}>{t('history.title')}</h1>
            <p className="d2emu-lede" style={{ marginTop: 6 }}>
              查看所有买入、上架、卖出、扩展仓操作记录。按类型过滤,统计收入支出,支持导出 CSV。
            </p>
            <div className="d2emu-tags">
              <span className="d2emu-tag">{stats.count} 条记录</span>
              <span className="d2emu-tag" style={{ color: 'var(--color-d2emu-good, #4caf50)' }}>
                {t('history.income')} {fmt(stats.income)}
              </span>
              <span className="d2emu-tag" style={{ color: 'var(--color-d2emu-bad, #ef5350)' }}>
                {t('history.spend')} {fmt(stats.spend)}
              </span>
              <span className={`d2emu-tag ${stats.net >= 0 ? 'd2emu-tag-active' : ''}`}
                style={{ color: stats.net >= 0 ? 'var(--color-d2emu-good)' : 'var(--color-d2emu-bad)' }}>
                {t('history.net')} {stats.net >= 0 ? '+' : ''}{fmt(stats.net)}
              </span>
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={load} disabled={loading}>
              <i className="fa-solid fa-rotate-right" /> 刷新
            </button>
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
              onClick={exportCsv} disabled={!entries.length || csvExporting}>
              <i className={`fa-solid ${csvExporting ? 'fa-spinner fa-spin' : 'fa-file-csv'}`} />
              {csvExporting ? '导出中…' : '导出 CSV'}
            </button>
          </div>
        </div>
      </section>

      {/* KPI 摘要 */}
      <KpiRow
        items={[
          { label: '记录总数', value: stats.count, delta: '近 500 条', trend: 'neutral' },
          { label: t('history.income'), value: fmt(stats.income), delta: '代币', trend: stats.income > 0 ? 'up' : 'neutral', gold: true },
          { label: t('history.spend'), value: fmt(stats.spend),  delta: '代币', trend: stats.spend > 0 ? 'down' : 'neutral' },
          { label: t('history.net'),   value: (stats.net >= 0 ? '+' : '') + fmt(stats.net),
            delta: stats.net >= 0 ? '盈利' : '亏损',
            trend: stats.net >= 0 ? 'up' : 'down',
            gold: true },
        ]}
      />

      {/* 类型过滤 + 搜索 */}
      <D2EmuCard
        kicker="Filter"
        title={typeFilter ? `类型: ${selectedMeta?.label}` : '全部类型'}
        actions={
          <div className="d2emu-field" style={{ minWidth: 200 }}>
            <input type="text" placeholder="搜索 描述 / 类型"
              value={search} onChange={e => setSearch(e.target.value)} />
          </div>
        }
      >
        {/* 类型 chip */}
        {/* 类型 chip + 聚焦横幅 */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {/* 来自 Listings 的 item 聚焦横幅 */}
          {pendingItemId && focusedItemName && (
            <div className="d2emu-lock-banner" style={{ margin: 0 }}>
              <span className="d2emu-lock-banner-icon">
                <i className="fa-solid fa-bullseye" />
              </span>
              <div className="d2emu-lock-banner-text">
                <strong>聚焦于 {focusedItemName}</strong>
                <span>
                  只显示该上架的成交 / 取消 / 状态变更记录。
                  <code style={{ marginLeft: 6, fontSize: 14, color: '#f2b84b', fontFamily: 'monospace' }}>
                    {pendingItemId.slice(0, 12)}…
                  </code>
                </span>
              </div>
              <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
                onClick={clearItemFilter}>
                <i className="fa-solid fa-xmark" /> 清除聚焦
              </button>
            </div>
          )}

        <div className="d2emu-tags" style={{ marginBottom: 10 }}>
          <button onClick={() => setTypeFilter(null)}
            className={`d2emu-tag ${typeFilter === null ? 'd2emu-tag-active' : ''}`}
            style={{ cursor: 'pointer', borderStyle: 'solid' }}>
            <i className="fa-solid fa-layer-group" style={{ marginRight: 4 }} />
            {t('history.filter_all')} ({entries.length})
          </button>
          {Object.entries(typeCounts).sort((a, b) => b[1] - a[1]).map(([t, count]) => {
            const m = txMeta(t)
            const on = typeFilter === t
            return (
              <button key={t}
                onClick={() => setTypeFilter(on ? null : t)}
                className={`d2emu-tag ${on ? 'd2emu-tag-active' : ''}`}
                style={{
                  cursor: 'pointer',
                  borderStyle: 'solid',
                  borderLeft: `3px solid ${m.color}`,
                  borderLeftWidth: 3,
                }}>
                <i className={`fa-solid ${m.icon}`} style={{ marginRight: 4, color: m.color }} />
                {m.label}
                <span style={{ marginLeft: 4, color: 'var(--color-d2emu-muted)' }}>({count})</span>
              </button>
            )
          })}
        </div>
        </div>
      </D2EmuCard>

      {/* 表格 */}
      <D2EmuCard
        kicker="Records"
        title={loading ? '加载中…' : `${filtered.length} 条`}
      >
        {loading ? (
          <div style={{ display: "flex", alignItems: "center", justifyContent: "center", minHeight: 240 }}><D2EmuLoading text="Loading history" /></div>
        ) : entries.length === 0 ? (
          <EmptyState
            icon="clock-rotate-left"
            title={t('history.empty')}
            hint="去市场购买物品或者上架一些物品,交易记录会显示在这里。"
          />
        ) : filtered.length === 0 ? (
          <EmptyState
            icon="magnifying-glass"
            title="没有匹配的记录"
            hint="尝试清空搜索框或选择其他类型。"
            compact
          />
        ) : (
          <div style={{ overflowX: 'auto' }}>
            <table className="d2emu-table d2emu-table-card" style={{ minWidth: 640 }}>
              <thead>
                <tr>
                  <th style={{ width: 140 }}>时间</th>
                  <th style={{ width: 110 }}>类型</th>
                  <th style={{ width: 110, textAlign: 'right' }}>金额</th>
                  <th>描述</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map(e => {
                  const m = txMeta(e.tx_type)
                  const positive = e.token_amount > 0
                  const negative = e.token_amount < 0
                  return (
                    <tr key={e.id}>
                      <td style={{ fontFamily: '"Roboto Mono", monospace', fontSize: 14, color: 'var(--color-d2emu-muted)' }}>
                        {fmtTime(e.date)}
                      </td>
                      <td>
                        <span style={{
                          display: 'inline-flex', alignItems: 'center', gap: 6,
                          padding: '2px 8px',
                          background: 'rgba(0,0,0,0.4)',
                          border: `1px solid ${m.color}55`,
                          borderRadius: 3,
                          fontSize: 14, fontWeight: 600,
                          color: m.color,
                          textTransform: 'uppercase',
                          letterSpacing: '0.04em',
                        }}>
                          <i className={`fa-solid ${m.icon}`} style={{ fontSize: 14 }} />
                          {m.label}
                        </span>
                      </td>
                      <td style={{
                        fontFamily: '"Roboto Mono", monospace',
                        fontWeight: 700,
                        textAlign: 'right',
                        color: positive ? 'var(--color-d2emu-good, #4caf50)'
                             : negative ? 'var(--color-d2emu-bad, #ef5350)'
                             : 'var(--color-d2emu-muted)',
                      }}>
                        {positive ? '+' : ''}{fmt(e.token_amount)}
                      </td>
                      <td style={{ color: 'var(--color-d2emu-text, #e8e8e8)' }}>
                        {e.description || '—'}
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          </div>
        )}
      </D2EmuCard>
    </div>
  )
}