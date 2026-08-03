import { useState, useMemo, useCallback, useEffect } from 'react'
import type { CharacterInfo, StashResult } from '../types'
import { tauriInvoke } from '../tauri'
import { stashStore } from '../cache/stash'
import { hasSkillTreeLayout } from '../utils/skillPresentation'
import SkillTreeDisplay from './SkillTree'
import EquipmentPanel, { type EquipSlot, type EquippedItem } from './EquipmentPanel'
import CharacterSkills from '../pages/characters/Skills'
import { CharacterWaypoints, CharacterQuests, CharacterRewards, CharacterMerc } from '../pages/characters/CharacterDetails'
import EquipmentDetailModal from './EquipmentDetailModal'
import EquipmentBonusTome from './EquipmentBonusTome'
import { PositionedItemGrid } from './CharacterGrids'
import D2EmuLoading from './D2EmuLoading'
import InventoryView from './InventoryView'
import StackablePageView from './StackablePageView'


type MicroTab = 'overview' | 'storage' | 'skills' | 'waypoints' | 'quests' | 'rewards' | 'merc' | 'warehouse'
const TABS: { key: MicroTab; label: string; icon: string }[] = [
  { key: 'overview',   label: '装备',    icon: 'fa-shield-halved' },
  { key: 'storage',    label: '存储',    icon: 'fa-box' },
  { key: 'skills',     label: '技能',    icon: 'fa-bolt' },
  { key: 'waypoints',  label: '小站',    icon: 'fa-location-dot' },
  { key: 'quests',     label: '任务',    icon: 'fa-list-check' },
  { key: 'rewards',    label: '奖励',    icon: 'fa-gift' },
  { key: 'merc',       label: '佣兵',    icon: 'fa-shield-halved' },
  { key: 'warehouse',  label: '共享仓库', icon: 'fa-warehouse' },
]

function fmtNum(n: number): string {
  return n.toLocaleString('en-US')
}

const CLASS_ICONS: Record<string, string> = {
  Amazon: 'fa-bow-arrow', Sorceress: 'fa-wand-magic-sparkles',
  Necromancer: 'fa-skull', Paladin: 'fa-cross',
  Barbarian: 'fa-axe', Druid: 'fa-paw',
  Assassin: 'fa-dagger', Warlock: 'fa-book-skull',
}

export interface CharacterPanelProps {
  characters: string[]
  selectedChar: string | null
  character: CharacterInfo | null
  saveFolder: string
  itemLanguage: string
  onSelectCharacter: (dir: string, name: string) => void
  onRefresh: (clearCache?: boolean) => void
  onLoad: () => void
  listStatus: ListPhase
  charStatus: CharPhase
  extracting: boolean
  onExtract: () => void
  changedNames?: string[]
  onDismissChanged?: (name: string) => void
  backpackCols?: number
  backpackRows?: number
  cubeCols?: number
  cubeRows?: number
  stashCols?: number
  stashRows?: number
}
type ListPhase = 'initial' | 'loading' | 'ready'
type CharPhase = 'idle' | 'loading' | 'ready'


const LABEL: React.CSSProperties = {
  color: 'var(--color-d2emu-muted, #888)', font: '600 14px/1 "Source Sans 3", sans-serif',
  letterSpacing: '0.08em', textTransform: 'uppercase', whiteSpace: 'nowrap',
}
const statBlockStyle: React.CSSProperties = {
  display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 3,
  padding: '8px 6px 6px', background: 'rgba(0,0,0,0.15)', borderRadius: 4,
  border: '1px solid var(--color-d2emu-line-soft, #1a1a1a)', minWidth: 0,
}

const statValueStyle: React.CSSProperties = {
  font: '700 20px/1 "Source Sans 3", sans-serif', fontVariantNumeric: 'tabular-nums',
  overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
}

