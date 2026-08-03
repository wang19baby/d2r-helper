import { useEffect, useState, useMemo } from 'react'
import { tauriInvoke } from '../tauri'
import { showToast } from '../components/Toast'
import D2EmuLoading from '../components/D2EmuLoading'
import EmptyState from '../components/EmptyState'

interface BuildMatch {
  build_id: string
  class: string
  name: string
  name_zh: string
  score: number
  core_owned: number
  core_total: number
  optional_owned: number
  optional_total: number
  owned_items: { slot: string; name: string; code: string; source: string }[]
  missing_core: { slot: string; name: string; code: string; weight: number }[]
  missing_runewords: { name: string; runes: string[]; missing_runes: string[] }[]
  description: string
}

interface BuildResponse {
  builds: BuildMatch[]
  total_builds: number
  source: string
}

const CLASS_OPTIONS = ['', 'Sorceress', 'Paladin', 'Barbarian', 'Necromancer', 'Amazon', 'Assassin', 'Druid']
const CLASS_LABEL: Record<string, string> = {
  Sorceress: '法师', Paladin: '圣骑士', Barbarian: '野蛮人',
  Necromancer: '死灵', Amazon: '亚马逊', Assassin: '刺客', Druid: '德鲁伊',
}

function scoreColor(s: number): string {
  if (s >= 0.8) return '#4caf50'
  if (s >= 0.5) return '#FBB13A'
  if (s >= 0.2) return '#ef5350'
  return '#888'
}

