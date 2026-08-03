/**
 * DeadBanner — 角色死亡横幅 (v2 P0)
 *
 * 沿用 .d2emu-lock-banner 装饰 (index.css 已存在)。
 *
 * Props:
 *  - characterName: 显示字符名
 *  - isHardcore: 是否专家模式 (HC 死亡不可逆)
 *  - killedAt: 死亡时间戳 (可选)
 *  - onArchive: 进入归档视图回调 (可选)
 */

import type { CSSProperties, JSX } from 'react'

export interface DeadBannerProps {
  characterName: string
  isHardcore?: boolean
  killedAt?: number | null
  onArchive?: () => void
}

const styles: CSSProperties = {
  margin: '0 0 12px',
}

export default function DeadBanner({
  characterName,
  isHardcore = false,
  killedAt,
  onArchive,
}: DeadBannerProps): JSX.Element {
  const killedAtStr = killedAt
    ? new Date(killedAt * 1000).toLocaleString('zh-CN', {
        hour12: false,
        year: 'numeric', month: '2-digit', day: '2-digit',
        hour: '2-digit', minute: '2-digit',
      })
    : null

  return (
    <div
      className="d2emu-lock-banner"
      role="alert"
      aria-live="assertive"
      style={styles}
    >
      <div className="d2emu-lock-banner-icon" aria-hidden="true">
        <i className="fa-solid fa-skull" />
      </div>
      <div className="d2emu-lock-banner-text">
        <strong>
          {isHardcore ? '专家模式死亡 · 不可逆' : '角色已死亡'}
          {characterName && <span style={{ color: '#fff', marginLeft: 6 }}>{characterName}</span>}
        </strong>
        <span>
          {isHardcore
            ? '该角色在专家模式下死亡,经验/装备/任务进度永久丢失。仅本工具读 d2s,不再写回。'
            : '该角色已死亡,无法执行任何操作(仅读)。可移入归档保留记录。'}
          {killedAtStr && (
            <>
              <span style={{ margin: '0 8px', opacity: 0.55 }}>·</span>
              <span style={{ fontFamily: '"JetBrains Mono", monospace' }}>死亡时间 {killedAtStr}</span>
            </>
          )}
        </span>
      </div>
      {onArchive && (
        <button
          className="d2emu-btn d2emu-btn-sm"
          style={{ borderColor: '#f2b84b', color: '#f2b84b' }}
          onClick={onArchive}
        >
          <i className="fa-solid fa-box-archive" style={{ marginRight: 6 }} />
          进入归档
        </button>
      )}
    </div>
  )
}
