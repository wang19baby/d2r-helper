import type { BackupEntry, AutoBackupEntry, SafetyBackupEntry } from '../types'

/* ─── Design tokens (CSS vars) ──────────────────────────────── */
export const GOLD = 'var(--color-d2emu-gold)'
export const MUTED = 'var(--color-d2emu-muted)'
export const BAD = 'var(--color-d2emu-bad)'

/* ─── Helpers ──────────────────────────────────────────────────── */
export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`
}

export function relativeTime(ts: string): string {
  const m = ts.match(/(\d{4})-?(\d{2})-?(\d{2})[ _]?(\d{2})-?(\d{2})-?(\d{2})/)
  if (!m) return ts
  const d = new Date(+m[1], +m[2] - 1, +m[3], +m[4], +m[5], +m[6])
  const diff = Date.now() - d.getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 1) return '刚刚'
  if (mins < 60) return `${mins} 分钟前`
  const h = Math.floor(mins / 60)
  if (h < 24) return `${h} 小时前`
  const days = Math.floor(h / 24)
  if (days < 7) return `${days} 天前`
  return `${m[1]}/${m[2]}/${m[3]}`
}

export function formatTs(ts: string): string {
  return ts.replace(/[ _]/g, ' ')
}

export function operationLabel(operation: string): string {
  if (operation === 'deposit') return '存入'
  if (operation === 'restore') return '恢复'
  return '取回'
}

export function diffStr(backup: number, current: number | null): { text: string; cls: string; label: string } {
  if (current === null) return { text: '已删除', cls: 'diff-del', label: '文件已从当前存档中删除' }
  if (backup === current) return { text: '无变化', cls: 'diff-same', label: '备份与当前大小相同' }
  const diff = (current as number) - backup
  const sign = diff > 0 ? '+' : ''
  const label = diff > 0 ? `当前比备份大 ${formatSize(Math.abs(diff))}` : `当前比备份小 ${formatSize(Math.abs(diff))}`
  return { text: `${sign}${formatSize(Math.abs(diff))}`, cls: diff > 0 ? 'diff-grow' : 'diff-shrink', label }
}

/* ─── Types ────────────────────────────────────────────────────── */
export type RestoreTarget =
  | { kind: 'snapshot' }
  | { kind: 'manual'; ts: string; files?: string[] }
  | { kind: 'auto'; filename: string }
  | { kind: 'safety'; dirname: string; files?: string[] }

export interface ConfirmDialog {
  title: string
  body: React.ReactNode
  danger?: boolean
  target: RestoreTarget
}

export type UnifiedEntry =
  | { id: string; kind: 'manual'; ts: string; label: string; subtitle: string; searchText: string; size: number; fileCount: number }
  | { id: string; kind: 'auto'; ts: string; label: string; subtitle: string; searchText: string; size: number; filename: string; operation: string }
  | { id: string; kind: 'safety'; ts: string; label: string; subtitle: string; searchText: string; size: number; dirname: string; fileCount: number }

export function buildUnifiedList(
  backups: BackupEntry[], autoBackups: AutoBackupEntry[], safetyBackups: SafetyBackupEntry[]
): UnifiedEntry[] {
  const manual: UnifiedEntry[] = backups.map(b => ({
    id: `manual-${b.timestamp}`, kind: 'manual' as const,
    ts: b.timestamp, label: formatTs(b.timestamp),
    subtitle: `${b.files.length} 个文件 · ${relativeTime(b.timestamp)}`,
    searchText: [b.timestamp, ...b.files.map(f => f.filename)].join(' '),
    size: b.total_size,
    fileCount: b.files.length,
  }))
  const auto: UnifiedEntry[] = autoBackups.map(a => ({
    id: `auto-${a.filename}`, kind: 'auto' as const,
    ts: a.timestamp, label: formatTs(a.timestamp),
    subtitle: `${operationLabel(a.operation)} · ${a.original_stash}`,
    searchText: `${a.timestamp} ${a.filename} ${a.original_stash} ${a.operation}`,
    size: a.size, filename: a.filename, operation: a.operation,
  }))
  const safety: UnifiedEntry[] = safetyBackups.map(s => ({
    id: `safety-${s.dirname}`, kind: 'safety' as const,
    ts: s.timestamp, label: formatTs(s.timestamp),
    subtitle: `${s.file_count} 个文件`,
    searchText: [s.timestamp, s.dirname, ...s.files].join(' '),
    size: s.total_size, dirname: s.dirname, fileCount: s.file_count,
  }))
  return [...manual, ...auto, ...safety].sort((a, b) => b.ts.localeCompare(a.ts))
}

/* ─── RestoreImpact component ──────────────────────────────────── */
export function RestoreImpact({ files, changedCount }: { files: string[]; changedCount?: number }) {
  return (
    <div className="restore-impact">
      <div className="restore-impact-summary">
        将覆盖 <strong>{files.length}</strong> 个文件
        {typeof changedCount === 'number' && <span> · 其中 {changedCount} 个与当前存档不同</span>}
      </div>
      {files.length > 0 && (
        <ul className="restore-impact-files">
          {files.slice(0, 5).map(file => <li key={file}>{file}</li>)}
          {files.length > 5 && <li>以及另外 {files.length - 5} 个文件</li>}
        </ul>
      )}
    </div>
  )
}