function StatBlock({ label, value, color, variant }: { label: string; value: number | string; color?: string; variant?: 'gold'|'str'|'dex'|'ene'|'vit'|'life'|'mana'|'xp'|'sp' }) {
  return (
    <div className={`d2emu-statblock-tome${variant ? ' ' + variant : ''}`}>
      <span className="stat-label">{label}</span>
      <span className="stat-ornament">❦</span>
      <span className="stat-value" style={color ? { color } : undefined}>{value}</span>
    </div>
  )
}

function Portrait({ character, itemLanguage }: { character: CharacterInfo; itemLanguage: string }) {
  const cls = character.class_en
  const imgSrc = `/assets/img/characters/${cls.toLowerCase()}.png`
  const icon = CLASS_ICONS[cls] || 'fa-user'
  const clsName = itemLanguage === 'zhTW'
    ? (character.class_zh_tw || character.class_cn || character.class_en)
    : itemLanguage === 'enUS'
      ? character.class_en
      : (character.class_cn || character.class_en)
  const lv = character.level
  const hpPct = Math.min(100, (character.current_hp / Math.max(1, character.max_hp)) * 100)
  const mpPct = Math.min(100, (character.current_mana / Math.max(1, character.max_mana)) * 100)
  return (
    <div className="d2emu-portrait-arcane">
      <div className="d2emu-portrait-corners" style={{ position: 'relative' }}>
        <div className="d2emu-portrait-frame" style={{ overflow: 'hidden' }}>
          <img src={imgSrc} alt={cls}
            style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }}
            onError={(e) => { (e.target as HTMLImageElement).style.display = 'none' }} />
          <i className={`fa-solid ${icon}`} style={{ fontSize: 30, color: '#c7a032', position: 'absolute', inset: 0, margin: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: -1 }} />
          {character.is_hardcore && (
            <span style={{
              position: 'absolute', top: -3, right: -3,
              background: '#5c1010', color: '#ff6a6a',
              font: '700 9px/1 "JetBrains Mono", monospace', padding: '2px 5px', borderRadius: 2,
              border: '1px solid #8a2020',
              boxShadow: '0 0 4px rgba(192,26,26,0.6)',
            }}>HC</span>
          )}
        </div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'center', gap: 3, minWidth: 0 }}>
        <div className="d2emu-portrait-name">{character.name}</div>
        <div className="d2emu-portrait-class">等级 {lv} · {clsName}</div>
        <div className="d2emu-portrait-bars">
          <div className="d2emu-portrait-bar">
            <div className="d2emu-portrait-bar-track" style={{ color: '#c05858' }}>
              <div className="d2emu-portrait-bar-fill" style={{ width: `${hpPct}%`, background: 'linear-gradient(90deg, #c0392b, #e74c3c)' }} />
            </div>
            <span className="d2emu-portrait-bar-val" style={{ color: '#c05858' }}>{character.current_hp}</span>
          </div>
          <div className="d2emu-portrait-bar">
            <div className="d2emu-portrait-bar-track" style={{ color: '#7090e0' }}>
              <div className="d2emu-portrait-bar-fill" style={{ width: `${mpPct}%`, background: 'linear-gradient(90deg, #2980b9, #5dade2)' }} />
            </div>
            <span className="d2emu-portrait-bar-val" style={{ color: '#7090e0' }}>{character.current_mana}</span>
          </div>
        </div>
      </div>
    </div>
  )
}

/** 缓存角色职业信息 */
interface CharClassCache {
  class_en: string
  class_cn?: string
  level: number
  is_hardcore?: boolean
  is_expansion?: boolean
}
function getCachedClass(name: string): CharClassCache | null {
  try {
    const raw = localStorage.getItem(`d2r-char-class-${name}`)
    return raw ? (JSON.parse(raw) as CharClassCache) : null
  } catch { return null }
}

