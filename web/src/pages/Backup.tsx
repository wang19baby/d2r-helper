import { useEffect, useState, useCallback, useMemo } from 'react'
import { tauriInvoke } from '../tauri'
import { showToast } from '../components/Toast'
import D2EmuCard from '../components/D2EmuCard'
import D2ConfirmModal from '../components/D2ConfirmModal'
import type { AppConfig, BackupEntry, BackupResult, AutoBackupEntry, AutoSaveInfo, SafetyBackupEntry, ModMeta } from '../types'

import BackupPageHeader from '../components/BackupPageHeader'
import BackupAutoSavePanel from '../components/BackupAutoSavePanel'
import BackupRetentionControls from '../components/BackupRetentionControls'
import BackupRecoveryList from '../components/BackupRecoveryList'
import BackupRecoveryDetail from '../components/BackupRecoveryDetail'

import {
  formatSize, relativeTime, formatTs, MUTED, BAD,
  buildUnifiedList, RestoreImpact,
} from '../utils/backupHelpers'
import type { UnifiedEntry, RestoreTarget, ConfirmDialog } from '../utils/backupHelpers'
import { useLocale } from '../locales/context'

/* ─── Main component ──────────────────────────────────────────── */
export default function Backup() {
  const { t } = useLocale()
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [folder, setFolder] = useState('')
  const [modHasSavePath, setModHasSavePath] = useState(false)
  const [currentModMeta, setCurrentModMeta] = useState<ModMeta | undefined>()

  const [backups, setBackups] = useState<BackupEntry[]>([])
  const [autoBackups, setAutoBackups] = useState<AutoBackupEntry[]>([])
  const [safetyBackups, setSafetyBackups] = useState<SafetyBackupEntry[]>([])
  const [autoSaveInfo, setAutoSaveInfo] = useState<AutoSaveInfo | null>(null)
  const [retentionDays, setRetentionDays] = useState(5)

  const [creatingBackup, setCreatingBackup] = useState(false)
  const [restoring, setRestoring] = useState<string | null>(null)
  const [archiving, setArchiving] = useState(false)
  const [cleaningAuto, setCleaningAuto] = useState(false)
  const [timerOn, setTimerOn] = useState(false)
  const [triggering, setTriggering] = useState(false)

  const [searchQ, setSearchQ] = useState('')
  const [typeFilter, setTypeFilter] = useState<string>('all')
  const [selected, setSelected] = useState<Record<string, Set<string>>>({})

  const [confirmDialog, setConfirmDialog] = useState<ConfirmDialog | null>(null)
  const [selectedEntry, setSelectedEntry] = useState<string | null>(null)
  const [loadingBackups, setLoadingBackups] = useState(true)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [lastSyncedAt, setLastSyncedAt] = useState<Date | null>(null)
  const [refreshing, setRefreshing] = useState(false)

  const entries = useMemo(() =>
    buildUnifiedList(backups, autoBackups, safetyBackups),
    [backups, autoBackups, safetyBackups]
  )

  /* ── Load ─────────────────────────────────────────────────── */
  const loadConfig = useCallback(async () => {
    try {
      const c = await tauriInvoke('get_app_config') as AppConfig
      setConfig(c)
      setFolder(c.save_folder || c.default_folder || '')
    } catch { /* ignore */ }
  }, [])

  const loadBackups = useCallback(async (notifyError = false) => {
    try {
      const [manual, auto, days, saveInfo, safety] = await Promise.all([
        tauriInvoke('list_backups') as Promise<BackupEntry[]>,
        tauriInvoke('list_auto_backups') as Promise<AutoBackupEntry[]>,
        tauriInvoke('get_auto_backup_retention') as Promise<number>,
        tauriInvoke('get_auto_save_info') as Promise<AutoSaveInfo>,
        tauriInvoke('list_safety_backups') as Promise<SafetyBackupEntry[]>,
      ])
      setBackups(manual)
      setAutoBackups(auto)
      setRetentionDays(days)
      setAutoSaveInfo(saveInfo)
      setSafetyBackups(safety)
      setLoadError(null)
      setLastSyncedAt(new Date())
    } catch (e: any) {
      const message = e?.message || '加载备份列表失败'
      setLoadError(message)
      if (notifyError) showToast(message, 'error')
    } finally {
      setLoadingBackups(false)
    }
  }, [])

  useEffect(() => {
    loadConfig(); loadBackups()
    const timer = setInterval(() => loadBackups(), 10_000)
    return () => clearInterval(timer)
  }, [loadConfig, loadBackups])

  useEffect(() => {
    const handleAutoSaveEvent = (event: Event) => {
      const action = (event as CustomEvent<{ action?: string }>).detail?.action
      if (action === 'start') setTimerOn(true)
      if (action === 'stop') setTimerOn(false)
    }
    window.addEventListener('auto-save', handleAutoSaveEvent)
    return () => window.removeEventListener('auto-save', handleAutoSaveEvent)
  }, [])

  useEffect(() => {
    setSelectedEntry(current => {
      if (current && entries.some(entry => entry.id === current)) return current
      return entries[0]?.id ?? null
    })
  }, [entries])

  /* ── Actions ──────────────────────────────────────────────── */
  const handleCreateBackup = async () => {
    setCreatingBackup(true)
    try {
      const res = await tauriInvoke('create_stash_backup') as any
      showToast(`备份成功：${res.backup_count} 个文件`, 'success')
      loadBackups()
    } catch (e: any) { showToast(e.message || '备份失败。', 'error') }
    finally { setCreatingBackup(false) }
  }

  const handleRefresh = useCallback(async () => {
    setRefreshing(true)
    try {
      await loadBackups(true)
    } finally {
      setRefreshing(false)
    }
  }, [loadBackups])

  const handleSaveFolder = async () => {
    try {
      await tauriInvoke('update_save_folder', { saveFolder: folder })
      showToast('已保存', 'success')
    } catch (e: any) { showToast(String(e), 'error') }
  }

  const handleRetentionChange = async (days: number) => {
    try {
      await tauriInvoke('set_auto_backup_retention', { days })
      showToast(`自动快照保留 ${days} 天`, 'success')
    } catch (e: any) {
      showToast(e?.message || '保存保留策略失败', 'error')
    }
  }

  const handleArchive = async () => {
    setArchiving(true)
    try {
      const res = await tauriInvoke('archive_old_auto_backups', { retentionDays }) as any
      showToast(`归档完成：${res.message}`, 'success')
      loadBackups()
    } catch (e: any) { showToast(e.message || '归档失败。', 'error') }
    finally { setArchiving(false) }
  }

  const handleCleanup = async () => {
    setCleaningAuto(true)
    try {
      await tauriInvoke('cleanup_auto_backups', { keepDays: Math.max(7, retentionDays) }) as any
      showToast('已清理旧备份', 'success')
      loadBackups()
    } catch (e: any) { showToast(e.message || '清理失败。', 'error') }
    finally { setCleaningAuto(false) }
  }

  const handleTriggerAutoSave = async () => {
    setTriggering(true)
    try {
      await tauriInvoke('auto_save_stash') as any
      showToast('已触发自动保存', 'success')
      loadBackups()
    } catch (e: any) { showToast(e.message || '触发失败。', 'error') }
    finally { setTriggering(false) }
  }

  const handleDeleteBackup = async (timestamp: string) => {
    if (!window.confirm(`删除 ${formatTs(timestamp)} 备份？`)) return
    try {
      await tauriInvoke('delete_backup', { timestamp }) as any
      setSelectedEntry(current => current === `manual-${timestamp}` ? null : current)
      setSelected(prev => {
        const next = { ...prev }
        delete next[`manual-${timestamp}`]
        return next
      })
      showToast('已删除备份', 'success')
      await loadBackups()
    } catch (e: any) {
      showToast(e?.message || '删除备份失败', 'error')
    }
  }

  const handleBrowseFolder = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const dir = await open({ directory: true, multiple: false, title: '选择 D2R 存档文件夹' })
      if (dir) setFolder(dir as string)
    } catch { /* fallback */ }
  }

  /* ── Restore with confirm ─────────────────────────────────── */
  const askRestore = (target: RestoreTarget) => {
    let title = ''
    let body: React.ReactNode = ''

    if (target.kind === 'snapshot') {
      const snapshotFiles = autoSaveInfo?.files ?? []
      title = '确认恢复自动快照'
      body = (
        <div>
          <p style={{ margin: '0 0 8px' }}>将恢复最近一次自动快照。</p>
          <RestoreImpact files={snapshotFiles} />
          <p style={{ margin: 0, color: MUTED, fontSize: 13 }}>
            此操作将覆盖当前存档文件，恢复前会保留安全副本。
          </p>
        </div>
      )
    } else if (target.kind === 'manual') {
      const backup = backups.find(item => item.timestamp === target.ts)
      const impacted = (backup?.files ?? []).filter(file => !target.files || target.files.includes(file.filename))
      const fileNames = impacted.map(file => file.filename)
      const changedCount = impacted.filter(file => file.current_size === null || file.current_size !== file.backup_size).length
      title = '确认恢复备份'
      body = (
        <div>
          <p style={{ margin: '0 0 8px' }}>
            将恢复备份 <strong style={{ color: MUTED }}>{formatTs(target.ts)}</strong>。
          </p>
          <RestoreImpact files={fileNames} changedCount={changedCount} />
          <p style={{ margin: 0, color: MUTED, fontSize: 13 }}>
            此操作将覆盖当前存档文件，无法撤销。恢复前会创建安全副本。
          </p>
        </div>
      )
    } else if (target.kind === 'auto') {
      const autoBackup = autoBackups.find(item => item.filename === target.filename)
      const impactedFiles = autoBackup?.original_stash ? [autoBackup.original_stash] : [target.filename]
      title = '确认恢复操作备份'
      body = (
        <div>
          <p style={{ margin: '0 0 8px' }}>
            将恢复操作备份 <strong style={{ color: MUTED }}>{target.filename}</strong>。
          </p>
          <RestoreImpact files={impactedFiles} />
          <p style={{ margin: 0, color: MUTED, fontSize: 13 }}>
            此操作将覆盖当前存档文件，恢复前会创建安全副本。
          </p>
        </div>
      )
    } else {
      const safetyBackup = safetyBackups.find(item => item.dirname === target.dirname)
      const impactedFiles = (safetyBackup?.files ?? []).filter(file => !target.files || target.files.includes(file))
      title = '确认恢复安全副本'
      body = (
        <div>
          <p style={{ margin: '0 0 8px' }}>
            将恢复安全副本 <strong style={{ color: BAD }}>{formatTs(target.dirname)}</strong>。
          </p>
          <RestoreImpact files={impactedFiles} />
          <p style={{ margin: 0, color: BAD, fontSize: 13 }}>
            这是恢复操作前自动创建的副本，恢复将覆盖当前存档。
          </p>
        </div>
      )
    }

    setConfirmDialog({ title, body, danger: true, target })
  }

  const executeRestore = async (target: RestoreTarget) => {
    setConfirmDialog(null)
    setRestoring(
      target.kind === 'snapshot' ? 'snapshot' :
      target.kind === 'manual' ? `manual-${target.ts}` :
      target.kind === 'auto' ? `auto-${target.filename}` : `safety-${target.dirname}`
    )
    try {
      let result: BackupResult
      if (target.kind === 'snapshot') {
        result = await tauriInvoke('restore_auto_save') as BackupResult
      } else if (target.kind === 'manual') {
        result = await tauriInvoke('restore_backup', { timestamp: target.ts, files: target.files || null }) as BackupResult
      } else if (target.kind === 'auto') {
        result = await tauriInvoke('restore_auto_backup', { backupFilename: target.filename }) as BackupResult
      } else {
        result = await tauriInvoke('restore_backup', { timestamp: target.dirname, files: target.files || null }) as BackupResult
      }
      showToast(`恢复完成：${result.backup_count} 个文件。${result.message}`, 'success')
      await loadBackups()
    } catch (e: any) {
      showToast(e?.message || '恢复失败。', 'error')
    } finally {
      setRestoring(null)
    }
  }

  /* ── Selection ─────────────────────────────────────────────── */
  const handleToggleSelect = useCallback((entryId: string, filename: string) => {
    setSelected(prev => {
      const cur = prev[entryId] ?? new Set<string>()
      const next = new Set(cur)
      if (next.has(filename)) next.delete(filename)
      else next.add(filename)
      return { ...prev, [entryId]: next }
    })
  }, [])

  const handleToggleAll = useCallback((entryId: string, filenames: string[]) => {
    setSelected(prev => {
      const current = prev[entryId] ?? new Set<string>()
      const allSelected = filenames.length > 0 && filenames.every(filename => current.has(filename))
      return {
        ...prev,
        [entryId]: allSelected ? new Set<string>() : new Set(filenames),
      }
    })
  }, [])

  /* ── Derived ─────────────────────────────────────────────── */
  const selectedEntryData = useMemo(() => {
    if (!selectedEntry) return null
    return entries.find(e => e.id === selectedEntry) ?? null
  }, [selectedEntry, entries])

  const pathOk = modHasSavePath || !!folder
  const modName = config?.active_mod || '(加载中…)'
  const manualTotalSize = backups.reduce((total, backup) => total + backup.total_size, 0)
  const latestManual = entries.find(entry => entry.kind === 'manual')
  const latestAuto = entries.find(entry => entry.kind === 'auto')

  return (
    <div className="font-d2emu-ui backup-page">
      <D2EmuCard
        kicker={t('backup.kicker')}
        title={t('backup.title')}
        lede={t('backup.desc')}
      />

      {/* Context header */}
      <div className="backup-context-header d2emu-card-quiet">
        <div className="backup-save-folder-row">
          <label className="backup-save-folder-label" htmlFor="save-folder-input">存档文件夹</label>
          <div className="backup-save-folder-inputs">
            <input
              id="save-folder-input"
              className="backup-save-folder-input"
              value={folder}
              onChange={e => setFolder(e.target.value)}
              placeholder="C:\Users\用户名\Saved Games\Diablo II Resurrected"
              aria-label="存档文件夹路径"
            />
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
              onClick={handleBrowseFolder} aria-label="浏览存档文件夹">
              <i className="fa-solid fa-folder-open" />
            </button>
            <button className="d2emu-btn d2emu-btn-primary d2emu-btn-sm"
              onClick={handleSaveFolder} aria-label="保存路径">
              <i className="fa-solid fa-floppy-disk" />
            </button>
          </div>
        </div>

        <BackupPageHeader
          modName={modName}
          pathOk={pathOk}
          folder={folder}
          meta={currentModMeta}
          creatingBackup={creatingBackup}
          refreshing={refreshing}
          onCreateBackup={handleCreateBackup}
          onRefresh={handleRefresh}
        />
      </div>

      <div className={`backup-sync-strip ${loadError ? 'is-error' : ''}`} role={loadError ? 'alert' : 'status'} aria-live="polite">
        <span>
          <i className={`fa-solid ${loadError ? 'fa-triangle-exclamation' : 'fa-circle-info'}`} aria-hidden="true" />
          {loadError ? loadError : lastSyncedAt ? `最后同步 ${lastSyncedAt.toLocaleTimeString('zh-CN')}` : '等待首次同步'}
        </span>
        {loadError && (
          <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={() => handleRefresh()}>
            重试
          </button>
        )}
      </div>

      <div className="backup-health-strip" aria-label="备份概览">
        <div className="backup-health-item">
          <span className="backup-health-label">恢复点</span>
          <strong>{entries.length}</strong>
        </div>
        <div className="backup-health-item">
          <span className="backup-health-label">最近手动</span>
          <strong>{latestManual ? relativeTime(latestManual.ts) : '暂无'}</strong>
        </div>
        <div className="backup-health-item">
          <span className="backup-health-label">自动快照</span>
          <strong>{autoSaveInfo?.file_count ? relativeTime(autoSaveInfo.timestamp) : latestAuto ? relativeTime(latestAuto.ts) : '暂无'}</strong>
        </div>
        <div className="backup-health-item">
          <span className="backup-health-label">手动占用</span>
          <strong>{formatSize(manualTotalSize)}</strong>
        </div>
      </div>

      {/* Controls row */}
      <div className="backup-controls-row">
        <BackupAutoSavePanel
          autoSaveInfo={autoSaveInfo}
          triggering={triggering}
          timerOn={timerOn}
          restoringSnapshot={restoring === 'snapshot'}
          onTrigger={handleTriggerAutoSave}
          onRestoreSnapshot={() => askRestore({ kind: 'snapshot' })}
          onStart={() => {
            setTimerOn(true)
            window.dispatchEvent(new CustomEvent('auto-save', { detail: { action: 'start' } }))
          }}
          onStop={() => {
            setTimerOn(false)
            window.dispatchEvent(new CustomEvent('auto-save', { detail: { action: 'stop' } }))
          }}
        />
        <BackupRetentionControls
          retentionDays={retentionDays} archiving={archiving} cleaningAuto={cleaningAuto}
          onRetentionChange={handleRetentionChange}
          onArchive={handleArchive}
          onCleanup={handleCleanup}
        />
      </div>

      {/* Master-detail console */}
      <div className="recovery-console">
        <BackupRecoveryList
          entries={entries}
          selectedId={selectedEntry}
          onSelect={setSelectedEntry}
          searchQ={searchQ}
          typeFilter={typeFilter}
          onSearchQ={setSearchQ}
          onTypeFilter={setTypeFilter}
          restoringId={restoring}
          loading={loadingBackups}
        />

        <BackupRecoveryDetail
          entry={selectedEntryData}
          backups={backups}
          autoBackups={autoBackups}
          safetyBackups={safetyBackups}
          selected={selected}
          restoring={restoring}
          onToggleSelect={handleToggleSelect}
          onToggleAll={handleToggleAll}
          onRestore={askRestore}
          onDelete={handleDeleteBackup}
        />
      </div>

      {/* Confirm modal */}
      {confirmDialog && (
        <D2ConfirmModal
          title={confirmDialog.title}
          danger={confirmDialog.danger}
          onConfirm={() => executeRestore(confirmDialog.target)}
          onClose={() => setConfirmDialog(null)}
          confirmText="确认恢复"
          cancelText="取消"
        >
          {confirmDialog.body}
        </D2ConfirmModal>
      )}

      {/* Styles */}
      <style>{`
        /* ════════════════════════════════════════════════════════════
           Backup 页面 UI (字号放大版)
           字号刻度:label 13px / body 16px / strong 17-18px / title 26px
           间距刻度:卡片 16-18px / 列表项 12px 14px / 段标题 14px 18px
           ════════════════════════════════════════════════════════════ */
        .backup-page {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        /* ── Health strip (顶部 4 列指标) ── */
        .backup-health-strip {
          display: grid;
          grid-template-columns: repeat(4, minmax(0, 1fr));
          gap: 12px;
        }
        .backup-health-item {
          display: flex;
          align-items: baseline;
          justify-content: space-between;
          gap: 12px;
          min-width: 0;
          padding: 14px 16px;
          border: 1px solid var(--color-d2emu-line);
          border-radius: 6px;
          background: rgba(255,255,255,0.02);
        }
        .backup-health-label {
          color: var(--color-d2emu-muted);
          font-size: 14px;
          font-weight: 600;
          letter-spacing: 0.06em;
          text-transform: uppercase;
          white-space: nowrap;
        }
        .backup-health-item strong {
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
          color: var(--color-d2emu-text);
          font-size: 18px;
          font-weight: 700;
        }

        /* ── Restore impact (modal 内) ── */
        .restore-impact {
          margin: 12px 0 14px;
          padding: 12px 14px;
          border: 1px solid var(--color-d2emu-line);
          border-radius: 6px;
          background: rgba(0,0,0,0.24);
        }
        .restore-impact-summary { color: var(--color-d2emu-text); font-size: 16px; line-height: 1.55; }
        .restore-impact-summary strong { color: var(--color-d2emu-gold); font-size: 17px; }
        .restore-impact-files {
          margin: 8px 0 0;
          padding-left: 20px;
          color: var(--color-d2emu-muted);
          font-size: 15px;
          line-height: 1.55;
        }

        /* ── Context header (存档文件夹输入) ── */
        .backup-context-header {
          padding: 18px 20px;
          display: flex;
          flex-direction: column;
          gap: 16px;
        }
        .backup-save-folder-row {
          display: flex;
          flex-direction: column;
          gap: 6px;
        }
        .backup-save-folder-label {
          font-size: 14px;
          font-weight: 600;
          color: var(--color-d2emu-muted);
          text-transform: uppercase;
          letter-spacing: 0.08em;
        }
        .backup-save-folder-inputs {
          display: flex;
          gap: 8px;
          align-items: center;
        }
        .backup-save-folder-input {
          flex: 1;
          background: var(--color-d2emu-field);
          color: var(--color-d2emu-text);
          border: 1px solid var(--color-d2emu-line);
          border-radius: 4px;
          padding: 10px 14px;
          font: 16px/1.4 "Source Sans 3", sans-serif;
          outline: none;
          box-sizing: border-box;
        }
        .backup-save-folder-input:focus { border-color: var(--color-d2emu-gold); }

        /* ── Page header (Mod / 路径标识) ── */
        .backup-page-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          flex-wrap: wrap;
          gap: 12px;
        }
        .backup-header-identity {
          display: flex;
          align-items: center;
          gap: 20px;
          flex-wrap: wrap;
        }
        .backup-header-mod,
        .backup-header-path {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }
        .backup-header-mod-label,
        .backup-header-path-label {
          font-size: 13px;
          font-weight: 600;
          color: var(--color-d2emu-muted);
          text-transform: uppercase;
          letter-spacing: 0.08em;
        }
        .backup-header-mod-value {
          font-size: 18px;
          font-weight: 700;
          color: var(--color-d2emu-gold);
          font-family: "Cinzel", serif;
        }
        .backup-header-path-value {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 15px;
          max-width: 320px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        .backup-header-path-value.is-ok { color: var(--color-d2emu-good); }
        .backup-header-path-value.is-warn { color: var(--color-d2emu-bad); }
        .backup-header-path-text { overflow: hidden; text-overflow: ellipsis; }
        .backup-header-meta-tag {
          font-size: 13px;
          font-weight: 600;
          padding: 4px 10px;
          border: 1px solid var(--color-d2emu-line);
          border-radius: 999px;
          color: var(--color-d2emu-muted);
        }
        .backup-header-actions { display: flex; gap: 8px; align-items: center; }

        /* ── Sync strip (同步状态条) ── */
        .backup-sync-strip {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 12px;
          padding: 10px 16px;
          border: 1px solid var(--color-d2emu-line);
          border-radius: 6px;
          color: var(--color-d2emu-muted);
          background: rgba(255,255,255,0.02);
          font-size: 15px;
          line-height: 1.5;
        }
        .backup-sync-strip.is-error {
          border-color: rgba(239,83,80,0.45);
          color: #ff9a9a;
          background: rgba(239,83,80,0.08);
        }
        .backup-sync-strip > span { display: inline-flex; align-items: center; gap: 8px; min-width: 0; }

        /* ── Controls row (auto-save + retention 双列) ── */
        .backup-controls-row {
          display: flex;
          gap: 16px;
          flex-wrap: wrap;
          align-items: stretch;
        }
        .auto-save-panel {
          flex: 0 0 auto;
          min-width: 280px;
          padding: 16px 18px;
          border: 1px solid var(--color-d2emu-line);
          border-radius: 8px;
          background: var(--color-d2emu-panel);
          display: flex;
          flex-direction: column;
          gap: 12px;
        }
        .auto-save-label {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 14px;
          font-weight: 600;
          color: var(--color-d2emu-muted);
          text-transform: uppercase;
          letter-spacing: 0.08em;
        }
        .auto-save-snapshot {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 12px;
          padding: 12px 14px;
          border: 1px solid rgba(76,175,80,0.3);
          border-radius: 6px;
          background: rgba(76,175,80,0.05);
        }
        .auto-save-snapshot.is-empty {
          border-color: var(--color-d2emu-line);
          background: rgba(0,0,0,0.2);
          justify-content: space-between;
        }
        .auto-save-snapshot-info {
          display: flex;
          gap: 14px;
          align-items: center;
          flex-wrap: wrap;
        }
        .auto-save-snapshot-actions {
          display: flex;
          align-items: center;
          gap: 8px;
          flex-shrink: 0;
        }
        .auto-save-snapshot-files,
        .auto-save-snapshot-size {
          font-size: 16px;
          font-weight: 700;
          color: var(--color-d2emu-text);
        }
        .auto-save-snapshot-time { font-size: 13px; color: var(--color-d2emu-muted); }
        .auto-save-empty-text { font-size: 15px; color: var(--color-d2emu-muted); }
        .auto-save-status-badge {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 12px;
          font-weight: 700;
          padding: 4px 10px;
          border-radius: 999px;
          text-transform: uppercase;
          letter-spacing: 0.06em;
          white-space: nowrap;
          flex-shrink: 0;
        }
        .auto-save-status-badge.is-active {
          background: rgba(76,175,80,0.12);
          color: var(--color-d2emu-good);
          border: 1px solid rgba(76,175,80,0.3);
        }
        .auto-save-status-badge.is-inactive {
          background: rgba(0,0,0,0.2);
          color: var(--color-d2emu-muted);
          border: 1px solid var(--color-d2emu-line);
        }
        .auto-save-status-badge i { font-size: 10px; }
        .auto-save-controls { display: flex; gap: 8px; flex-wrap: wrap; }

        /* ── Retention controls ── */
        .retention-controls {
          flex: 1 1 auto;
          min-width: 240px;
          padding: 16px 18px;
          border: 1px solid var(--color-d2emu-line);
          border-radius: 8px;
          background: var(--color-d2emu-panel);
          display: flex;
          flex-direction: column;
          gap: 12px;
        }
        .retention-row {
          display: flex;
          align-items: center;
          gap: 12px;
          flex-wrap: wrap;
        }
        .retention-label {
          font-size: 14px;
          font-weight: 600;
          color: var(--color-d2emu-muted);
          text-transform: uppercase;
          letter-spacing: 0.08em;
          white-space: nowrap;
        }
        .retention-input {
          width: 72px;
          padding: 8px 10px;
          background: var(--color-d2emu-field);
          color: var(--color-d2emu-text);
          border: 1px solid var(--color-d2emu-line);
          border-radius: 4px;
          font: 16px/1.4 "Source Sans 3", sans-serif;
          text-align: center;
          outline: none;
        }
        .retention-input:focus { border-color: var(--color-d2emu-gold); }

        /* ── Recovery console (主从布局) ── */
        .recovery-console {
          display: grid;
          grid-template-columns: minmax(260px, 1fr) minmax(360px, 2fr);
          gap: 16px;
          align-items: start;
        }

        /* ── Left: unified list ── */
        .recovery-list-panel {
          border: 1px solid var(--color-d2emu-line);
          border-radius: 8px;
          background: var(--color-d2emu-panel);
          display: flex;
          flex-direction: column;
          overflow: hidden;
        }
        .recovery-list-filters {
          display: flex;
          gap: 8px;
          padding: 12px 14px;
          border-bottom: 1px solid var(--color-d2emu-line);
          flex-wrap: wrap;
        }
        .recovery-filter-select,
        .recovery-search-input {
          flex: 1;
          min-width: 100px;
          padding: 8px 12px;
          background: var(--color-d2emu-field);
          color: var(--color-d2emu-text);
          border: 1px solid var(--color-d2emu-line);
          border-radius: 4px;
          font: 16px/1.4 "Source Sans 3", sans-serif;
          outline: none;
        }
        .recovery-filter-select:focus,
        .recovery-search-input:focus { border-color: var(--color-d2emu-gold); }
        .recovery-search-input::placeholder { color: var(--color-d2emu-muted); font-size: 15px; }

        .recovery-list-results {
          flex: 1;
          overflow-y: auto;
          max-height: 520px;
        }
        .recovery-list-empty {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 10px;
          padding: 48px 24px;
          color: var(--color-d2emu-muted);
          font-size: 17px;
        }
        .recovery-list-empty i { font-size: 34px; opacity: 0.4; }

        .recovery-list-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px 14px;
          border-bottom: 1px solid rgba(37,37,37,0.8);
          cursor: pointer;
          transition: background 120ms ease;
          user-select: none;
        }
        .recovery-list-item:hover { background: rgba(255,255,255,0.03); }
        .recovery-list-item.is-selected {
          background: rgba(251,177,58,0.08);
          border-left: 3px solid var(--color-d2emu-gold);
          padding-left: 11px;
        }
        .recovery-list-item.is-restoring { opacity: 0.6; }

        .recovery-list-kind {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 3px;
          width: 44px;
          flex-shrink: 0;
          font-size: 11px;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.04em;
          text-align: center;
        }
        .recovery-list-kind i { font-size: 16px; }
        .kind-manual { color: var(--color-d2emu-gold); }
        .kind-auto { color: #5a9e6f; }
        .kind-safety { color: var(--color-d2emu-bad); }

        .recovery-list-body { flex: 1; min-width: 0; }
        .recovery-list-primary {
          font-size: 17px;
          font-weight: 700;
          color: var(--color-d2emu-text);
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
          line-height: 1.3;
        }
        .recovery-list-secondary {
          font-size: 15px;
          color: var(--color-d2emu-muted);
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
          margin-top: 3px;
          line-height: 1.3;
        }
        .recovery-list-meta {
          display: flex;
          flex-direction: column;
          align-items: flex-end;
          gap: 3px;
          flex-shrink: 0;
        }
        .recovery-list-size { font-size: 15px; color: var(--color-d2emu-muted); white-space: nowrap; font-weight: 600; }

        .recovery-list-footer {
          padding: 12px 14px;
          border-top: 1px solid var(--color-d2emu-line);
        }
        .recovery-list-count { font-size: 15px; color: var(--color-d2emu-muted); }

        /* ── Right: detail ── */
        .recovery-detail-empty,
        .recovery-detail {
          border: 1px solid var(--color-d2emu-line);
          border-radius: 8px;
          background: var(--color-d2emu-panel);
          overflow: hidden;
        }
        .recovery-detail-header {
          padding: 18px 20px;
          border-bottom: 1px solid var(--color-d2emu-line);
          background: rgba(0,0,0,0.15);
        }
        .recovery-detail-title {
          display: flex;
          align-items: center;
          gap: 10px;
          font-size: 26px;
          font-weight: 700;
          color: var(--color-d2emu-text);
          font-family: "Cinzel", serif;
          margin-bottom: 6px;
          line-height: 1.2;
        }
        .recovery-detail-title i { color: var(--color-d2emu-gold); font-size: 22px; }
        .recovery-detail-subtitle { font-size: 16px; color: var(--color-d2emu-muted); margin-bottom: 10px; line-height: 1.5; }
        .recovery-detail-meta-row {
          display: flex;
          align-items: center;
          gap: 10px;
          flex-wrap: wrap;
        }
        .recovery-detail-badge {
          font-size: 12px;
          font-weight: 700;
          padding: 4px 10px;
          border-radius: 999px;
          text-transform: uppercase;
          letter-spacing: 0.06em;
          background: rgba(251,177,58,0.08);
          color: var(--color-d2emu-gold);
          border: 1px solid rgba(251,177,58,0.2);
        }
        .recovery-detail-size { font-size: 16px; color: var(--color-d2emu-muted); font-weight: 600; }

        .recovery-detail-section { border-top: 1px solid var(--color-d2emu-line); }
        .recovery-detail-section-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 14px 20px 10px;
        }
        .recovery-detail-section-title {
          font-size: 15px;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          color: var(--color-d2emu-muted);
        }
        .recovery-detail-section-count { font-size: 15px; color: var(--color-d2emu-muted); font-weight: 600; }
        .file-type-filter {
          margin-left: auto;
          min-width: 110px;
          padding: 7px 10px;
          border: 1px solid var(--color-d2emu-line);
          border-radius: 4px;
          background: var(--color-d2emu-field);
          color: var(--color-d2emu-text);
          font: 15px/1.3 "Source Sans 3", sans-serif;
        }
        .file-type-filter:focus { border-color: var(--color-d2emu-gold); }

        /* ── File table (文件差异列表) ── */
        .file-table-head {
          display: grid;
          grid-template-columns: 32px 28px 1fr 60px 60px 60px 78px;
          gap: 6px;
          padding: 8px 16px;
          font-size: 12px;
          font-weight: 700;
          text-transform: uppercase;
          letter-spacing: 0.06em;
          color: var(--color-d2emu-muted);
          border-bottom: 1px solid var(--color-d2emu-line);
          align-items: center;
        }
        .file-table-body { max-height: 320px; overflow-y: auto; }
        .file-row {
          display: grid;
          grid-template-columns: 32px 28px 1fr 60px 60px 60px 78px;
          gap: 6px;
          padding: 9px 16px;
          align-items: center;
          border-bottom: 1px solid rgba(37,37,37,0.8);
          transition: background 80ms ease;
        }
        .file-row:hover { background: rgba(255,255,255,0.02); }
        .file-col-check,
        .file-col-icon,
        .file-col-num,
        .file-col-diff,
        .file-col-action { display: flex; align-items: center; }
        .file-col-icon { color: var(--color-d2emu-muted); font-size: 15px; justify-content: center; }
        .file-col-name {
          font-size: 16px;
          font-weight: 500;
          color: var(--color-d2emu-text);
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
          line-height: 1.4;
        }
        .file-col-num { font-size: 13px; color: var(--color-d2emu-muted); justify-content: flex-end; font-weight: 600; }
        .file-col-diff { font-size: 13px; font-weight: 700; justify-content: flex-end; }
        .diff-del { color: var(--color-d2emu-bad); }
        .diff-same { color: var(--color-d2emu-muted); }
        .diff-grow { color: #c94a4a; }
        .diff-shrink { color: var(--color-d2emu-good); }

        .file-bulk-actions {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 12px 16px;
          border-top: 1px solid var(--color-d2emu-line);
          background: rgba(0,0,0,0.15);
          flex-wrap: wrap;
        }
        .file-bulk-select {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 14px;
          color: var(--color-d2emu-muted);
          cursor: pointer;
          user-select: none;
        }

        .recovery-detail-delete { margin-left: auto; }
        .safety-restore-note {
          margin: 0;
          padding: 0 20px 12px;
          color: var(--color-d2emu-muted);
          font-size: 15px;
          line-height: 1.5;
        }
        .safety-file-list {
          border-top: 1px solid var(--color-d2emu-line);
          max-height: 260px;
          overflow-y: auto;
        }
        .safety-select-all {
          display: flex;
          padding: 12px 20px;
          border-bottom: 1px solid var(--color-d2emu-line);
        }
        .safety-file-row {
          display: grid;
          grid-template-columns: 28px minmax(0, 1fr) auto;
          align-items: center;
          gap: 10px;
          padding: 9px 20px;
          border-bottom: 1px solid rgba(37,37,37,0.8);
          color: var(--color-d2emu-text);
          font-size: 15px;
          line-height: 1.4;
        }
        .safety-file-row > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

        .auto-restore-row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 14px 20px;
          gap: 14px;
          flex-wrap: wrap;
        }
        .auto-restore-info { display: flex; flex-direction: column; gap: 4px; }
        .auto-restore-op { font-size: 15px; font-weight: 700; color: var(--color-d2emu-gold); }
        .auto-restore-stash { font-size: 14px; color: var(--color-d2emu-muted); }

        /* Safety restore row */
        .safety-restore-row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 14px 20px;
          gap: 14px;
          flex-wrap: wrap;
        }
        .safety-restore-info { display: flex; flex-direction: column; gap: 4px; }
        .safety-restore-desc { font-size: 14px; color: var(--color-d2emu-muted); max-width: 360px; line-height: 1.5; }

        /* ── Responsive ── */
        @media (max-width: 1100px) {
          .recovery-console {
            grid-template-columns: minmax(220px, 1fr) minmax(300px, 2fr);
          }
          .backup-health-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
        }
        @media (max-width: 800px) {
          .recovery-console { grid-template-columns: 1fr; }
          .backup-controls-row { flex-direction: column; }
          .auto-save-panel, .retention-controls { min-width: 0; }
          .recovery-detail-title { font-size: 22px; }
        }
        @media (max-width: 560px) {
          .backup-page-header { flex-direction: column; align-items: flex-start; }
          .backup-header-actions { width: 100%; justify-content: flex-end; }
          .backup-health-strip { grid-template-columns: 1fr; }
          .auto-save-snapshot { align-items: flex-start; flex-wrap: wrap; }
          .auto-save-snapshot-actions { width: 100%; justify-content: flex-end; }
          .file-table-head, .file-row {
            grid-template-columns: 26px 22px 1fr 48px 48px 48px 62px;
            gap: 3px;
            padding: 6px 10px;
          }
          .recovery-detail-delete { margin-left: 0; }
        }
      `}</style>
    </div>
  )
}
