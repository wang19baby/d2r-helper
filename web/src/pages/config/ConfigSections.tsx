import { useState, type Dispatch } from 'react'
import { tauriInvoke } from '../../tauri'
import { showToast } from '../../components/Toast'
import Accordion from '../../components/Accordion'
import type { AppConfig, ModMeta } from '../../types'
import { useLocale } from '../../locales/context'
import type { Locale } from '../../locales/context'

const GOLD = 'var(--color-d2emu-gold)'
const TEXT = 'var(--color-d2emu-text)'
const MUTED = 'var(--color-d2emu-muted)'
const LINE = 'var(--color-d2emu-line)'

export interface ModLanguageSectionProps {
  config: AppConfig | null
  activeMod: string
  gameVersion: string
  linkEnabled: boolean
  modHasSavePath: boolean
  currentModMeta: ModMeta | undefined
  importRunning: boolean
  zhTwDiag: any
  searchFields: Record<string, boolean | null> | null
  changeMod: (mod: string) => void
  setGameVersion: (v: string) => void
  setLinkEnabled: (v: boolean) => void
  setZhTwDiag: (v: any) => void
  setConfig: React.Dispatch<React.SetStateAction<AppConfig | null>>
  applyConfig: (c: AppConfig) => void
  showToast: (msg: string, type?: 'success' | 'error' | 'warning' | 'info') => void
}

export interface ResourceImportSectionProps {
  config: AppConfig | null
  importRunning: boolean
  importProgress: { table_name: string; status: string; rows: number; elapsed_ms?: number }[]
  searchFields: Record<string, boolean | null> | null
  applyConfig: (c: AppConfig) => void
}

/* ═══════════════════════════════════════════════
   Mod & Language Accordion
   ═══════════════════════════════════════════════ */
