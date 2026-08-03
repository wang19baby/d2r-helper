/** 把 SQLite CURRENT_TIMESTAMP 字符串(UTC)解析为本地 Date */
export function parseDate(s: string | null | undefined): Date | null {
  if (!s) return null
  const norm = s.replace(' ', 'T') + 'Z'
  const d = new Date(norm)
  return isNaN(d.getTime()) ? null : d
}

/**
 * 相对时间格式化: "刚刚" / "5分钟前" / "3小时前" / "2天前"
 * 7 天以上显示完整日期。
 */
export function fmtTime(s: string | null | undefined): string {
  const d = parseDate(s)
  if (!d) return '—'
  const diff = Date.now() - d.getTime()
  const min = Math.floor(diff / 60000)
  if (min < 1) return '刚刚'
  if (min < 60) return `${min}分钟前`
  if (min < 1440) return `${Math.floor(min / 60)}小时前`
  if (min < 1440 * 7) return `${Math.floor(min / 1440)}天前`
  return d.toLocaleDateString('zh-CN') + ' ' + d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}
