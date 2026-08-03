import { useState, useEffect, useMemo } from 'react'
import type { BackupEntry, AutoBackupEntry, SafetyBackupEntry } from '../types'
import EmptyState from './EmptyState'
import { formatSize, formatTs, operationLabel, diffStr, GOLD, MUTED, BAD } from '../utils/backupHelpers'
import type { UnifiedEntry, RestoreTarget } from '../utils/backupHelpers'

interface Props {
  entry: UnifiedEntry | null
  backups: BackupEntry[]
  autoBackups: AutoBackupEntry[]
  safetyBackups: SafetyBackupEntry[]
  selected: Record<string, Set<string>>
  restoring: string | null
  onToggleSelect: (entryId: string, filename: string) => void
  onToggleAll: (entryId: string, filenames: string[]) => void
  onRestore: (target: RestoreTarget) => void
  onDelete: (timestamp: string) => void
}

export default function BackupRecoveryDetail({
  entry, backups, autoBackups, safetyBackups,
  selected, restoring,
  onToggleSelect, onToggleAll, onRestore, onDelete,
}: Props) {
  const manualEntry = useMemo(() =>
    entry?.kind === 'manual' ? backups.find(b => b.timestamp === entry.ts) : null
  , [entry, backups])
  const autoEntry = useMemo(() =>
    entry?.kind === 'auto' ? autoBackups.find(a => a.filename === entry.filename) : null
  , [entry, autoBackups])
  const safetyEntry = useMemo(() =>
    entry?.kind === 'safety' ? safetyBackups.find(s => s.dirname === entry.dirname) : null
  , [entry, safetyBackups])
  const [fileTypeFilter, setFileTypeFilter] = useState('all')
  const visibleManualFiles = useMemo(() => {
    const files = manualEntry?.files ?? []
    if (fileTypeFilter === 'all') return files
    return files.filter(file => file.file_type === fileTypeFilter)
  }, [manualEntry, fileTypeFilter])

  useEffect(() => {
    setFileTypeFilter('all')
  }, [entry?.id])

  if (!entry) {
    return (
      <div className="recovery-detail-empty">
        <EmptyState
          icon="arrow-pointer"
          title="选择恢复点"
          hint="从左侧列表选择一个备份点以查看详情并执行恢复操作"
          minHeight={280}
        />
      </div>
    )
  }

  const isRestoring = restoring === entry.id
  const selectedFiles = selected[entry.id] ?? new Set<string>()
  const manualFiles = visibleManualFiles
  const safetyFiles = safetyEntry?.files ?? []
  const allFilesSelected = manualFiles.length > 0 && manualFiles.every(f => selectedFiles.has(f.filename))
  const allSafetyFilesSelected = safetyFiles.length > 0 && safetyFiles.every(f => selectedFiles.has(f))

  return (
    <div className="recovery-detail">
      {/* Detail header */}
      <div className="recovery-detail-header">
        <div className="recovery-detail-title">
          <i className={`fa-solid ${entry.kind === 'manual' ? 'fa-box-archive' : entry.kind === 'auto' ? 'fa-rotate' : 'fa-shield-halved'}`} aria-hidden="true" />
          <span>{entry.label}</span>
        </div>
        <div className="recovery-detail-subtitle">{entry.subtitle}</div>
        <div className="recovery-detail-meta-row">
          <span className="recovery-detail-badge">
            {entry.kind === 'manual' ? '手动备份' : entry.kind === 'auto' ? `操作备份 · ${entry.operation}` : '安全副本'}
          </span>
          <span className="recovery-detail-size">{formatSize(entry.size)}</span>
          {entry.kind === 'manual' && (
            <button className="d2emu-btn d2emu-btn-danger d2emu-btn-sm recovery-detail-delete"
              onClick={() => onDelete(entry.ts)} aria-label={`删除备份 ${entry.label}`}>
              <i className="fa-solid fa-trash" aria-hidden="true" /> 删除备份
            </button>
          )}
        </div>
      </div>

      {/* Manual: file table */}
      {entry.kind === 'manual' && manualEntry && (
        <div className="recovery-detail-section">
          <div className="recovery-detail-section-header">
            <span className="recovery-detail-section-title">文件列表</span>
            <select className="file-type-filter" value={fileTypeFilter}
              onChange={event => setFileTypeFilter(event.target.value)} aria-label="筛选备份文件类型">
              <option value="all">全部文件</option>
              <option value="character">角色</option>
              <option value="stash">仓库</option>
              <option value="config">配置</option>
              <option value="other">其他</option>
            </select>
            <span className="recovery-detail-section-count">
              {visibleManualFiles.length} / {manualEntry.files.length} 个文件
            </span>
          </div>

          <div className="file-table-head" role="row">
            <div role="columnheader" className="file-col-check" />
            <div role="columnheader" className="file-col-icon" />
            <div role="columnheader" className="file-col-name">文件</div>
            <div role="columnheader" className="file-col-num">备份</div>
            <div role="columnheader" className="file-col-num">当前</div>
            <div role="columnheader" className="file-col-diff">差异</div>
            <div role="columnheader" className="file-col-action" />
          </div>

          <div className="file-table-body">
            {visibleManualFiles.map(f => {
              const diff = diffStr(f.backup_size, f.current_size)
              const icon = f.file_type === 'character' ? 'fa-user' : f.file_type === 'stash' ? 'fa-box' : 'fa-file'
              return (
                <div key={f.filename} className="file-row" role="row">
                  <div className="file-col-check">
                    <input type="checkbox"
                      checked={selected[entry.id]?.has(f.filename) ?? false}
                      onChange={() => onToggleSelect(entry.id, f.filename)}
                      aria-label={`选择 ${f.filename}`}
                      style={{ accentColor: GOLD }}
                    />
                  </div>
                  <div className="file-col-icon" aria-hidden="true">
                    <i className={`fa-solid ${icon}`} />
                  </div>
                  <div className="file-col-name" title={f.filename}>{f.filename}</div>
                  <div className="file-col-num d2emu-mono-num">{formatSize(f.backup_size)}</div>
                  <div className="file-col-num d2emu-mono-num">
                    {f.current_size !== null ? formatSize(f.current_size) : '-'}
                  </div>
                  <div className={`file-col-diff d2emu-mono-num ${diff.cls}`} title={diff.label}>
                    {diff.text}
                  </div>
                  <div className="file-col-action">
                    <button className="d2emu-btn d2emu-btn-action d2emu-btn-sm"
                      disabled={isRestoring}
                      onClick={() => onRestore({ kind: 'manual', ts: entry.ts, files: [f.filename] })}
                      aria-label={`恢复 ${f.filename}`}>
                      {isRestoring ? '…' : '恢复'}
                    </button>
                  </div>
                </div>
              )
            })}
          </div>

          {/* Bulk actions */}
          <div className="file-bulk-actions">
            <label className="file-bulk-select">
              <input type="checkbox"
                checked={allFilesSelected}
                aria-label="全选当前备份文件"
                style={{ accentColor: GOLD }}
                onChange={() => onToggleAll(entry.id, visibleManualFiles.map(f => f.filename))}
              />
              <span>{selectedFiles.size > 0 ? `已选 ${selectedFiles.size} 项` : '全选当前筛选'}</span>
            </label>
            <button className="d2emu-btn d2emu-btn-action d2emu-btn-sm"
              disabled={isRestoring || selectedFiles.size === 0}
              onClick={() => onRestore({ kind: 'manual', ts: entry.ts, files: Array.from(selectedFiles) })}
              aria-label={`恢复选中的 ${selectedFiles.size} 个文件`}>
              <i className="fa-solid fa-rotate" aria-hidden="true" />
              {selectedFiles.size > 0 ? `恢复选中 ${selectedFiles.size} 项` : '恢复选中'}
            </button>
            <button className="d2emu-btn d2emu-btn-danger d2emu-btn-sm"
              disabled={isRestoring}
              onClick={() => onRestore({ kind: 'manual', ts: entry.ts })}
              aria-label="恢复全部文件">
              <i className="fa-solid fa-triangle-exclamation" aria-hidden="true" />
              恢复全部
            </button>
          </div>
        </div>
      )}

      {/* Auto: single restore */}
      {entry.kind === 'auto' && autoEntry && (
        <div className="recovery-detail-section">
          <div className="recovery-detail-section-header">
            <span className="recovery-detail-section-title">操作备份</span>
          </div>
          <div className="auto-restore-row">
            <div className="auto-restore-info">
              <span className="auto-restore-op">
                {operationLabel(autoEntry.operation)}
              </span>
              <span className="auto-restore-stash">{autoEntry.original_stash}</span>
            </div>
            <button className="d2emu-btn d2emu-btn-action d2emu-btn-sm"
              disabled={isRestoring}
              onClick={() => onRestore({ kind: 'auto', filename: autoEntry.filename })}
              aria-label="恢复此自动备份">
              {isRestoring ? '恢复中…' : '恢复'}
            </button>
          </div>
        </div>
      )}

      {/* Safety: selective or full restore */}
      {entry.kind === 'safety' && safetyEntry && (
        <div className="recovery-detail-section">
          <div className="recovery-detail-section-header">
            <span className="recovery-detail-section-title">恢复前安全副本</span>
            <span className="recovery-detail-section-count">{safetyEntry.file_count} 个文件 · {formatSize(safetyEntry.total_size)}</span>
          </div>
          <p className="safety-restore-note">
            这是恢复操作前自动创建的副本，可用于回退当前存档。
          </p>
          <div className="safety-file-list">
            <label className="file-bulk-select safety-select-all">
              <input type="checkbox"
                checked={allSafetyFilesSelected}
                aria-label="全选安全副本文件"
                style={{ accentColor: GOLD }}
                onChange={() => onToggleAll(entry.id, safetyFiles)}
              />
              <span>{selectedFiles.size > 0 ? `已选 ${selectedFiles.size}/${safetyFiles.length}` : '全选'}</span>
            </label>
            {safetyFiles.map(filename => (
              <div key={filename} className="safety-file-row">
                <input type="checkbox"
                  checked={selectedFiles.has(filename)}
                  aria-label={`选择 ${filename}`}
                  style={{ accentColor: GOLD }}
                  onChange={() => onToggleSelect(entry.id, filename)}
                />
                <span title={filename}>{filename}</span>
                <button className="d2emu-btn d2emu-btn-danger d2emu-btn-sm"
                  disabled={isRestoring}
                  onClick={event => { event.preventDefault(); onRestore({ kind: 'safety', dirname: entry.dirname, files: [filename] }) }}
                  aria-label={`恢复 ${filename}`}>
                  {isRestoring ? '…' : '恢复'}
                </button>
              </div>
            ))}
          </div>
          <div className="file-bulk-actions">
            <button className="d2emu-btn d2emu-btn-action d2emu-btn-sm"
              disabled={isRestoring || selectedFiles.size === 0}
              onClick={() => onRestore({ kind: 'safety', dirname: entry.dirname, files: Array.from(selectedFiles) })}
              aria-label={`恢复选中的 ${selectedFiles.size} 个安全副本文件`}>
              恢复选中 {selectedFiles.size > 0 ? `${selectedFiles.size} 项` : ''}
            </button>
            <button className="d2emu-btn d2emu-btn-danger d2emu-btn-sm"
              disabled={isRestoring}
              onClick={() => onRestore({ kind: 'safety', dirname: entry.dirname })}
              aria-label="恢复全部安全副本文件">
              <i className="fa-solid fa-triangle-exclamation" aria-hidden="true" /> 恢复全部
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
