import { useEffect, useState, useCallback, useRef, useMemo } from 'react'
import { tauriInvoke } from '../tauri'
import { showToast } from '../components/Toast'
import CharacterPanel from '../components/CharacterPanel'
import DeadBanner from '../components/DeadBanner'
import type { CharacterInfo, CharacterBriefInfo } from '../types'
import { clearRunewordContextCache } from '../utils/runewordCache'
import { characterStore, getClassCache } from '../cache/characters'
import { warehouseStore } from '../cache/warehouse'
import { useLocale } from '../locales/context'
import { useCached } from '../cache/useCache'

const LS_KEY = 'd2r-last-character'
function loadSavedCharacter(): string | null {
  try { return localStorage.getItem(LS_KEY) } catch { return null }
}
function saveCharacter(name: string) {
  try { localStorage.setItem(LS_KEY, name) } catch { /\* noop \*/ }
}

// ── Unified phase states ──
export type CharPhase = 'idle' | 'loading' | 'ready'
type ListPhase = 'initial' | 'loading' | 'ready'

const CHAR_FILTER_KEY = 'd2r-char-filter-state'
interface CharFilterState {
  classFilter: string
  lifecycleFilter: 'all' | 'alive' | 'dead'
  hardcoreFilter: 'all' | 'normal' | 'hardcore'
}
function loadFilterState(): CharFilterState | null {
  try {
    const raw = localStorage.getItem(CHAR_FILTER_KEY)
    return raw ? (JSON.parse(raw) as CharFilterState) : null
  } catch { return null }
}
function saveFilterState(s: CharFilterState): void {
  try { localStorage.setItem(CHAR_FILTER_KEY, JSON.stringify(s)) } catch { /\* noop \*/ }
}