const CLASS_NAMES_CN: Record<string, string> = {
  Amazon: '亚马逊', Sorceress: '法师', Necromancer: '死灵法师',
  Paladin: '圣骑士', Barbarian: '野蛮人', Druid: '德鲁伊',
  Assassin: '刺客', Warlock: '术士',
}
function getClassName(en: string, cn?: string): string {
  return cn || CLASS_NAMES_CN[en] || en
}

/** 单个角色列表项 */
function CharListItem({
  name, selected, character, onClick, disabled,
  changed, onDismiss,
}: {
  name: string
  selected: boolean
  character: CharacterInfo | null
  onClick: () => void
  disabled?: boolean
  changed?: boolean
  onDismiss?: (name: string) => void
}) {
  const [hovered, setHovered] = useState(false)
  const cached = !character ? getCachedClass(name) : null
  const effective = character ?? cached
  const icon = effective ? (CLASS_ICONS[effective.class_en] || 'fa-user') : 'fa-user'
  const imgSrc = effective ? `/assets/img/characters/${effective.class_en.toLowerCase()}.png` : null
  const showLevel = (selected || hovered) && effective?.level != null
  const eff = effective as Record<string, unknown> | null
  const className = effective ? getClassName(effective.class_en, eff?.class_cn as string | undefined) : ''
  const tags = [eff?.is_expansion ? '资料片' : null, eff?.is_hardcore ? '专家' : null].filter(Boolean).join(' ')
  return (
    <div
      onClick={disabled ? undefined : onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => { if (e.key === 'Enter' && !disabled) onClick() }}
      className={`d2emu-char-list-item${selected ? ' is-selected' : ''}`}
    >
      {changed && (
        <div
          onClick={(e) => { e.stopPropagation(); onDismiss?.(name) }}
          title="此角色存档已更新，点击刷新缓存"
          style={{
            position: 'absolute', top: -2, right: -2, zIndex: 5,
            width: 16, height: 16, borderRadius: '50%',
            background: '#e8b84b', border: '2px solid #1a0f05',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            cursor: 'pointer',
          }}
        >
          <i className="fa-solid fa-refresh" style={{ fontSize: 9, color: '#1a0f05' }} />
        </div>
      )}
      <div style={{
        width: 36, height: 36, borderRadius: 6,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        background: selected ? 'rgba(251,177,58,0.1)' : 'rgba(255,255,255,0.03)',
        border: `1px solid ${selected ? 'rgba(251,177,58,0.2)' : 'rgba(255,255,255,0.06)'}`,
        flexShrink: 0,
      }}>
        {imgSrc ? (
          <img src={imgSrc} alt={effective!.class_en}
            style={{ width: 34, height: 34, borderRadius: 4, objectFit: 'cover', display: 'block' }}
            onError={(e) => { (e.target as HTMLImageElement).style.display = 'none' }} />
        ) : (
          <i className={`fa-solid ${icon}`} style={{ fontSize: 16, color: selected ? '#c7a032' : '#666' }} />
        )}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', minWidth: 0, flex: 1, gap: 1 }}>
        <div style={{
          width: '100%', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
          font: selected ? '600 14px/1.3 "Source Sans 3", sans-serif' : '400 14px/1.3 "Source Sans 3", sans-serif',
        }}>
          {name}
        </div>
        {showLevel && (
          <div style={{
            font: '400 11px/1.3 "Source Sans 3", sans-serif',
            color: 'rgba(251,177,58,0.5)',
            overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
          }}>
            {className} Lv.{effective!.level}{tags ? ' ' + tags : ''}
          </div>
        )}
      </div>
    </div>
  )
}

