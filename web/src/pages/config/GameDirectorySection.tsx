import Accordion from '../../components/Accordion'

const GOLD = 'var(--color-d2emu-gold)'
const MUTED = 'var(--color-d2emu-muted)'

interface Props {
  gameRoot: string
  searchMatch?: boolean | null
  setGameRoot: (v: string) => void
  browseGameRoot: () => void
  availableModsCount?: number
}

/**
 * GameDirectorySection — 游戏目录配置手风琴
 */
export default function GameDirectorySection({
  gameRoot, searchMatch, setGameRoot, browseGameRoot, availableModsCount,
}: Props) {
  return (
    <div id="acc-game">
      <Accordion
        title="游戏目录"
        icon="fa-folder-tree"
        defaultOpen
        badge={gameRoot ? '已配置' : '未配置'}
        searchMatch={searchMatch ?? undefined}
      >
        <div className="d2emu-field">
          <label>游戏安装目录</label>
          <div className="flex gap-2 items-stretch">
            <input className="flex-1 min-w-0" value={gameRoot} onChange={e => setGameRoot(e.target.value)}
              placeholder="D:\games\Diablo II Resurrected" />
            <button className="d2emu-btn d2emu-btn-ghost" style={{ whiteSpace: 'nowrap' }} onClick={browseGameRoot}>
              <i className="fa-solid fa-folder-open" /> 浏览
            </button>
          </div>
          {!gameRoot && (
            <p style={{ fontSize: 14, color: MUTED, marginTop: 6 }}>
              设置游戏目录后，自动检测 mods/ 下的可用模组。
            </p>
          )}
          {gameRoot && availableModsCount != null && (
            <p style={{ fontSize: 14, color: MUTED, marginTop: 6 }}>
              已检测到 <strong style={{ color: GOLD }}>{availableModsCount}</strong> 个模组，
              请到「模组 & 语言」面板配置。
            </p>
          )}
        </div>
      </Accordion>
    </div>
  )
}