const initialFilter = loadFilterState()
export default function Characters() {
  // ── 持久状态: 页面配置、筛选、UI 控制 ──
  const { t } = useLocale()
  const [selectedChar, setSelectedChar] = useState<string | null>(null)
  const [saveFolder, setSaveFolder] = useState<string>(() => {
    try { return localStorage.getItem('d2r-save-folder') ?? '' } catch { return '' }
  })
  const [grid, setGrid] = useState({ backpackCols: 10, backpackRows: 4, cubeCols: 10, cubeRows: 10, stashCols: 16, stashRows: 16 })
  const [itemLanguage, setItemLanguage] = useState<string>('zhCN')
  const [pickerClassFilter, setPickerClassFilter] = useState<string>(initialFilter?.classFilter ?? 'all')
  const briefHashRef = useRef<Record<string, string>>({})
  const [pickerLifecycleFilter, setPickerLifecycleFilter] = useState<'all' | 'alive' | 'dead'>(initialFilter?.lifecycleFilter ?? 'all')
  const [pickerHardcoreFilter, setPickerHardcoreFilter] = useState<'all' | 'normal' | 'hardcore'>(initialFilter?.hardcoreFilter ?? 'all')
  const [changedNames, setChangedNames] = useState<string[]>([])
  const [extracting, setExtracting] = useState(false)
  const appConfigDone = useRef(false)
  const initialCharDone = useRef(false)

  // ── 1. 启动时获取 app 配置 (一次) ──
  useEffect(() => {
    if (appConfigDone.current) return
    appConfigDone.current = true
    tauriInvoke('get_app_config').then((cfg) => {
      const c = cfg as { save_folder?: string; default_folder?: string; language?: string; backpack_cols?: number; backpack_rows?: number; cube_cols?: number; cube_rows?: number; stash_grid_size?: number }
      const dir = c.save_folder || c.default_folder
      if (dir) {
        setSaveFolder(dir)
        try { localStorage.setItem('d2r-save-folder', dir) } catch { /\* noop \*/ }
      }
      setItemLanguage(c.language || 'zhCN')
      setGrid({
        backpackCols: c.backpack_cols ?? 10,
        backpackRows: c.backpack_rows ?? 4,
        cubeCols: c.cube_cols ?? 10,
        cubeRows: c.cube_rows ?? 10,
        stashCols: c.stash_grid_size ?? 10,
        stashRows: c.stash_grid_size ?? 10,
      })
    })
  }, [])

  // ── 2. 角色列表 via useCached ──
  const { data: _briefList, loading: listLoading, refresh: listRefresh, error: listError } = useCached({
    key: characterStore.listKey,
    loader: () => characterStore.getList({ dir: saveFolder }),
    enabled: !!saveFolder,
  })

  // ── 3. 同步 briefList → characters[] + class cache + hash + 角色选择 ──
  const [characters, setCharacters] = useState<string[]>([])
  useEffect(() => {
    if (!_briefList || _briefList.length === 0) return
    const briefList = _briefList
    const names = briefList.map(b => b.name)
    setCharacters(names)

    // 更新每个角色的 class cache (chip filter 需要)
    for (const b of briefList) {
      const existing = (getClassCache(b.name) ?? {}) as Record<string, unknown>
      const { is_dead: _stale, ...rest } = existing
      void _stale
      try {
        localStorage.setItem(`d2r-char-class-${b.name}`, JSON.stringify({
          ...rest,
          class_en: b.class_en,
          class_cn: b.class_cn,
          level: b.level,
          is_hardcore: b.is_hardcore,
          is_expansion: b.is_expansion,
          is_dead: b.is_dead,
          hash: existing.hash,
        }))
      } catch { /\* noop \*/ }
    }

    // 检测 file hash 变化
    const changed: string[] = []
    const hashMap: Record<string, string> = {}
    for (const b of briefList) {
      hashMap[b.name] = b.file_hash
      const cached = getClassCache(b.name)
      if (cached?.hash && cached.hash !== b.file_hash) {
        changed.push(b.name)
      }
    }
    briefHashRef.current = hashMap
    setChangedNames(changed)

    // 自动选中首个角色
    if (!initialCharDone.current) {
      initialCharDone.current = true
      const saved = loadSavedCharacter()
      const target = saved && names.includes(saved) ? saved : names[0]
      if (target) {
        setSelectedChar(target)
        saveCharacter(target)
      }
    }
  }, [_briefList])

  // ── 4. 角色完整数据 via useCached + event→Promise adapter ──
  const { data: character, loading: charLoading, refresh: charRefresh } = useCached({
    key: selectedChar ? characterStore.fullKey(selectedChar) : '',
    loader: () => {
      if (!selectedChar || !saveFolder) return Promise.reject(new Error('no char'))
      return characterStore.loadFull(selectedChar, saveFolder)
    },
    enabled: !!selectedChar && !!saveFolder,
    maxAgeMs: 5 * 60_000,
  })

  // ── 5. 派生状态 ──
  const listStatus: ListPhase = !saveFolder ? 'initial' : listLoading ? 'loading' : 'ready'
  const charStatus: CharPhase = !selectedChar ? 'idle' : charLoading ? 'loading' : character ? 'ready' : 'idle'

  // ── 6. displayCharacters memo (跟原来一样,用 getClassCache 代替 loadCharClassCache) ──
  const displayCharacters = useMemo(() => {
    if (pickerClassFilter === 'all' && pickerLifecycleFilter === 'all' && pickerHardcoreFilter === 'all') {
      return characters
    }
    const filtered = characters.filter(name => {
      const brief = (getClassCache(name) ?? {}) as Record<string, unknown>
      if (pickerClassFilter !== 'all' && brief.class_en !== pickerClassFilter) {
        if (Object.keys(brief).length > 0) return false
      }
      if (pickerLifecycleFilter === 'alive' && brief.is_dead === true) return false
      if (pickerLifecycleFilter === 'dead' && !brief.is_dead) return false
      if (pickerHardcoreFilter === 'hardcore' && !brief.is_hardcore) return false
      if (pickerHardcoreFilter === 'normal' && brief.is_hardcore) return false
      return true
    })
    if (selectedChar && !filtered.includes(selectedChar)) {
      return [selectedChar, ...filtered]
    }
    return filtered
  }, [characters, pickerClassFilter, pickerLifecycleFilter, pickerHardcoreFilter, selectedChar])

  function clearPickerFilters() {
    setPickerClassFilter('all')
    setPickerLifecycleFilter('all')
    setPickerHardcoreFilter('all')
  }

  // ── 7. Handler ──

  // 刷新列表 (替换 fetchCharacters)
  const handleRefresh = useCallback((clearCache?: boolean) => {
    if (clearCache) {
      // 清除所有 L2 角色全量缓存
      const keys: string[] = []
      for (let i = 0; i < localStorage.length; i++) {
        const k = localStorage.key(i)
        if (k?.startsWith('d2r-char-full-')) keys.push(k)
      }
      keys.forEach(k => localStorage.removeItem(k))
    }
    listRefresh(clearCache)
  }, [listRefresh])

  // 加载/重载角色 (替换 handleLoadCharacter)
  const handleLoadCharacter = useCallback(() => {
    if (selectedChar) charRefresh(true)
  }, [selectedChar, charRefresh])

  // 角色选择
  const selectCharacterByName = useCallback(async (_dir: string, name: string) => {
    setSelectedChar(name)
    saveCharacter(name)
  }, [])

  // 忽略文件变化提示
  const dismissCharChanged = useCallback((name: string) => {
    const hash = briefHashRef.current[name]
    if (!hash) return
    try {
      const existing = getClassCache(name) ?? ({} as Record<string, unknown>)
      localStorage.setItem(`d2r-char-class-${name}`, JSON.stringify({ ...existing, hash }))
    } catch { /\* noop \*/ }
    setChangedNames(prev => prev.filter(n => n !== name))
  }, [])

  // 装备提取到扩展仓库
  const extractEquip = useCallback(async () => {
    if (!saveFolder || !selectedChar) return
    setExtracting(true)
    try {
      const d2sPath = `${saveFolder}\\${selectedChar}.d2s`
      const res: Record<string, unknown> = await tauriInvoke('extract_character_equipment', { path: d2sPath })
      // 新物品已写入 SQLite, 仓库页缓存必须失效, 否则切过去看到的还是旧列表
      warehouseStore.invalidateAll()
      const count = Number(res?.extracted_count ?? 0)
      const equipped = Number(res?.equipped_count ?? 0)
      const backpack = Number(res?.backpack_count ?? 0)
      const belt = Number(res?.belt_count ?? 0)
      const skippedRaw = res?.skipped_items
      const skipped: { item_name: string; reason: string }[] = Array.isArray(skippedRaw)
        ? (skippedRaw as { item_name: string; reason: string }[])
        : []
      const pageName = typeof res?.page_name === 'string' ? res.page_name : '角色装备'
      const parts = [`已存入 ${count} 件`]
      if (equipped > 0) parts.push(`身上${equipped}`)
      if (backpack > 0) parts.push(`背包${backpack}`)
      if (belt > 0) parts.push(`腰带${belt}`)
      const detail = skipped.length > 0
        ? `（${skipped.length} 件未存入: ${skipped.slice(0, 5).map(s => s.item_name).join('、')}${skipped.length > 5 ? '…' : ''}）`
        : ''
      showToast(`${parts.join(' · ')} → 收藏页「${pageName}」${detail}`, 'success')
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      showToast(`提取失败: ${msg}`, 'error')
    } finally { setExtracting(false) }
  }, [saveFolder, selectedChar])


  // ── 效果: 切角色时清除符文上下文 ──
  useEffect(() => {
    clearRunewordContextCache()
  }, [selectedChar])

  // ── 效果: filter state 持久化到 L2 ──
  useEffect(() => {
    saveFilterState({ classFilter: pickerClassFilter, lifecycleFilter: pickerLifecycleFilter, hardcoreFilter: pickerHardcoreFilter })
  }, [pickerClassFilter, pickerLifecycleFilter, pickerHardcoreFilter])

  return (
    <div className="font-d2emu-ui flex flex-col" style={{ gap: 12, flexGrow: 1, flexShrink: 1, flexBasis: '0%', minHeight: 0, display: 'flex', flexDirection: 'column' }}>
      {/* ── CharacterPicker 过滤 chips (v2 §1.2 P0) ── 只在任一 filter 非 all 时显示 */}
      {(pickerClassFilter !== 'all' || pickerLifecycleFilter !== 'all' || pickerHardcoreFilter !== 'all') && (
        <section className="d2emu-card" style={{ padding: '10px 14px' }}>
          <div className="d2emu-tags" style={{ margin: 0, gap: 6, flexWrap: 'wrap' }}>
            <span style={{ color: 'var(--color-d2emu-muted, #aaa)', font: '600 13px/1 "Source Sans 3", sans-serif', textTransform: 'uppercase', letterSpacing: '0.06em', marginRight: 4 }}>
              <i className="fa-solid fa-filter" style={{ marginRight: 4 }} />
              {t('characters.filter_class')}
            </span>
            <button className={`d2emu-tag ${pickerClassFilter === 'all' ? 'd2emu-tag-active' : ''}`}
              style={{ cursor: 'pointer', borderStyle: 'solid' }}
              onClick={() => setPickerClassFilter('all')}>{t('characters.filter_all')}</button>
            {(['Amazon', 'Sorceress', 'Necromancer', 'Paladin', 'Barbarian', 'Druid', 'Assassin', 'Warlock'] as const).map(cls => (
              <button key={cls}
                className={`d2emu-tag ${pickerClassFilter === cls ? 'd2emu-tag-active' : ''}`}
                style={{ cursor: 'pointer', borderStyle: 'solid' }}
                onClick={() => setPickerClassFilter(cls)}>
                {cls}
              </button>
            ))}
            <span style={{ color: 'var(--color-d2emu-line)', margin: '0 4px' }}>·</span>
            <button className={`d2emu-tag ${pickerLifecycleFilter === 'all' ? 'd2emu-tag-active' : ''}`}
              style={{ cursor: 'pointer', borderStyle: 'solid' }}
              onClick={() => setPickerLifecycleFilter('all')}>{t('characters.filter_all')}</button>
            <button className={`d2emu-tag ${pickerLifecycleFilter === 'alive' ? 'd2emu-tag-active' : ''}`}
              style={{ cursor: 'pointer', borderStyle: 'solid' }}
              onClick={() => setPickerLifecycleFilter('alive')}>{t('characters.filter_alive')}</button>
            <button className={`d2emu-tag ${pickerLifecycleFilter === 'dead' ? 'd2emu-tag-active' : ''}`}
              style={{ cursor: 'pointer', borderStyle: 'solid' }}
              onClick={() => setPickerLifecycleFilter('dead')}>{t('characters.filter_dead')}</button>
            <span style={{ color: 'var(--color-d2emu-line)', margin: '0 4px' }}>·</span>
            <button className={`d2emu-tag ${pickerHardcoreFilter === 'all' ? 'd2emu-tag-active' : ''}`}
              style={{ cursor: 'pointer', borderStyle: 'solid' }}
              onClick={() => setPickerHardcoreFilter('all')}>{t('characters.filter_all')}</button>
            <button className={`d2emu-tag ${pickerHardcoreFilter === 'hardcore' ? 'd2emu-tag-active' : ''}`}
              style={{ cursor: 'pointer', borderStyle: 'solid' }}
              onClick={() => setPickerHardcoreFilter('hardcore')}>{t('characters.filter_hardcore')}</button>
            <button className={`d2emu-tag ${pickerHardcoreFilter === 'normal' ? 'd2emu-tag-active' : ''}`}
              style={{ cursor: 'pointer', borderStyle: 'solid' }}
              onClick={() => setPickerHardcoreFilter('normal')}>{t('characters.filter_normal')}</button>
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
              style={{ marginLeft: 'auto' }}
              onClick={clearPickerFilters}>
              <i className="fa-solid fa-xmark" /> 清除
            </button>
          </div>
        </section>
      )}

      {/* ── 列表加载错误 / 空目录提示 ── */}
      {listError ? (
        <div className="d2emu-card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 10, padding: '40px 20px', flex: 1 }}>
          <i className="fa-solid fa-triangle-exclamation" style={{ fontSize: 24, color: 'var(--color-d2emu-gold-dim, #b8963c)' }} />
          <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--color-d2emu-gold-dim, #b8963c)' }}>角色列表加载失败</div>
          <div style={{ fontSize: 14, color: '#999', textAlign: 'center', maxWidth: 400 }}>
            请检查设置中的存档路径是否正确。
            <br />
            <span style={{ fontSize: 12, color: '#777' }}>当前路径：{saveFolder || '未配置'}</span>
          </div>
          <button type="button" onClick={() => handleRefresh(true)} className="d2emu-btn d2emu-btn-ghost" style={{ marginTop: 8 }}>
            <i className="fa-solid fa-rotate-right" /> 重试
          </button>
        </div>
      ) : !saveFolder ? (
        <div className="d2emu-card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 10, padding: '40px 20px', flex: 1 }}>
          <i className="fa-solid fa-folder-open" style={{ fontSize: 24, color: 'var(--color-d2emu-muted, #666)' }} />
          <div style={{ fontSize: 16, fontWeight: 600, color: '#ccc' }}>未配置存档路径</div>
          <div style={{ fontSize: 14, color: '#999', textAlign: 'center', maxWidth: 400 }}>
            请在设置中配置正确的 D2R 存档目录，然后刷新角色列表。
          </div>
          <button type="button" onClick={() => window.location.href = '/config'} className="d2emu-btn d2emu-btn-ghost" style={{ marginTop: 8 }}>
            <i className="fa-solid fa-gear" /> 前往设置
          </button>
        </div>
      ) : !listLoading && characters.length === 0 ? (
        <div className="d2emu-card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 10, padding: '40px 20px', flex: 1 }}>
          <i className="fa-solid fa-users-slash" style={{ fontSize: 24, color: 'var(--color-d2emu-muted, #666)' }} />
          <div style={{ fontSize: 16, fontWeight: 600, color: '#ccc' }}>未找到角色存档</div>
          <div style={{ fontSize: 14, color: '#999', textAlign: 'center', maxWidth: 400 }}>
            存档目录中未找到 .d2s 角色文件。
            <br />
            <span style={{ fontSize: 12, color: '#777' }}>当前路径：{saveFolder}</span>
          </div>
          <button type="button" onClick={() => handleRefresh(true)} className="d2emu-btn d2emu-btn-ghost" style={{ marginTop: 8 }}>
            <i className="fa-solid fa-rotate-right" /> 刷新
          </button>
        </div>
      ) : null}
      {!listError && saveFolder && (listLoading || characters.length > 0) ? <CharacterPanel
        characters={displayCharacters}
        selectedChar={selectedChar}
        character={character}
        saveFolder={saveFolder}
        itemLanguage={itemLanguage}
        onSelectCharacter={selectCharacterByName}
        onRefresh={handleRefresh}
        onLoad={handleLoadCharacter}
        listStatus={listStatus}
        charStatus={charStatus}
        extracting={extracting}
        changedNames={changedNames}
        onDismissChanged={dismissCharChanged}
        onExtract={extractEquip}
        cubeCols={grid.cubeCols}
        cubeRows={grid.cubeRows}
        stashCols={grid.stashCols}
        stashRows={grid.stashRows}
        backpackCols={grid.backpackCols}
        backpackRows={grid.backpackRows}
      /> : null}

      {/* ── Death banner (v2 P0): selectedChar 已死 → 警告横幅 ── */}
      {selectedChar && (() => {
        const cached = getClassCache(selectedChar)
        const isDead = cached ? (cached as { is_dead?: boolean }).is_dead === true : false
        if (!isDead) return null
        return (
          <DeadBanner
            characterName={selectedChar}
            isHardcore={!!(cached as { is_hardcore?: boolean } | null)?.is_hardcore}
          />
        )
      })()}
    </div>
  )
}