function BuildCard({ build, expanded, onToggle }: {
  build: BuildMatch; expanded: boolean; onToggle: () => void
}) {
  const pct = Math.round(build.score * 100)
  const sc = scoreColor(build.score)

  return (
    <div className="d2emu-card-quiet" style={{ padding: 0, overflow: 'hidden' }}>
      <button type="button" onClick={onToggle}
        className="w-full text-left"
        style={{ padding: '12px 14px', background: 'none', border: 'none', color: 'inherit', cursor: 'pointer' }}>
        <div className="flex items-center gap-3">
          <div className="flex-1 min-w-0">
            <div style={{ fontWeight: 700, fontSize: 14, color: sc }}>
              {build.name_zh}
              {build.core_total > 0 && (
                <span style={{ fontSize: 14, color: '#888', fontWeight: 400, marginLeft: 8 }}>
                  {build.core_owned}/{build.core_total} 核心
                </span>
              )}
            </div>
            <div style={{ fontSize: 14, color: '#666', marginTop: 2 }}>
              {build.name} · {CLASS_LABEL[build.class] || build.class}
            </div>
          </div>
          <div className="text-right flex-shrink-0" style={{ minWidth: 80 }}>
            <div style={{ fontSize: 22, fontWeight: 700, color: sc, lineHeight: 1 }}>{pct}%</div>
            <div style={{ fontSize: 14, color: '#666' }}>匹配度</div>
          </div>
        </div>
        {/* Progress bar */}
        <div style={{ marginTop: 8, height: 4, background: '#1a1612', borderRadius: 2, overflow: 'hidden' }}>
          <div style={{ width: `${pct}%`, height: '100%', background: sc, borderRadius: 2, transition: 'width 0.3s' }} />
        </div>
      </button>

      {expanded && (
        <div style={{ padding: '0 14px 12px', borderTop: '1px solid #1f1812' }}>
          <p style={{ fontSize: 14, color: '#aa9', margin: '8px 0', lineHeight: 1.5 }}>{build.description}</p>

          {/* Owned items */}
          {build.owned_items.length > 0 && (
            <div style={{ marginBottom: 8 }}>
              <div style={{ fontSize: 14, color: '#4caf50', fontWeight: 600, marginBottom: 4 }}>已拥有</div>
              <div className="flex flex-wrap gap-1">
                {build.owned_items.map(oi => (
                  <span key={oi.name} style={{ fontSize: 14, padding: '2px 6px', background: '#0a1a0a', border: '1px solid #2a5a2a', borderRadius: 2 }}>{oi.name}</span>
                ))}
              </div>
            </div>
          )}

          {/* Missing core items */}
          {build.missing_core.length > 0 && (
            <div style={{ marginBottom: 8 }}>
              <div style={{ fontSize: 14, color: '#ef5350', fontWeight: 600, marginBottom: 4 }}>缺少核心</div>
              <div className="flex flex-wrap gap-1">
                {build.missing_core.map(mi => (
                  <span key={mi.name + '-' + mi.slot} style={{ fontSize: 14, padding: '2px 6px', background: '#1a0a0a', border: '1px solid #5a2a2a', borderRadius: 2 }}>{mi.name} ({mi.slot})</span>
                ))}
              </div>
            </div>
          )}

          {/* Missing runewords */}
          {build.missing_runewords.length > 0 && (
            <div>
              <div style={{ fontSize: 14, color: '#FBB13A', fontWeight: 600, marginBottom: 4 }}>可制作的符文之语</div>
              <div className="flex flex-wrap gap-1">
                {build.missing_runewords.map(rw => (
                  <span key={rw.name} style={{ fontSize: 14, padding: '2px 6px', background: '#1a140a', border: '1px solid #5a4a20', borderRadius: 2 }}>{rw.name} · 缺 {rw.missing_runes.join(', ')}</span>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default function Builds() {
  const [builds, setBuilds] = useState<BuildMatch[]>([])
  const [loading, setLoading] = useState(true)
  const [classFilter, setClassFilter] = useState('')
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [characters, setCharacters] = useState<string[]>([])
  const [selectedChar, setSelectedChar] = useState('')
  const [saveFolder, setSaveFolder] = useState('')

  useEffect(() => {
    const load = async () => {
      try {
        const cfg = await tauriInvoke('get_app_config') as Record<string, unknown>
        const dir = (cfg.save_folder || cfg.default_folder) as string
        if (dir) {
          setSaveFolder(dir)
          const chars = await tauriInvoke('list_characters', { dir }) as string[]
          setCharacters(chars || [])
        }
      } catch { showToast('加载角色列表失败', 'warning') }
    }
    load()
  }, [])

  useEffect(() => {
    const load = async () => {
      setLoading(true)
      try {
        const filters: Record<string, unknown> = {}
        if (classFilter) filters.class_filter = classFilter
        if (selectedChar && saveFolder) {
          filters.character_path = `${saveFolder}\\${selectedChar}.d2s`
        }
        const res = await tauriInvoke('get_build_recommendations', filters) as BuildResponse
        setBuilds(res.builds || [])
      } catch (e) {
        showToast(`加载 Build 推荐失败`, 'error')
        setBuilds([])
      } finally { setLoading(false) }
    }
    load()
  }, [classFilter, selectedChar, saveFolder])

  const topBuild = useMemo(() => builds[0] ?? null, [builds])

  return (
    <div className="font-d2emu-ui flex flex-col" style={{ gap: 12 }}>
      <section className="d2emu-card">
        <div className="flex items-start gap-4 flex-wrap">
          <img className="d2emu-portrait" alt="builds"
            src="data:image/svg+xml;utf8,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 64 64%27%3E%3Crect width=%2764%27 height=%2764%27 fill=%27%23100a05%27/%3E%3Ccircle cx=%2732%27 cy=%2720%27 r=%278%27 fill=%27none%27 stroke=%27%23FBB13A%27 stroke-width=%271.5%27/%3E%3Cpath d=%27M12 50c0-10 8-18 20-18s20 8 20 18%27 fill=%27none%27 stroke=%27%23FBB13A%27 stroke-width=%271.5%27/%3E%3Cpath d=%27M32 32l-6 12h12z%27 fill=%27%23FBB13A%27 opacity=%270.4%27/%3E%3C/svg%3E" />
          <div className="flex-1 min-w-0">
            <p className="d2emu-kicker">Build 推荐</p>
            <h1 className="font-d2emu-title" style={{ textAlign: 'left', padding: 0 }}>Build 推荐</h1>
            <p className="d2emu-lede" style={{ marginTop: 6 }}>
              根据你的仓库装备和符文，自动匹配最佳 Build 方案。
            </p>
            <div className="d2emu-tags">
              <span className="d2emu-tag">{builds.length} 个 Build</span>
              {topBuild && (
                <span className="d2emu-tag-dot" />
              )}
              {topBuild && (
                <span className="d2emu-tag" style={{ color: scoreColor(topBuild.score) }}>
                  推荐: {topBuild.name_zh} ({Math.round(topBuild.score * 100)}%)
                </span>
              )}
            </div>
          </div>
        </div>
      </section>

      {/* Filters */}
      <div className="flex items-center gap-2 flex-wrap">
        <select value={selectedChar} onChange={e => setSelectedChar(e.target.value)}
          className="d2-input" style={{ padding: '4px 8px', fontSize: 14, width: 140 }}>
          <option value="">不指定角色</option>
          {characters.map(c => <option key={c} value={c}>{c}</option>)}
        </select>
        <select value={classFilter} onChange={e => setClassFilter(e.target.value)}
          className="d2-input" style={{ padding: '4px 8px', fontSize: 14, width: 120 }}>
          <option value="">所有职业</option>
          {CLASS_OPTIONS.filter(Boolean).map(cl => (
            <option key={cl} value={cl}>{CLASS_LABEL[cl] || cl}</option>
          ))}
        </select>
        <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={() => {
          setClassFilter('')
          setSelectedChar('')
        }}>
          <i className="fa-solid fa-rotate-left" /> 重置
        </button>
      </div>

      {/* Build list */}
      {loading ? (
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", minHeight: 240 }}><D2EmuLoading text="分析装备中..." /></div>
      ) : builds.length === 0 ? (
        <EmptyState
          icon="flask"
          title="暂无推荐的 Build"
          hint="先存入一些装备到仓库吧。" />
      ) : (
        <div className="space-y-2">
          {builds.map(b => (
            <BuildCard
              key={b.build_id}
              build={b}
              expanded={expandedId === b.build_id}
              onToggle={() => setExpandedId(expandedId === b.build_id ? null : b.build_id)}
            />
          ))}
        </div>
      )}
    </div>
  )
}
