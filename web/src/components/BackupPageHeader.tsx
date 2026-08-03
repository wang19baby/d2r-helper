import type { ModMeta } from '../types'

interface Props {
  modName: string
  pathOk: boolean
  folder: string
  meta: ModMeta | undefined
  creatingBackup: boolean
  refreshing: boolean
  onCreateBackup: () => void
  onRefresh: () => void
}

export default function BackupPageHeader({
  modName, pathOk, folder, meta,
  creatingBackup, refreshing, onCreateBackup, onRefresh,
}: Props) {
  return (
    <div className="backup-page-header">
      <div className="backup-header-identity">
        <div className="backup-header-mod">
          <span className="backup-header-mod-label">当前 Mod</span>
          <span className="backup-header-mod-value">{modName}</span>
        </div>
        <div className="backup-header-path">
          <span className="backup-header-path-label">存档路径</span>
          <span className={`backup-header-path-value ${pathOk ? 'is-ok' : 'is-warn'}`}
            title={pathOk ? folder : '路径未设置'}>
            <i className={`fa-solid ${pathOk ? 'fa-circle-check' : 'fa-triangle-exclamation'}`} aria-hidden="true" />
            <span className="backup-header-path-text">{folder || '未设置'}</span>
          </span>
        </div>
        {meta && (
          <span className="backup-header-meta-tag">v{meta.version}</span>
        )}
      </div>

      <div className="backup-header-actions">
        <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
          disabled={refreshing} onClick={onRefresh} aria-label="刷新备份列表">
          <i className={`fa-solid fa-rotate${refreshing ? ' fa-spin' : ''}`} /> {refreshing ? '刷新中…' : '刷新'}
        </button>
        <button className="d2emu-btn d2emu-btn-primary d2emu-btn-sm"
          disabled={creatingBackup} onClick={onCreateBackup} aria-label="创建手动备份">
          <i className="fa-solid fa-box-archive" />
          {creatingBackup ? '备份中…' : '创建备份'}
        </button>
      </div>
    </div>
  )
}
