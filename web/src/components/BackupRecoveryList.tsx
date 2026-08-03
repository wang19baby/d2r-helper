import { useMemo } from 'react'
import type { UnifiedEntry } from '../utils/backupHelpers'
import { formatSize } from '../utils/backupHelpers'

interface Props {
  entries: UnifiedEntry[]
  selectedId: string | null
  onSelect: (id: string) => void
  searchQ: string
  typeFilter: string
  onSearchQ: (v: string) => void
  onTypeFilter: (v: string) => void
  restoringId: string | null
  loading: boolean
}

const kindLabel: Record<string, string> = { manual: '手动', auto: '自动', safety: '安全副本' }
const kindCls: Record<string, string> = { manual: 'kind-manual', auto: 'kind-auto', safety: 'kind-safety' }

export default function BackupRecoveryList({
  entries, selectedId, onSelect,
  searchQ, typeFilter, onSearchQ, onTypeFilter,
  restoringId, loading,
}: Props) {
  const filtered = useMemo(() => entries.filter(e => {
    if (typeFilter !== 'all' && e.kind !== typeFilter) return false
    if (searchQ && !e.searchText.toLowerCase().includes(searchQ.toLowerCase())) return false
    return true
  }), [entries, typeFilter, searchQ])

  return (
    <div className="recovery-list-panel">
      <div className="recovery-list-filters">
        <select
          className="recovery-filter-select"
          value={typeFilter}
          onChange={e => onTypeFilter(e.target.value)}
          aria-label="筛选备份类型">
          <option value="all">全部</option>
          <option value="manual">手动</option>
          <option value="auto">自动</option>
          <option value="safety">安全副本</option>
        </select>
        <input
          className="recovery-search-input"
          type="search"
          placeholder="搜索…"
          value={searchQ}
          onChange={e => onSearchQ(e.target.value)}
          aria-label="搜索备份"
        />
      </div>

      <div
        className="recovery-list-results"
        role="listbox"
        tabIndex={0}
        aria-label="恢复点列表"
        aria-activedescendant={selectedId ?? undefined}
        onKeyDown={(event: React.KeyboardEvent<HTMLDivElement>) => {
          if (filtered.length === 0) return
          const currentIndex = Math.max(0, filtered.findIndex(e => e.id === selectedId))
          let nextIndex = currentIndex
          if (event.key === 'ArrowDown') nextIndex = Math.min(filtered.length - 1, currentIndex + 1)
          else if (event.key === 'ArrowUp') nextIndex = Math.max(0, currentIndex - 1)
          else if (event.key === 'Home') nextIndex = 0
          else if (event.key === 'End') nextIndex = filtered.length - 1
          else return
          event.preventDefault()
          onSelect(filtered[nextIndex].id)
        }}
      >
        {loading && entries.length === 0 ? (
          <div className="recovery-list-empty" role="status" aria-live="polite">
            <i className="fa-solid fa-spinner fa-spin" aria-hidden="true" />
            <span>正在加载恢复点…</span>
          </div>
        ) : filtered.length === 0 ? (
          <div className="recovery-list-empty" role="option" aria-selected="false">
            <i className="fa-solid fa-inbox" aria-hidden="true" />
            <span>无匹配结果</span>
          </div>
        ) : (
          filtered.map(entry => (
            <div
              key={entry.id}
              id={entry.id}
              role="option"
              aria-selected={selectedId === entry.id}
              aria-label={`${entry.label}，${entry.subtitle}`}
              className={`recovery-list-item${selectedId === entry.id ? ' is-selected' : ''}${restoringId === entry.id ? ' is-restoring' : ''}`}
              onClick={() => onSelect(entry.id)}
              tabIndex={-1}
            >
              <div className={`recovery-list-kind ${kindCls[entry.kind]}`} aria-hidden="true">
                <i className={`fa-solid ${entry.kind === 'manual' ? 'fa-box-archive' : entry.kind === 'auto' ? 'fa-rotate' : 'fa-shield-halved'}`} />
                <span>{kindLabel[entry.kind]}</span>
              </div>
              <div className="recovery-list-body">
                <div className="recovery-list-primary">{entry.label}</div>
                <div className="recovery-list-secondary">{entry.subtitle}</div>
              </div>
              <div className="recovery-list-meta">
                <span className="recovery-list-size">{formatSize(entry.size)}</span>
                {restoringId === entry.id && (
                  <i className="fa-solid fa-spinner fa-spin" aria-label="恢复中" />
                )}
              </div>
            </div>
          ))
        )}
      </div>

      <div className="recovery-list-footer">
        <span className="recovery-list-count">
          {filtered.length} / {entries.length} 个恢复点
        </span>
      </div>
    </div>
  )
}
