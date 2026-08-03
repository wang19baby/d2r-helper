import { useEffect, useState, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import { tauriInvoke } from '../tauri'
import { showToast } from '../components/Toast'
import { useImportProgress } from '../hooks/useImportProgress'
import type { AppConfig, ModMeta } from '../types'
import ConfigHeader from './config/ConfigHeader'
import ConfigStatusBar from './config/ConfigStatusBar'
import GameDirectorySection from './config/GameDirectorySection'
import { ModLanguageSection, ResourceImportSection } from './config/ConfigSections'

export default function Config() {
  const navigate = useNavigate()
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [folder, setFolder] = useState('')
  const [gameRoot, setGameRoot] = useState('')
  const [activeMod, setActiveMod] = useState('(原版)')
  const [gameVersion, setGameVersion] = useState('')
  const [zhTwDiag, setZhTwDiag] = useState<any>(null)
  const { running: importRunning, tables: importProgress } = useImportProgress({
    onComplete: () => load(),
  })
  const [linkEnabled, setLinkEnabled] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')

  const applyConfig = (c: AppConfig) => {
    setConfig(c)
    setFolder(c.save_folder || c.default_folder || '')
    setGameRoot(c.game_root || '')
    setActiveMod(c.active_mod || '(原版)')
    setGameVersion(c.game_version || '')
  }

  const load = async () => {
    try {
      const c = await tauriInvoke('get_app_config') as AppConfig
      applyConfig(c)
    } catch (e: unknown) { showToast(e instanceof Error ? e.message : '加载配置失败', 'error') }
  }
  useEffect(() => { load() }, [])

  const currentModMeta: ModMeta | undefined = useMemo(() => {
    if (!config?.mod_metadata) return undefined
    return config.mod_metadata.find((_, i) => config.available_mods[i] === activeMod)
  }, [config, activeMod])

  const searchFields = useMemo(() => {
    const q = searchQuery.toLowerCase().trim()
    if (!q) return null
    const matchAny = (keywords: string, q: string) =>
      keywords.split(' ').some(kw => q.includes(kw))
    return {
      gameDir: matchAny('游戏 目录 安装 路径 浏览', q) || q.includes('游戏'),
      modLang: matchAny('模组 语言 联动 版本 仓库 格子 同步 存档', q),
      resource: matchAny('资源 画像 导入 翻译 诊断 进度', q),
    }
  }, [searchQuery])

  const browseFolder = async () => {
    try {
      const defaultPath = config?.default_folder || folder || undefined
      const path = await tauriInvoke('plugin:dialog|open', {
        options: { directory: true, multiple: false, title: '选择 D2R 存档文件夹', defaultPath },
      })
      if (typeof path === 'string' && path) {
        setFolder(path)
        if (linkEnabled && currentModMeta?.save_path) {
          showToast('已取消联动（存档路径被手动修改）', 'warning')
          setLinkEnabled(false)
        }
        if (config) setConfig({ ...config, save_folder: path, default_folder: path })
        await tauriInvoke('update_save_folder', { saveFolder: path })
        showToast('存档路径已保存！', 'success')
      }
    } catch { showToast('对话框不可用。', 'warning') }
  }

  const browseGameRoot = async () => {
    try {
      const path = await tauriInvoke('plugin:dialog|open', {
        options: { directory: true, multiple: false, title: '选择 Diablo II Resurrected 游戏目录' },
      })
      if (typeof path === 'string' && path) {
        setGameRoot(path)
        try {
          const res = await tauriInvoke('set_game_root', { gameRoot: path }) as AppConfig
          setConfig(res)
          setActiveMod(res.active_mod || '(原版)')
          showToast(`已检测到 ${res.available_mods.length - 1} 个模组！`, 'success')
        } catch (e: any) {
          showToast(e.message || '目录无效', 'error')
        }
      }
    } catch { showToast('对话框不可用。', 'warning') }
  }

  const changeMod = async (mod: string) => {
    setActiveMod(mod)
    try {
      const res = await tauriInvoke('set_active_mod', { activeMod: mod }) as AppConfig
      setConfig(res)
      setGameVersion(res.game_version || '')
      const modMeta = res.mod_metadata?.find((_, i) => res.available_mods[i] === mod)
      if (linkEnabled && modMeta?.save_path) {
        const resolvedPath = modMeta.save_path.includes(':')
          ? modMeta.save_path
          : gameRoot
            ? `${gameRoot.replace(/\\$/, '')}\\${modMeta.save_path.replace(/^[\\/]/, '').replace(/[\\/]/g, '\\')}`
            : modMeta.save_path
        setFolder(resolvedPath)
        setConfig({ ...res, save_folder: resolvedPath })
        try {
          await tauriInvoke('update_save_folder', { saveFolder: resolvedPath })
          showToast(`已联动存档路径至: ${mod}`, 'success')
        } catch { /* ignore */ }
      }
    } catch (e: any) { showToast(e.message, 'error') }
  }

  const modHasSavePath = !!currentModMeta?.save_path

  return (
    <div className="font-d2emu-ui" style={{ margin: '0 auto' }}>
      <ConfigHeader
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        onSearchClear={() => setSearchQuery('')}
      />
      <ConfigStatusBar
        gameRoot={gameRoot}
        configGameDataPath={config?.game_data_path}
        activeMod={activeMod}
        currentModMeta={currentModMeta}
        folder={folder}
        linkEnabled={linkEnabled}
        modHasSavePath={modHasSavePath}
      />
      <GameDirectorySection
        gameRoot={gameRoot}
        searchMatch={searchFields?.gameDir ?? undefined}
        setGameRoot={setGameRoot}
        browseGameRoot={browseGameRoot}
        availableModsCount={config && gameRoot ? config.available_mods.length - 1 : undefined}
      />
      <ModLanguageSection
        config={config}
        activeMod={activeMod}
        gameVersion={gameVersion}
        linkEnabled={linkEnabled}
        modHasSavePath={modHasSavePath}
        currentModMeta={currentModMeta}
        importRunning={importRunning}
        zhTwDiag={zhTwDiag}
        searchFields={searchFields}
        changeMod={changeMod}
        setGameVersion={setGameVersion}
        setLinkEnabled={setLinkEnabled}
        setZhTwDiag={setZhTwDiag}
        setConfig={setConfig}
        applyConfig={applyConfig}
        showToast={showToast}
      />
      <ResourceImportSection
        config={config}
        importRunning={importRunning}
        importProgress={importProgress}
        searchFields={searchFields}
        applyConfig={applyConfig}
      />


      {/* 存档文件夹 */}
      <div style={{ marginTop: 12 }}>
        <div style={{
          padding: '14px 16px', borderRadius: 8,
          border: '1px solid var(--color-d2emu-line)',
          background: 'rgba(255,255,255,0.03)',
        }}>
          <label style={{ fontSize: 14, fontWeight: 600, color: 'var(--color-d2emu-text)', marginBottom: 8, display: 'block' }}>
            D2R 存档文件夹
          </label>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
            <input className="flex-1 min-w-0"
              style={{ minWidth: 200, flex: '1 1 300px' }}
              value={folder} onChange={e => { setFolder(e.target.value); setLinkEnabled(false) }}
              placeholder="C:\Users\用户名\Saved Games\Diablo II Resurrected" />
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={browseFolder}>
              <i className="fa-solid fa-folder-open" /> 浏览
            </button>
            <button className="d2emu-btn d2emu-btn-primary d2emu-btn-sm" onClick={async () => {
              try {
                await tauriInvoke('update_save_folder', { saveFolder: folder })
                if (config) setConfig({ ...config, save_folder: folder, default_folder: folder })
                showToast('已保存', 'success')
              } catch (e: any) { showToast(String(e), 'error') }
            }}>
              <i className="fa-solid fa-floppy-disk" /> 保存
            </button>
          </div>
          {linkEnabled && modHasSavePath && currentModMeta?.save_path && (
            <p style={{ fontSize: 13, color: 'var(--color-d2emu-muted)', marginTop: 6 }}>
              已联动至模组存档路径: <code style={{ color: 'var(--color-d2emu-gold)' }}>{currentModMeta.save_path}</code>
              <span style={{ color: 'var(--color-d2emu-gold)', cursor: 'pointer', marginLeft: 8 }} onClick={() => setLinkEnabled(false)}>
                <i className="fa-solid fa-unlink" /> 取消联动
              </span>
            </p>
          )}
        </div>
      </div>
      {/* Link to Backup page */}
      <div style={{ marginTop: 12 }}>
        <div
          onClick={() => navigate('/backup')}
          style={{
            display: 'flex', alignItems: 'center', gap: 10,
            padding: '14px 16px', borderRadius: 8,
            border: '1px solid var(--color-d2emu-line)',
            background: 'rgba(255,255,255,0.03)',
            cursor: 'pointer', transition: 'background 0.15s',
          }}
          onMouseEnter={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.06)')}
          onMouseLeave={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.03)')}
        >
          <i className="fa-solid fa-floppy-disk" style={{ fontSize: 20, color: 'var(--color-d2emu-gold)' }} />
          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 600, color: 'var(--color-d2emu-text)' }}>存档备份</div>
            <div style={{ fontSize: 13, color: 'var(--color-d2emu-muted)' }}>
              管理自动备份和手动备份，恢复角色或仓库存档
            </div>
          </div>
          <i className="fa-solid fa-chevron-right" style={{ color: 'var(--color-d2emu-muted)' }} />
        </div>
      </div>
    </div>
  )
}
