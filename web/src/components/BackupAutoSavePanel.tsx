import type { AutoSaveInfo } from '../types'
import { formatSize, relativeTime } from '../utils/backupHelpers'

interface Props {
  autoSaveInfo: AutoSaveInfo | null
  triggering: boolean
  timerOn: boolean
  restoringSnapshot: boolean
  onTrigger: () => void
  onStart: () => void
  onStop: () => void
  onRestoreSnapshot: () => void
}

export default function BackupAutoSavePanel({
  autoSaveInfo, triggering, timerOn, restoringSnapshot,
  onTrigger, onStart, onStop, onRestoreSnapshot,
}: Props) {
  const hasSnapshot = autoSaveInfo && autoSaveInfo.file_count > 0

  return (
    <div className="auto-save-panel">
      <div className="auto-save-label">
        <i className="fa-solid fa-clock" aria-hidden="true" />
        自动快照
      </div>

      {hasSnapshot ? (
        <div className="auto-save-snapshot">
          <div className="auto-save-snapshot-info">
            <span className="auto-save-snapshot-files">{autoSaveInfo!.file_count} 个文件</span>
            <span className="auto-save-snapshot-size">{formatSize(autoSaveInfo!.total_size)}</span>
            <span className="auto-save-snapshot-time">{relativeTime(autoSaveInfo!.timestamp)}</span>
          </div>
          <div className="auto-save-snapshot-actions">
            <span className={`auto-save-status-badge ${timerOn ? 'is-active' : 'is-inactive'}`}
              aria-label={timerOn ? '自动快照运行中' : '自动快照已停止'}>
              <i className="fa-solid fa-circle" aria-hidden="true" /> {timerOn ? '运行中' : '已停止'}
            </span>
            <button className="d2emu-btn d2emu-btn-danger d2emu-btn-sm"
              disabled={restoringSnapshot} onClick={onRestoreSnapshot} aria-label="恢复最近自动快照">
              {restoringSnapshot ? '恢复中…' : '恢复快照'}
            </button>
          </div>
        </div>
      ) : (
        <div className="auto-save-snapshot is-empty">
          <span className="auto-save-empty-text">等待首次保存…</span>
          <span className={`auto-save-status-badge ${timerOn ? 'is-active' : 'is-inactive'}`}
            aria-label={timerOn ? '自动快照运行中' : '自动快照已停止'}>
            <i className="fa-solid fa-circle" aria-hidden="true" /> {timerOn ? '运行中' : '已停止'}
          </span>
        </div>
      )}

      <div className="auto-save-controls">
        <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
          disabled={triggering} onClick={onTrigger}
          aria-label="立即触发自动保存">
          <i className="fa-solid fa-bolt" aria-hidden="true" />
          {triggering ? '触发中…' : '立即触发'}
        </button>
        {timerOn ? (
          <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
            onClick={onStop} aria-label="停止自动保存定时器">
            <i className="fa-solid fa-pause" aria-hidden="true" /> 停止
          </button>
        ) : (
          <button className="d2emu-btn d2emu-btn-action d2emu-btn-sm"
            onClick={onStart} aria-label="启动自动保存定时器">
            <i className="fa-solid fa-play" aria-hidden="true" /> 启动
          </button>
        )}
      </div>
    </div>
  )
}
