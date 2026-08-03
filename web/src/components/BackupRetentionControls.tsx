import { useState, useEffect } from 'react'

interface Props {
  retentionDays: number
  archiving: boolean
  cleaningAuto: boolean
  onRetentionChange: (d: number) => void
  onArchive: () => void
  onCleanup: () => void
}

export default function BackupRetentionControls({
  retentionDays, archiving, cleaningAuto,
  onRetentionChange, onArchive, onCleanup,
}: Props) {
  const [localDays, setLocalDays] = useState(retentionDays)

  useEffect(() => {
    setLocalDays(retentionDays)
  }, [retentionDays])

  const commit = (v: number) => {
    const d = Math.min(365, Math.max(1, v))
    setLocalDays(d)
    onRetentionChange(d)
  }

  return (
    <div className="retention-controls">
      <div className="retention-row">
        <label className="retention-label" htmlFor="retention-days">保留天数</label>
        <input
          id="retention-days"
          type="number" min={1} max={365}
          className="retention-input"
          value={localDays}
          onChange={e => setLocalDays(Number(e.target.value))}
          onBlur={() => commit(localDays)}
          onKeyDown={e => { if (e.key === 'Enter') commit(localDays) }}
          aria-label="自动备份保留天数"
        />
        <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
          disabled={archiving} onClick={onArchive} aria-label="归档旧备份">
          <i className="fa-solid fa-file-zipper" aria-hidden="true" />
          {archiving ? '归档中…' : '归档'}
        </button>
        <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
          disabled={cleaningAuto} onClick={onCleanup} aria-label="清理旧备份"
          title={`仅清理超过 ${Math.max(7, localDays)} 天的自动备份`}>
          <i className="fa-solid fa-broom" aria-hidden="true" />
          {cleaningAuto ? '清理中…' : '清理'}
        </button>
      </div>
    </div>
  )
}
