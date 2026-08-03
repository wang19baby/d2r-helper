import { useEffect, useState } from 'react'
import { tauriInvoke } from '../tauri'
import { showToast } from '../components/Toast'
import D2EmuCard from '../components/D2EmuCard'
import D2EmuLoading from '../components/D2EmuLoading'
import EmptyState from '../components/EmptyState'

const GOLD = 'var(--color-d2emu-gold)'
const TEXT = 'var(--color-d2emu-text)'
const MUTED = 'var(--color-d2emu-muted)'
const LINE = 'var(--color-d2emu-line)'

const QUALITY_COLOR: Record<string, string> = {
  unique: '#c7b377', set: '#2c8c3a',
}

type TabFilter = 'all' | 'unique' | 'set'

export default function Grail() {
  const [data, setData] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [filter, setFilter] = useState<TabFilter>('all')
  const [searchTerm, setSearchTerm] = useState('')

  const load = async () => {
    setLoading(true)
    try {
      const res = await tauriInvoke('get_grail') as any
      setData(res)
    } catch (e: any) {
      showToast(String(e), 'error')
    }
    setLoading(false)
  }

  useEffect(() => { load() }, [])

  const toggle = async (key: string, currentlyFound: boolean) => {
    try {
      await tauriInvoke('toggle_grail', { itemKey: key, found: !currentlyFound })
      load()
    } catch (e: any) {
      showToast(String(e), 'error')
    }
  }

  const filtered = data?.items?.filter((item: any) => {
    if (filter !== 'all' && item.item_type !== filter) return false
    if (searchTerm) {
      const q = searchTerm.toLowerCase()
      return item.name_en.toLowerCase().includes(q) || item.name_zh?.toLowerCase().includes(q) || item.item_code?.toLowerCase().includes(q)
    }
    return true
  })?.sort((a: any, b: any) => a.found - b.found || (a.level || 0) - (b.level || 0)) ?? []

  return (
    <div className="font-d2emu-ui flex flex-col" style={{ gap: 12 }}>
      <D2EmuCard kicker="圣杯追踪" title="圣杯追踪"
        lede="记录已获得的暗金和套装物品，追踪你的收集进度。"
        actions={<i className="fa-solid fa-trophy" style={{ fontSize: 22, color: GOLD, opacity: 0.7 }} />}
      >
        {/* 进度条 */}
        {data && (
          <div style={{ marginBottom: 16 }}>
            <div className="flex justify-between items-center" style={{ marginBottom: 6 }}>
              <span style={{ fontSize: 14, color: TEXT, fontWeight: 600 }}>
                收集进度
              </span>
              <span style={{ fontSize: 14, color: MUTED }}>
                {data.found} / {data.total} ({data.pct.toFixed(1)}%)
              </span>
            </div>
            <div style={{
              height: 20, borderRadius: 10, overflow: 'hidden',
              background: '#0a0806', border: `1px solid ${LINE}`,
            }}>
              <div style={{
                width: `${data.pct}%`, height: '100%',
                background: 'linear-gradient(90deg, #c7b377, #2c8c3a)',
                transition: 'width 300ms ease',
                borderRadius: 10,
              }} />
            </div>
          </div>
        )}

        {/* 筛选 */}
        <div className="flex gap-2 items-center flex-wrap" style={{ marginBottom: 12 }}>
          {(['all', 'unique', 'set'] as TabFilter[]).map(t => (
            <button key={t} onClick={() => setFilter(t)}
              className="d2emu-btn d2emu-btn-sm"
              style={{
                background: filter === t ? GOLD + '22' : 'transparent',
                border: `1px solid ${filter === t ? GOLD : LINE}`,
                color: filter === t ? GOLD : MUTED,
              }}>
              {t === 'all' ? '全部' : t === 'unique' ? '暗金' : '套装'}
              {data && ` (${data.items.filter((i: any) => (t === 'all' || i.item_type === t) && (!searchTerm || i.name_en.toLowerCase().includes(searchTerm.toLowerCase()) || i.name_zh?.toLowerCase().includes(searchTerm.toLowerCase()) || i.item_code?.toLowerCase().includes(searchTerm.toLowerCase()))).length})`}
            </button>
          ))}
          <div className="d2emu-field" style={{ minWidth: 160, flex: 1 }}>
            <input type="text" placeholder="搜索物品名或代码..."
              value={searchTerm} onChange={e => setSearchTerm(e.target.value)} />
          </div>
        </div>
      </D2EmuCard>

      {/* 列表 */}
      {loading ? <div style={{ display: "flex", alignItems: "center", justifyContent: "center", minHeight: 240 }}><D2EmuLoading /></div> : filtered.length === 0 ? (
        <EmptyState icon="trophy" title="没有匹配的物品" hint="调整筛选条件试试" />
      ) : (
        <div className="grid gap-1.5" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))' }}>
          {filtered.map((item: any) => {
            const isUnique = item.item_type === 'unique'
            const c = isUnique ? QUALITY_COLOR.unique : QUALITY_COLOR.set
            return (
              <button key={item.item_key} onClick={() => toggle(item.item_key, item.found)}
                className="transition-all duration-75"
                style={{
                  display: 'flex', alignItems: 'center', gap: 10, cursor: 'pointer',
                  padding: '8px 12', borderRadius: 6, textAlign: 'left',
                  background: item.found ? `${c}10` : '#0a0806',
                  border: `1px solid ${item.found ? c + '44' : LINE}`,
                  opacity: item.found ? 0.8 : 1,
                }}>
                {/* Check indicator */}
                <div style={{
                  width: 22, height: 22, borderRadius: 4, flexShrink: 0,
                  display: 'grid', placeItems: 'center',
                  background: item.found ? c : 'transparent',
                  border: `2px solid ${item.found ? c : MUTED}`,
                  color: item.found ? '#000' : 'transparent',
                  fontSize: 14, fontWeight: 700,
                }}>
                  {item.found ? '✓' : ''}
                </div>
                {/* Info */}
                <div className="min-w-0 flex-1">
                  <div style={{ color: c, fontWeight: 600, fontSize: 14, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {item.name_en}
                  </div>
                  <div style={{ fontSize: 14, color: MUTED }}>
                    {item.item_code && <span style={{ fontFamily: 'monospace' }}>{item.item_code}</span>}
                    {item.level > 0 && <span> · lv{item.level}</span>}
                    {item.found_at && <span> · {new Date(item.found_at).toLocaleDateString('zh-CN')}</span>}
                  </div>
                </div>
              </button>
            )
          })}
        </div>
      )}
    </div>
  )
}