export default function CharacterPanel({
  characters, selectedChar, character, saveFolder,
  itemLanguage, onSelectCharacter, onRefresh,
  onLoad, listStatus, charStatus,
  extracting, onExtract,
  changedNames, onDismissChanged,
  cubeCols, cubeRows, stashCols, stashRows,
  backpackCols, backpackRows,
}: CharacterPanelProps) {
  const [tab, setTab] = useState<MicroTab>('overview')
  const [selectedEquip, setSelectedEquip] = useState<{ slot: EquipSlot; item: EquippedItem } | null>(null)

  // ── 共享仓库（d2i）数据 ──
  const [stashData, setStashData] = useState<StashResult | null>(null)
  const [stashPage, setStashPage] = useState(0)
  const [stashLoading, setStashLoading] = useState(false)

  const fetchStash = useCallback(async (forceRefresh?: boolean) => {
    if (!forceRefresh && stashData) {
      if (stashData.pages.length > 0) setStashPage(stashData.pages[0].index)
      return
    }
    setStashLoading(true)
    try {
      const res = await stashStore.fetch('shared', { force: !!forceRefresh })
      setStashData(res)
      if (res.pages.length > 0) setStashPage(res.pages[0].index)
    } catch (e) {
      setStashData(null)
    } finally {
      setStashLoading(false)
    }
  }, [stashData])

  useEffect(() => {
    if (tab === 'warehouse') fetchStash()
  }, [tab, fetchStash])

  const stashQualityToNum = (q: string): number | undefined => {
    const map: Record<string, number> = { unique: 7, set: 5, rare: 6, magic: 4, superior: 3, normal: 2, low: 1 }
    return map[q]
  }

  const equipMap = useMemo(() => {
    const m: Partial<Record<EquipSlot, EquippedItem>> = {}
    if (character?.equipment) {
      for (const slotInfo of character.equipment) {
        const slot = slotInfo.slot as EquipSlot
        if (slotInfo.occupied && slotInfo.code) {
          m[slot] = {
            code: slotInfo.code ?? '',
            name_zh: slotInfo.name_zh,
            name_en: slotInfo.name_en,
            name_zh_tw: slotInfo.name_zh_tw,
            quality: (slotInfo.quality as EquippedItem['quality']) ?? 'normal',
            tooltipData: slotInfo.tooltip,
            durability_cur: slotInfo.durability_cur,
            stats: slotInfo.stats,
            skill_bonuses: slotInfo.skill_bonuses,
          }
        } else {
          m[slot] = undefined
        }
      }
    }
    return m
  }, [character])

  const equipmentSkillBonuses = useMemo(
    () => character?.equipment.flatMap(slot => slot.skill_bonuses ?? []) ?? [],
    [character],
  )

  const mercEquipMap = useMemo(() => {
    const m: Partial<Record<EquipSlot, EquippedItem>> = {}
    if (character?.merc_equipment) {
      for (const slotInfo of character.merc_equipment) {
        const slot = slotInfo.slot as EquipSlot
        if (slotInfo.occupied && slotInfo.code) {
          m[slot] = {
            code: slotInfo.code ?? '',
            name_zh: slotInfo.name_zh,
            name_en: slotInfo.name_en,
            name_zh_tw: slotInfo.name_zh_tw,
            quality: (slotInfo.quality as EquippedItem['quality']) ?? 'normal',
            tooltipData: slotInfo.tooltip,
            durability_cur: slotInfo.durability_cur,
            durability_max: slotInfo.durability_max,
            stats: slotInfo.stats,
            skill_bonuses: (slotInfo as any).skill_bonuses,
          }
        } else {
          m[slot] = undefined
        }
      }
    }
    return m
  }, [character])

  const isLoadingList = listStatus === 'loading' || listStatus === 'initial'
  const isIdleChar = charStatus === 'idle'
  const isLoadingChar = charStatus === 'loading'
  const isCharReady = charStatus === 'ready'

  return (<>
      <div style={{
        display: 'flex', alignItems: 'center', gap: 10,
        padding: '10px 16px',
        borderBottom: '1px solid var(--color-d2emu-line, #252525)',
        background: 'rgba(0,0,0,0.15)',
        flexWrap: 'wrap', rowGap: 8,
      }}>
        {/* 左侧：角色列表控制 */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexShrink: 0 }}>
          <i className="fa-solid fa-users" style={{ color: 'var(--color-d2emu-gold-dim, #b8963c)', fontSize: 14 }} />
          <span style={{ ...LABEL, fontSize: 13 }}>角色</span>
          <span className="d2emu-tag" style={{ margin: 0 }}>{characters.length}</span>
          <button type="button" onClick={() => onRefresh(false)} disabled={isLoadingList}
            className="d2emu-btn d2emu-btn-ghost d2emu-btn-xs" title="刷新角色列表">
            <i className={`fa-solid fa-rotate-right${isLoadingList ? ' fa-spin' : ''}`} />
          </button>
        </div>

        <div style={{ flex: 1, minWidth: 16 }} />

        {/* 右侧：当前角色信息 + 操作 */}
        {selectedChar && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexShrink: 0, flexWrap: 'wrap', rowGap: 4 }}>
            {/* 角色名/等级/职业（从缓存或已加载数据） */}
            {(() => {
              const cached = getCachedClass(selectedChar)
              const clsName = cached ? getClassName(cached.class_en, cached.class_cn) : null
              const lv = cached?.level
              return (cached && (clsName || lv)) ? (
                <span style={{ color: 'var(--color-d2emu-gold-bright, #d6a735)', fontSize: 13, whiteSpace: 'nowrap' }}>
                  {selectedChar}
                  {lv ? <span style={{ color: 'var(--color-d2emu-muted, #666)', marginLeft: 4 }}>Lv.{lv} · {clsName}</span> : null}
                </span>
              ) : (
                <span style={{ color: '#aaa', fontSize: 13, whiteSpace: 'nowrap' }}>{selectedChar}</span>
              )
            })()}

            {/* 重载角色（已加载时可见） */}
            {isCharReady && (
              <button type="button" onClick={onLoad} disabled={isLoadingChar}
                className="d2emu-btn d2emu-btn-ghost d2emu-btn-xs" title="重新加载角色数据">
                <i className="fa-solid fa-arrow-rotate-right" /> 重载
              </button>
            )}
            {/* 存入仓库 */}
            {isCharReady && (
              <button type="button" onClick={onExtract} disabled={extracting}
                className="d2emu-btn d2emu-btn-xs" style={{ fontSize: 12, padding: '2px 10px' }}>
                <i className="fa-solid fa-box-archive" /> {extracting ? '存入中...' : '存入仓库'}
              </button>
            )}
          </div>
        )}
      </div>

      {/* ═══ 主体：左列表 + 右内容 ═══ */}
      <div style={{ display: 'flex', flexGrow: 1, flexShrink: 1, flexBasis: '0%', minHeight: 0, padding: 0 }}>
        {/* ═══ 左侧：角色列表 ═══ */}
        <div style={{
          flex: '0 1 230px', display: 'flex', flexDirection: 'column', gap: 2,
          borderRight: '1px solid var(--color-d2emu-line, #252525)',
          padding: '10px 12px', position: 'relative',
          minWidth: 160,
        }}>
          <div style={{ flex: 1, overflowY: 'auto', overflowX: 'hidden', display: 'flex', flexDirection: 'column', gap: 2 }}>
            {characters.map(name => (
              <CharListItem
                key={name}
                name={name}
                selected={selectedChar === name}
                character={character && selectedChar === name ? character : null}
                onClick={() => onSelectCharacter(saveFolder, name)}
                disabled={isLoadingList}
                changed={changedNames?.includes(name)}
                onDismiss={onDismissChanged}
              />
            ))}
          </div>
          {isLoadingList && (
            <div style={{
              position: 'absolute', inset: 0,
              background: 'rgba(8, 4, 2, 0.85)',
              display: 'flex', flexDirection: 'column',
              alignItems: 'center', justifyContent: 'center',
              gap: 8, zIndex: 10, borderRadius: 6,
            }}>
              <i className="fa-solid fa-spinner fa-spin" style={{ fontSize: 20, color: 'var(--color-d2emu-gold-bright)' }} />
              <div style={{ fontSize: 12, color: '#888' }}>加载角色列表...</div>
            </div>
          )}
        </div>

        {/* ═══ 右侧：内容面板 ═══ */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, padding: '12px 16px', position: 'relative' }}>

          {/* 加载中（列表就绪但角色在加载） */}
          {isLoadingChar && (
            <div style={{
              flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
            }}>
              <D2EmuLoading text="正在加载角色数据..." />
            </div>
          )}

          {/* 就绪态：显示角色内容 */}
          {isCharReady && character && (
            <>
              {/* 角色头像/姓名条 */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 12, paddingBottom: 12, borderBottom: '1px solid var(--color-d2emu-line, #252525)' }}>
                <Portrait character={character} itemLanguage={itemLanguage} />
              </div>

              {/* 标签栏 */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10, flexWrap: 'wrap', rowGap: 8 }}>
                <div className="d2emu-tabbar-ribbon">
                  {TABS.map(t => (
                    <button key={t.key} type="button" onClick={() => setTab(t.key)}
                      className={`d2emu-tab-ribbon ${tab === t.key ? 'is-on' : ''}`}>
                      <i className={`fa-solid ${t.icon}`} style={{ marginRight: 4 }} />
                      {t.label}
                    </button>
                  ))}
                </div>
              </div>

              {/* Tab 内容 */}
              <div className="d2emu-tab-content d2emu-tab-panel">
                {tab === 'overview' && (
                  <div style={{ display: 'flex', gap: 20, flexWrap: 'wrap', justifyContent: 'flex-start', alignItems: 'flex-start' }}>
                    {/* 左侧:核心属性 stat 列表(靠左,每元素一行) */}
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 4, flex: '0 0 auto', minWidth: 200 }}>
                      <div className="d2emu-section-title-arcane" style={{ marginBottom: 4, textAlign: 'left' }}>
                        <i className="fa-solid fa-coins" style={{ marginRight: 4 }} />档案
                      </div>
                      <div className="d2emu-stat-grid" style={{ gridTemplateColumns: '1fr' }}>
                        <StatBlock label="等级" value={character.level} variant="gold" />
                        <StatBlock label="经验" value={fmtNum(character.experience)} variant="xp" />
                        <StatBlock label="背包金币" value={fmtNum(character.gold)} variant="gold" />
                        <StatBlock label="仓库金币" value={fmtNum(character.gold_bank)} variant="gold" />
                        <StatBlock label="剩余技能" value={fmtNum(character.new_skills)} variant="sp" />
                      </div>
                      <div className="d2emu-section-title-arcane" style={{ margin: '10px 0 4px', textAlign: 'left' }}>
                        <i className="fa-solid fa-chart-simple" style={{ marginRight: 4 }} />属性
                      </div>
                      <div className="d2emu-stat-grid" style={{ gridTemplateColumns: '1fr' }}>
                        <StatBlock label="力量" value={character.strength} variant="str" />
                        <StatBlock label="精力" value={character.energy} variant="ene" />
                        <StatBlock label="敏捷" value={character.dexterity} variant="dex" />
                        <StatBlock label="体力" value={character.vitality} variant="vit" />
                        <StatBlock label="剩余属性" value={fmtNum(character.stat_points)} variant="gold" />
                      </div>
                    </div>
                    <EquipmentPanel
                      characterName={character.name}
                      level={character.level}
                      className={character.class_cn}
                      displayLanguage={itemLanguage}
                      equipment={equipMap}
                      hideHeader
                      onSelect={(slot, item) => {
                        if (item) setSelectedEquip({ slot, item })
                      }}
                    />
                    {/* 中右:背包 */}
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 8, flex: '0 0 auto' }}>
                      <PositionedItemGrid
                        items={(character.backpack_items ?? []).filter(item => !item.page || item.page === 1).map(item => ({
                          code: item.code, x: item.x, y: item.y,
                          amount: item.amount,
                          inv_width: item.inv_width, inv_height: item.inv_height,
                          quality: item.quality,
                          name_zh: item.name_zh, name_en: item.name_en,
                          page: item.page,
                          tooltipData: (item as any).tooltip,
                          stats: item.stats,
                          skillBonuses: item.skill_bonuses,
                        }))}
                        label="背包"
                        cols={backpackCols ?? 10}
                        rows={backpackRows ?? 4}
                      />
                    </div>
                    {/* 最右:装备加层 (EquipmentBonusTome) */}
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 8, flex: '0 0 auto', minWidth: 220 }}>
                      <EquipmentBonusTome
                        bonuses={equipmentSkillBonuses}
                        language={itemLanguage}
                        characterClass={character.class_en}
                      />
                    </div>
                  </div>
                )}
                {tab === 'storage' && (
                  <>
                    <InventoryView
                      backpackItems={character.backpack_items ?? []}
                      beltItems={character.belt_items ?? []}
                      backpackCols={backpackCols ?? 10}
                      backpackRows={backpackRows ?? 4}
                      cubeCols={cubeCols ?? 10}
                      cubeRows={cubeRows ?? 10}
                    />
                    <PositionedItemGrid
                      items={(character.personal_stash_items ?? []).map(item => ({
                        code: item.code, x: item.x, y: item.y,
                        amount: item.amount,
                        inv_width: item.inv_width, inv_height: item.inv_height,
                        quality: item.quality,
                        name_zh: item.name_zh, name_en: item.name_en,
                        tooltipData: (item as any).tooltip,
                        stats: item.stats,
                        skillBonuses: item.skill_bonuses,
                      }))}
                      label="个人仓库"
                      cols={16}
                      rows={16}
                      fixed
                    />
                  </>
                )}
                {tab === 'skills' && (
                  <div style={{ width: '100%', maxWidth: hasSkillTreeLayout(character.class_en) ? undefined : 520 }}>
                    {hasSkillTreeLayout(character.class_en)
                      ? <SkillTreeDisplay
                          skills={character.skills_decoded ?? []}
                          class_en={character.class_en}
                          language={itemLanguage}
                          remainingSkillPoints={character.new_skills}
                          equipmentBonuses={equipmentSkillBonuses}
                        />
                      : <CharacterSkills skills={character.skills_decoded ?? []} class_en={character.class_en} language={itemLanguage} />
                    }
                  </div>
                )}
                {tab === 'waypoints' && (
                  <div style={{ width: '100%' }}>
                    <CharacterWaypoints waypoints={character.waypoints} language={itemLanguage} />
                  </div>
                )}
                {tab === 'quests' && (
                  <div style={{ width: '100%' }}>
                    <CharacterQuests quests={character.quests} language={itemLanguage} />
                  </div>
                )}
                {tab === 'rewards' && (
                  <CharacterRewards w4={character.w4} quests={character.quests} />
                )}
                {tab === 'merc' && (
                  <EquipmentPanel
                    characterName={character.name + '(佣兵)'}
                    level={0}
                    className="Mercenary"
                    displayLanguage={itemLanguage}
                    equipment={mercEquipMap}
                    hideHeader
                    slots={['helm', 'amulet', 'ring_l', 'ring_r', 'armor', 'weapon_main', 'shield_main', 'gloves', 'boots', 'belt']}
                  />
                )}
                {tab === 'warehouse' && (
                  <div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
                      {stashData && stashData.pages.length > 1 && (
                        <div className="d2emu-tabbar" style={{ marginBottom: 4 }}>
                          {stashData.pages.map(pg => {
                            const hasItems = pg.item_count > 0
                            const name = pg.label?.trim() || `页 ${pg.index + 1}`
                            return (
                              <button key={pg.index} type="button"
                                onClick={() => setStashPage(pg.index)}
                                className={`d2emu-tab ${stashPage === pg.index ? 'd2emu-tab-active' : ''}`}
                                style={{ opacity: hasItems ? 1 : 0.45 }}
                                title={hasItems ? `${name} · ${pg.item_count} 件物品` : `${name} · 空`}>
                                {name}
                                {hasItems ? (
                                  <span style={{ marginLeft: 4, color: '#c7b377', fontWeight: 700 }}>({pg.item_count})</span>
                                ) : (
                                  <span style={{ marginLeft: 4, opacity: 0.6 }}>·空</span>
                                )}
                              </button>
                            )
                          })}
                        </div>
                      )}
                      <button type="button" onClick={() => { setStashData(null); fetchStash(true) }}
                        className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" title="刷新共享仓库"
                        style={{ marginLeft: 'auto' }}>
                        <i className="fa-solid fa-rotate-right" />
                      </button>
                    </div>
                    {!stashData ? (
                      <D2EmuLoading text="加载共享仓库中..." />
                    ) : (() => {
                      const pageInfo = stashData.pages.find(p => p.index === stashPage) ?? stashData.pages[0]
                      const pageItems = stashData.items.filter(item => item.page_index === pageInfo.index)
                      if (pageInfo.is_stackable) {
                        return <StackablePageView items={pageItems} />
                      }
                      return (
                        <PositionedItemGrid
                          items={pageItems.map(item => ({
                            code: item.code,
                            x: item.position_x,
                            y: item.position_y,
                            amount: item.quantity,
                            inv_width: item.inv_width,
                            inv_height: item.inv_height,
                            quality: stashQualityToNum(item.quality),
                            name_zh: item.item_name,
                            name_en: item.name_en,
                            page: item.page_index,
                            tooltipData: item.tooltip,
                            tooltip_lines: undefined,
                          }))}
                          label={pageInfo.label}
                          cols={pageInfo.grid_width}
                          rows={pageInfo.grid_height}
                        />
                      )
                    })()}
                  </div>
                )}
              </div>
            </>
          )}
          
          {/* 空闲态：已选角色但未加载 */}

          {/* 空闲态：已选角色但未加载 */}
          {!isLoadingChar && !isCharReady && selectedChar && (
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 16, padding: 48 }}>
              <button type="button" onClick={onLoad} disabled={isLoadingList}
                className="d2emu-btn d2emu-btn-sm" style={{ fontSize: 15, padding: '10px 28px' }}>
                <i className={`fa-solid fa-play`} />
                {' '}加载角色
              </button>
              <div style={{ textAlign: 'center', color: 'var(--color-d2emu-muted, #666)', font: '500 14px/1.6 "Source Sans 3", sans-serif' }}>
                点击上方按钮查看角色详情
              </div>
            </div>
          )}

          {/* 无选中角色 */}
          {!isLoadingChar && !isCharReady && !selectedChar && characters.length > 0 && (
            <div className="empty-state">请从左侧选择角色</div>
          )}
          {!isLoadingChar && !isCharReady && !selectedChar && characters.length === 0 && !isLoadingList && (
            <div className="empty-state">未找到角色档案</div>
          )}

          <EquipmentDetailModal
            selected={selectedEquip}
            onClose={() => setSelectedEquip(null)}
          />

          {isLoadingList && characters.length === 0 && (
            <div style={{
              position: 'absolute', inset: 0,
              background: 'rgba(8, 4, 2, 0.85)',
              display: 'flex', flexDirection: 'column',
              alignItems: 'center', justifyContent: 'center',
              zIndex: 50, borderRadius: 8, gap: 8,
            }}>
              <D2EmuLoading text="加载角色列表..." />
            </div>
          )}
        </div>
    </div>
  </>)
}
