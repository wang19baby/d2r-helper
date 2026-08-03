import StatusWidget from '../../components/StatusWidget'
import type { ModMeta } from '../../types'
import type { CSSProperties } from 'react'

const GOLD = 'var(--color-d2emu-gold)'
const MUTED = 'var(--color-d2emu-muted)'
const LINE = 'var(--color-d2emu-line)'

interface Props {
  gameRoot: string
  configGameDataPath?: string | null
  activeMod: string
  currentModMeta?: ModMeta | null
  folder: string
  linkEnabled: boolean
  modHasSavePath: boolean
}

/**
 * ConfigStatusBar — 3 个状态 Widget + 系统信息栏
 */
export default function ConfigStatusBar({
  gameRoot, configGameDataPath, activeMod, currentModMeta,
  folder, linkEnabled, modHasSavePath,
}: Props) {
  return (
    <>
      {/* Status Widgets */}
      <div style={{ display: 'flex', gap: 10, marginBottom: 12, flexWrap: 'wrap' }}>
        <StatusWidget
          icon="fa-folder-tree"
          label="游戏"
          status={gameRoot ? 'ready' : 'error'}
          statusText={gameRoot ? '已配置' : '未配置'}
          onClick={() => document.querySelector<HTMLDetailsElement>('#acc-game')?.toggleAttribute?.('open', true)}
        >
          {gameRoot ? (
            <span style={{ wordBreak: 'break-all', fontSize: 14 }}>{gameRoot}</span>
          ) : (
            <span style={{ color: MUTED, fontSize: 14 }}>点击下方设置游戏目录</span>
          )}
        </StatusWidget>
        <StatusWidget
          icon="fa-gamepad"
          label="模组"
          status={configGameDataPath ? 'ready' : 'warn'}
          statusText={configGameDataPath ? '已就绪' : '无数据'}
        >
          <span>
            {activeMod}
            {currentModMeta?.version ? ` v${currentModMeta.version}` : ''}
          </span>
        </StatusWidget>
        <StatusWidget
          icon="fa-floppy-disk"
          label="存档"
          status={folder ? 'ready' : 'error'}
          statusText={linkEnabled && modHasSavePath ? '联动中' : (folder ? '已设置' : '未设置')}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
            <span style={{ fontSize: 14, wordBreak: 'break-all' }}>
              {folder || '（未设置）'}
            </span>
            {linkEnabled && modHasSavePath && (
              <span style={{ fontSize: 14, color: GOLD, padding: '1px 6px', borderRadius: 8, background: 'rgba(201,163,74,0.12)' }}>
                <i className="fa-solid fa-link" /> 联动
              </span>
            )}
          </div>
        </StatusWidget>
      </div>

      {/* System info footer */}
      <div style={{
        display: 'flex', gap: 16, alignItems: 'center', marginBottom: 16,
        padding: '6px 14px', fontSize: 14, color: MUTED,
        border: `1px solid ${LINE}`, borderRadius: 6,
        background: 'rgba(255,255,255,0.015)',
      } as CSSProperties}>
        <span><i className="fa-solid fa-circle" style={{ fontSize: 7, color: '#3bbf4f', marginRight: 4 }} /> 数据库运行中</span>
        <span style={{ color: LINE }}>|</span>
        <span>版本 0.1.0</span>
      </div>
    </>
  )
}