export function ModLanguageSection({
  config, activeMod, gameVersion, linkEnabled, modHasSavePath, currentModMeta,
  importRunning, zhTwDiag, searchFields,
  changeMod, setGameVersion, setLinkEnabled, setZhTwDiag, setConfig, applyConfig, showToast,
}: ModLanguageSectionProps) {
  return (
    <div style={{ marginTop: 10 }}>
      <Accordion
        title="模组 & 语言"
        icon="fa-gamepad"
        defaultOpen
        badge={activeMod}
        searchMatch={searchFields?.modLang ?? undefined}
      >
        <div className="d2emu-field" style={{ marginBottom: 12 }}>
          <label>活跃模组</label>
          <div className="flex gap-2 items-stretch flex-wrap">
            <select className="flex-1 min-w-0" value={activeMod} onChange={e => changeMod(e.target.value)}
              disabled={!config?.available_mods?.length}>
              {config?.available_mods?.map((m, i) => {
                const meta = config?.mod_metadata?.[i]
                const label = meta?.version ? `${m} v${meta.version}` : m
                return <option key={m} value={m}>{label}</option>
              })}
            </select>
            <span className="d2emu-tag self-center" style={{ whiteSpace: 'nowrap' }}>
              {config?.game_data_path ? '已就绪' : '无数据'}
            </span>
          </div>
        </div>

        <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap', marginBottom: 12 }}>
          <div className="d2emu-field" style={{ flex: '1 1 160px', minWidth: 140 }}>
            <label>游戏版本</label>
            <select value={gameVersion} onChange={e => {
              setGameVersion(e.target.value)
              tauriInvoke('set_game_version', { gameVersion: e.target.value }).then(applyConfig).catch((err: any) => showToast(err, 'error'))
            }}>
              <option value="">自动检测</option>
              <option value="2.4">2.4</option>
              <option value="2.5">2.5</option>
              <option value="2.6">2.6</option>
              <option value="2.7">2.7</option>
              <option value="3.0">3.0 (D2R 2.0+)</option>
            </select>
          </div>
          <div className="d2emu-field" style={{ flex: '1 1 160px', minWidth: 140 }}>
            <label>物品名称语言</label>
            <select value={config?.language || 'enUS'} onChange={e => {
              tauriInvoke('set_language', { language: e.target.value }).then(applyConfig).catch((err: any) => showToast(err, 'error'))
            }}>
              <option value="enUS">English</option>
              <option value="zhCN">简体中文</option>
              <option value="zhTW">繁體中文</option>
              <option value="deDE">Deutsch</option>
              <option value="frFR">Français</option>
              <option value="koKR">한국어</option>
              <option value="plPL">Polski</option>
              <option value="ptBR">Português</option>
              <option value="ruRU">Русский</option>
              <option value="jaJP">日本語</option>
            </select>
          </div>
          <div className="d2emu-field" style={{ flex: '1 1 100px', minWidth: 100 }}>
            <label>界面语言</label>
            <LanguageSelector />
          </div>
          <div className="d2emu-field" style={{ flex: '0 0 110px' }}>
            <label>仓库格子</label>
            <select value={config?.stash_grid_size ?? 10} onChange={e => {
              const newSize = Number(e.target.value)
              if (config) setConfig({ ...config, stash_grid_size: newSize })
              tauriInvoke('set_stash_grid_size', { size: newSize }).catch((err: any) => showToast(err, 'error'))
            }}>
              <option value={10}>10×10</option>
              <option value={16}>16×16</option>
            </select>
          </div>
          <div className="d2emu-field" style={{ flex: '0 0 100px' }}>
            <label>背包格子</label>
            <select value={`${config?.backpack_cols ?? 10}×${config?.backpack_rows ?? 4}`} onChange={e => {
              const [c, r] = e.target.value.split('×').map(Number)
              if (config) setConfig({ ...config, backpack_cols: c, backpack_rows: r })
              tauriInvoke('set_grid_sizes', { backpackCols: c, backpackRows: r, cubeCols: config?.cube_cols ?? 10, cubeRows: config?.cube_rows ?? 10 })
                .then(applyConfig).catch((err: any) => showToast(err, 'error'))
            }}>
              <option value="10×4">10×4</option>
              <option value="16×16">16×16</option>
            </select>
          </div>
          <div className="d2emu-field" style={{ flex: '0 0 100px' }}>
            <label>盒子格子</label>
            <select value={`${config?.cube_cols ?? 10}×${config?.cube_rows ?? 10}`} onChange={e => {
              const [c, r] = e.target.value.split('×').map(Number)
              if (config) setConfig({ ...config, cube_cols: c, cube_rows: r })
              tauriInvoke('set_grid_sizes', { backpackCols: config?.backpack_cols ?? 10, backpackRows: config?.backpack_rows ?? 4, cubeCols: c, cubeRows: r })
                .then(applyConfig).catch((err: any) => showToast(err, 'error'))
            }}>
              <option value="3×4">3×4</option>
              <option value="10×10">10×10</option>
            </select>
          </div>
        </div>

        <div style={{
          marginBottom: 12, padding: '10px 12px',
          border: `1px solid ${modHasSavePath ? GOLD : LINE}`,
          borderRadius: 6,
          background: modHasSavePath ? 'rgba(201,163,74,0.05)' : 'transparent',
          opacity: modHasSavePath ? 1 : 0.5,
        }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: modHasSavePath ? 'pointer' : 'not-allowed' }}>
            <input type="checkbox" checked={linkEnabled}
              disabled={!modHasSavePath}
              onChange={e => setLinkEnabled(e.target.checked)}
              style={{ accentColor: GOLD }} />
            <span style={{ fontSize: 14, color: TEXT }}>切换模组时自动同步存档路径</span>
          </label>
          {modHasSavePath && currentModMeta?.save_path ? (
            <p style={{ fontSize: 14, color: MUTED, margin: '6px 0 0 24px' }}>
              已关联存档: <code style={{ color: GOLD, fontSize: 14 }}>{currentModMeta.save_path}</code>
              {linkEnabled && (
                <span style={{ color: GOLD, marginLeft: 8 }}>
                  <i className="fa-solid fa-link" /> 联动生效中
                </span>
              )}
            </p>
          ) : (
            <p style={{ fontSize: 14, color: MUTED, margin: '6px 0 0 24px' }}>
              当前模组未提供 savepath，联动不可用。
            </p>
          )}
        </div>

        {currentModMeta && (
          <div style={{ fontSize: 14, color: MUTED, marginBottom: 12 }}>
            <p>
              {[
                currentModMeta.author && `作者: ${currentModMeta.author}`,
                currentModMeta.description,
                currentModMeta.save_path && `存档路径: ${currentModMeta.save_path}`,
              ].filter(Boolean).join(' · ') || '（无额外信息）'}
            </p>
            {config?.game_data_path && (
              <p style={{ wordBreak: 'break-all', marginTop: 4 }}>
                数据源: {config.game_data_path}
              </p>
            )}
          </div>
        )}

        <div className="flex gap-2" style={{ marginTop: 8 }}>
          <button className="d2emu-btn d2emu-btn-sm" disabled={importRunning} onClick={async () => {
            let confirmed = false
            try {
              const { confirm } = await import('@tauri-apps/plugin-dialog')
              confirmed = await confirm('重新从游戏目录导入所有数据？已有数据将被覆盖。', { title: '重新导入', kind: 'warning' })
            } catch {
              confirmed = window.confirm('确定重新导入所有游戏数据吗？已有数据将被覆盖。')
            }
            if (!confirmed) return
            try {
              await tauriInvoke('reimport_game_data')
              showToast('后台导入已启动', 'success')
            } catch (e: any) { showToast(String(e), 'error') }
          }}>
            <i className={`fa-solid fa-rotate ${importRunning ? 'fa-spin' : ''}`} /> {importRunning ? '导入中...' : '重新导入'}
          </button>
          <button className="d2emu-btn d2emu-btn-sm d2emu-btn-ghost" onClick={async () => {
            try {
              const r = await tauriInvoke('diagnose_zh_tw') as any
              setZhTwDiag(r)
            } catch (e: any) { showToast(String(e), 'error') }
          }}>
            <i className="fa-solid fa-language" /> 翻译诊断
          </button>
        </div>
        {zhTwDiag &&
          <div style={{ marginTop: 12, fontSize: 14, color: TEXT }}>
            <p>{zhTwDiag.target_lang} 资源缺失率: <strong style={{ color: zhTwDiag.overall_missing_pct > 10 ? '#c9a34a' : '#3bbf4f' }}>
              {zhTwDiag.overall_missing_pct.toFixed(1)}%</strong>
              {' '}({zhTwDiag.namespaces.reduce((a: number, n: any) => a + n.missing, 0)} missing / {zhTwDiag.namespaces.reduce((a: number, n: any) => a + n.enus_count, 0)} enUS)
            </p>
            <table className="d2emu-table" style={{ width: '100%', fontSize: 14 }}>
              <thead><tr><th>Namespace</th><th>enUS</th><th>{zhTwDiag.target_lang}</th><th>缺失</th><th>缺失率</th></tr></thead>
              <tbody>
                {zhTwDiag.namespaces.map((n: any) => (
                  <tr key={n.namespace}>
                    <td>{n.namespace}</td>
                    <td>{n.enus_count}</td>
                    <td>{n.lang_count}</td>
                    <td style={{ color: n.missing > 0 ? '#c9a34a' : '#3bbf4f' }}>{n.missing}</td>
                    <td>{n.missing_pct.toFixed(1)}%</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        }
      </Accordion>
    </div>
  )
}

/* ═══════════════════════════════════════════════
   Resource Import Accordion
   ═══════════════════════════════════════════════ */
export function ResourceImportSection({
  config, importRunning, importProgress, searchFields, applyConfig,
}: ResourceImportSectionProps) {
  return (
    <div style={{ marginTop: 10 }}>
      <Accordion
        title="资源 & 导入"
        icon="fa-download"
        badge={importRunning ? '导入中' : undefined}
        searchMatch={searchFields?.resource ?? undefined}
      >
        {config?.game_data_path && (
          <div style={{ fontSize: 14, color: MUTED, marginBottom: 12, wordBreak: 'break-all' }}>
            数据源: {config.game_data_path}
          </div>
        )}

        <div className="d2emu-field">
          <label>资源管理</label>
          <p style={{ fontSize: 14, color: MUTED, margin: '8px 0' }}>
            从游戏目录导入物品 / 词缀 / 符文 / 套装等数据。已有数据将被覆盖。
          </p>
          <button className="d2emu-btn d2emu-btn-sm" disabled={importRunning} onClick={async () => {
            let confirmed = false
            try {
              const { confirm } = await import('@tauri-apps/plugin-dialog')
              confirmed = await confirm('重新从游戏目录导入所有数据？已有数据将被覆盖。', { title: '重新导入', kind: 'warning' })
            } catch {
              confirmed = window.confirm('确定重新导入所有游戏数据吗？已有数据将被覆盖。')
            }
            if (!confirmed) return
            try {
              await tauriInvoke('reimport_game_data')
              showToast('后台导入已启动', 'success')
            } catch (e: any) { showToast(String(e), 'error') }
          }}>
            <i className={`fa-solid fa-rotate ${importRunning ? 'fa-spin' : ''}`} /> {importRunning ? '导入中...' : '重新导入'}
          </button>
        </div>

        {importProgress.length > 0 && (
          <div style={{ marginTop: 12 }}>
            <table className="d2emu-table" style={{ width: '100%', fontSize: 14 }}>
              <thead><tr><th>表</th><th>状态</th><th>行数</th><th>耗时</th></tr></thead>
              <tbody>
                {importProgress.map((t, i) => (
                  <tr key={i}>
                    <td>{t.table_name}</td>
                    <td style={{ color: t.status === 'done' ? '#3bbf4f' : t.status === 'error' ? '#c94a4a' : '#c9a34a' }}>{t.status}</td>
                    <td>{t.rows}</td>
                    <td>{t.elapsed_ms ? `${(t.elapsed_ms / 1000).toFixed(1)}s` : '-'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Accordion>
    </div>
  )
}

function LanguageSelector() {
  const { locale, setLocale } = useLocale()
  const locales: { value: Locale; label: string }[] = [
    { value: 'zhCN', label: '简体中文' },
    { value: 'enUS', label: 'English' },
  ]
  return (
    <select value={locale} onChange={e => setLocale(e.target.value as Locale)}>
      {locales.map(l => <option key={l.value} value={l.value}>{l.label}</option>)}
    </select>
  )
}
