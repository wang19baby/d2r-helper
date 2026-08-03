import type { CSSProperties } from 'react'
import type { QualityKey } from './QualityLegend'
import { QUALITY_COLOR } from './QualityLegend'
import ItemTooltip from './ItemTooltip'
import SocketsOverlay from './SocketsOverlay'
import type { SkillBonus, ItemStats } from '../types'
import type { TooltipData } from '../types'
import { resolveItemIcon, handleImgError } from '../utils/itemImages'

export type EquipSlot =
  | 'helm' | 'amulet' | 'ring_l' | 'ring_r' | 'armor'
  | 'weapon_main' | 'shield_main'
  | 'weapon_alt' | 'shield_alt'
  | 'gloves' | 'boots' | 'belt'

export const SLOT_LABEL: Record<EquipSlot, string> = {
  helm:   '头部',
  amulet: '护身符',
  ring_l: '左戒指',
  ring_r: '右戒指',
  armor:  '护甲',
  weapon_main: '主武器',
  shield_main: '主手盾',
  weapon_alt: '副武器',
  shield_alt: '副手盾',
  gloves: '手套',
  boots:  '靴子',
  belt:   '腰带',
}

/* 8 sub-cols × 10 sub-rows,每个 sub-cell = equip-cell / 2
   旧 (r, c) → 新 (2r-1, 2c-1, 2r, 2c)  (end-exclusive 约定:2 old rows = 4 sub-rows)
   头盔显式 2×2 sub;盾牌/副武器左移到衣服旁边(col 5-6 / col 7-8,不再在 col 13-14)
   项链缩成 1×1 sub,放在原 amulet 区域左上角(col 5 row 1)
   双戒 1×1 sub,在项链正下方 row 2 水平并排(col 5 + col 6)
   副武/副盾高度翻倍成 4×2 sub: weapon_alt rows 1-4, shield_alt rows 5-8 */
export const SLOT_GRID_AREA: Record<EquipSlot, string> = {
  helm:        '1 / 3 / 3 / 5',       // 2 sub-rows × 2 sub-cols (2×2 sub,120×120,正上方 armor)
  amulet:      '1 / 5 / 2 / 6',       // 1×1 sub @ 当前区域左上角(col 5 row 1)
  ring_l:      '2 / 5 / 3 / 6',       // 1×1 sub @ 项链正下方左(col 5 row 2)
  ring_r:      '2 / 6 / 3 / 7',       // 1×1 sub @ 项链正下方右(col 6 row 2)
  weapon_main: '3 / 1 / 7 / 3',       // 4×2 sub (cols 1-2)
  armor:       '3 / 3 / 7 / 5',       // 4×4 sub (cols 3-4)
  shield_main: '3 / 5 / 7 / 7',       // 4×2 sub (cols 5-6,挨着 armor)
  weapon_alt:  '1 / 7 / 5 / 9',       // 4×2 sub → 高度翻倍 (rows 1-4, cols 7-8)
  shield_alt:  '5 / 7 / 9 / 9',       // 4×2 sub → 高度翻倍 (rows 5-8, cols 7-8)
  belt:        '7 / 3 / 9 / 5',       // 2×2 sub
  gloves:      '7 / 1 / 9 / 3',       // 2×2 sub
  boots:       '7 / 5 / 9 / 7',       // 2×2 sub
}
export interface EquippedItem {
  /** 装备 code (e.g. "gth", "amu") */
  code: string
  /** 中文名 (optional, fallback to code) */
  name_zh?: string
  /** 英文名 (optional, fallback to code) */
  name_en?: string
  /** 繁体中文名 (optional, fallback to code) */
  name_zh_tw?: string
  /** 装备 quality (用于 border 颜色) */
  quality: QualityKey
  /** 结构化 tooltip 数据（后端已分类） */
  tooltipData?: TooltipData
  /** 物品图标 URL (可选,fallback 为纯文字 code) */
  icon?: string
  /** 是否有孔（镶嵌） */
  socketed?: boolean
  /** 当前耐久度 */
  durability_cur?: number
  /** 最大耐久度 */
  durability_max?: number
  /** 原始 stat 分类数据 */
  skill_bonuses?: SkillBonus[]
  stats?: ItemStats
}

export interface EquipmentPanelProps {
  /** 角色名 */
  characterName: string
  /** 等级 */
  level: number
  /** 职业 */
  className: string
  /** 12 个展示槽位装备 (未装备 = undefined) */
  equipment: Partial<Record<EquipSlot, EquippedItem>>
  /** 当前显示语言 */
  displayLanguage?: string
  /** 点击装备槽 (optional, 用于 future 显示详情) */
  onSelect?: (slot: EquipSlot, item?: EquippedItem) => void
  /** 槽位尺寸 (default 56) */
  cellSize?: number
  /** 是否隐藏标题块（角色名/等级/职业），用于嵌入 CharacterPanel 时去重 */
  hideHeader?: boolean
  /** 自定义槽位列表（默认 12 槽）*/
  slots?: EquipSlot[]
}

function displayItemName(item: EquippedItem, language: string) {
  const CODE_ZH: Record<string, string> = {
    dag: '匕首', dgr: '匕首', dir: '短刀', kri: '波形刀', bld: '刃',
    hlm: '帽子', hla: '硬皮甲', hem: '头盔', hbl: '高级头盔', skp: '骷髅帽',
    ghm: '巨盔', shl: '王冠', xhl: '头冠', qui: '布甲', lea: '皮甲',
    stu: '镶嵌甲', rng: '锁环甲', scl: '鳞甲', chn: '锁子甲',
    buc: '小盾牌', smi: '小型盾', lrg: '大型盾', kit: '鸢盾', tow: '塔盾',
    gts: '哥特盾', bon: '骨盾', mst: '统治者大盾',
    hnd: '手斧', axe: '斧', '2ax': '双面斧', wax: '战斧',
    wnd: '法杖', sds: '短剑', sbr: '弯刀', scs: '军刀', spr: '长矛',
    crs: '水晶剑', bsw: '阔剑', lsd: '长剑', '2hs': '双手剑',
    lgl: '轻手套', vgl: '手套', mgl: '重手套', tgl: '巨战手套',
    lbt: '轻靴', vbt: '靴子', mbt: '重靴', tbt: '巨战之靴',
    lbl: '饰带', vbl: '轻腰带', mbl: '腰带', tbl: '重腰带',
    rin: '戒指', amu: '护身符', jew: '珠宝',
    r01: '艾尔符文', r02: '艾德符文', r03: '特尔符文', r04: '那夫符文',
    r05: '爱斯符文', r06: '伊司符文', r07: '塔尔符文', r08: '拉尔符文',
    r09: '欧特符文', r10: '书尔符文', r11: '安姆符文', r12: '索尔符文',
    r13: '沙尔符文', r14: '多尔符文', r15: '海尔符文', r16: '埃欧符文',
    r17: '卢姆符文', r18: '科符文', r19: '法尔符文', r20: '蓝姆符文',
    r21: '普尔符文', r22: '乌姆符文', r23: '马尔符文', r24: '伊司特符文',
    r25: '古尔符文', r26: '伐克斯符文', r27: '欧姆符文', r28: '罗符文',
    r29: '瑟符文', r30: '贝符文', r31: '乔符文', r32: '查姆符文', r33: '萨德符文',
    gcv: '裂开的紫宝石', gcy: '有瑕疵的紫宝石', gcb: '紫宝石', gcp: '无瑕的紫宝石', gcz: '完美的紫宝石',
    gsv: '裂开的蓝宝石', gsy: '有瑕疵的蓝宝石', gsb: '蓝宝石', gsp: '无瑕的蓝宝石', gsz: '完美的蓝宝石',
    grv: '裂开的红宝石', gry: '有瑕疵的红宝石', grb: '红宝石', grp: '无瑕的红宝石', grz: '完美的红宝石',
    gnv: '裂开的绿宝石', gny: '有瑕疵的绿宝石', gnb: '绿宝石', gnp: '无瑕的绿宝石', gnz: '完美的绿宝石',
    k1: '恐惧之钥', k2: '憎恨之钥', k3: '毁灭之钥',
    key: '钥匙', tps: '回城卷轴', hpo: '治疗药水', mpo: '法力药水',
  }
  const code = item.code?.toLowerCase() ?? ''
  if (language === 'zhCN' || language === 'zhTW') {
    // 优先用后端返回的 name_zh（含符文之语名等动态名称）
    if (item.name_zh && /[\u4e00-\u9fff]/.test(item.name_zh)) return item.name_zh
    const zh = CODE_ZH[code]
    if (zh) return zh
  }
  const raw = language === 'zhTW'
    ? item.name_zh_tw || item.name_zh || item.name_en
    : language === 'enUS'
      ? item.name_en || item.name_zh || item.name_zh_tw
      : item.name_zh || item.name_zh_tw || item.name_en
  if (raw && raw !== item.code && raw.toLowerCase() !== code) return raw
  return code || ''
}

/**
 * EquipmentPanel — D2 角色身上装备 12 槽展示区
 *
 * 布局 (8 sub-cols × 10 sub-rows,4 列 × 5 行人型):
 *   row 1: [   ]      [Helm]      [A]      [W_alt]
 *   row 2: [   ]      [Helm]      [L|R]    [W_alt]
 *   row 3: [主武]     [护甲     ] [主盾]    [S_alt]
 *   row 4: [主武]     [护甲     ] [主盾]    [S_alt]
 *   row 5: [主武]     [护甲     ] [主盾]    [   ]
 *   row 6: [主武]     [护甲     ] [主盾]    [   ]
 *   row 7: [手套]     [腰带]      [靴]     [   ]
 *   row 8: [手套]     [腰带]      [靴]     [   ]
 *
 * 注:项链缩成 1×1 sub @ row 1 col 5;双戒 1×1 sub 在 row 2 cols 5-6 水平并排;
 * 副武/副盾缩成 2×2 sub,顶部右侧排布(weapon_alt rows 1-2,shield_alt rows 3-4);
 * 每个 cell 沿用 d2emu quality border 5 色系统
 */
export default function EquipmentPanel({
  characterName, level, className, equipment, displayLanguage = 'zhCN',
  onSelect, cellSize = 56, hideHeader = false, slots: slotsProp,
}: EquipmentPanelProps) {
  const slots: EquipSlot[] = slotsProp ?? [
    'helm', 'amulet', 'ring_l', 'ring_r', 'armor',
    'weapon_main', 'shield_main',
    'weapon_alt', 'shield_alt',
    'gloves', 'boots', 'belt',
  ]

  /** 渲染单格物品（图标+名称，空时显示占位文本） */
  const renderSlotItem = (item: EquippedItem | undefined, label: string, lang: string) => {
    if (item) {
      const c = QUALITY_COLOR[item.quality] ?? QUALITY_COLOR.normal
      return (
        <>
          <img src={resolveItemIcon(item)} alt={item.code} data-code={item.code}
            style={{ width: '70%', height: '60%', objectFit: 'contain', imageRendering: 'pixelated' }}
            onError={handleImgError} />
          <span style={{
            color: c, fontSize: 11, lineHeight: 1.2,
            fontFamily: '"Source Sans 3", sans-serif',
            fontWeight: 600, textAlign: 'center',
            maxWidth: '100%',
            whiteSpace: 'pre-line', padding: '0 2px',
            wordBreak: 'break-word',
          }}>{displayItemName(item, lang)}</span>
        </>
      )
    }
    return (
      <span style={{
        color: 'var(--color-d2emu-muted, #555)',
        font: '700 7px/1 "Source Sans 3", sans-serif',
        letterSpacing: '0.05em',
      }}>{label.slice(0, 2)}</span>
    )
  }

  return (
    <div className="d2emu-equipment-panel" role="group"
      style={{ padding: '12px 8px', overflow: 'auto' }}
      aria-label={`${characterName} 装备`}>
      {/* 标题块 (hideHeader=true 时跳过，用于嵌入 CharacterPanel 去重) */}
      {!hideHeader && (
        <div style={{
          textAlign: 'center',
          marginBottom: 10,
          paddingBottom: 10,
          borderBottom: '1px solid var(--color-d2emu-line, #252525)',
        }}>
          <div style={{
            color: 'var(--color-d2emu-muted, #888)',
            font: '600 14px/1 "Source Sans 3", sans-serif',
            letterSpacing: '0.12em',
            textTransform: 'uppercase',
          }}>
            Equipment
          </div>
          <div style={{
            color: 'var(--color-d2emu-gold-bright, #fff)',
            font: '600 14px/1.2 Cinzel, serif',
            letterSpacing: '1.5px',
            textTransform: 'uppercase',
            margin: '4px 0 2px',
          }}>
            {characterName}
          </div>
          <div style={{
            color: 'var(--color-d2emu-gold, #c7b377)',
            font: '600 14px/1 "Source Sans 3", sans-serif',
          }}>
            Lv {level} · {className}
          </div>
        </div>
      )}

      {/* 4×5 grid 装备人型 — 尺寸由 CSS var --equip-cell 控制,断点降级在 index.css;cellSize 入参仅作 JS 回退 */}
      <div className="d2emu-equipment-grid"
        style={{
          gap: 6,
          width: 'max-content',
          maxWidth: '100%',
          overflowX: 'auto',
          margin: '0 auto',
          position: 'relative',
        } as CSSProperties}>
        {slots.map(slot => {

          const item = equipment[slot]
          const bc = item
            ? (QUALITY_COLOR[item.quality] ?? QUALITY_COLOR.normal)
            : 'var(--color-d2emu-line, #252525)'
          const area = SLOT_GRID_AREA[slot]
          const isOn = !!item
          const isLowDur = item && item.durability_max && item.durability_max > 0
            && (item.durability_cur ?? 0) / item.durability_max <= 0.1
          return (
            <button key={slot}
              className={`group d2emu-item-slot-arcane ${isOn ? 'd2emu-item-slot-occupied' : ''} ${isLowDur ? 'is-low-dur' : ''}`}
              type="button"
              disabled={!onSelect}
              onClick={() => onSelect?.(slot, item)}
              // 有物品的槽由 ItemTooltip (portal) 展示详情, 原生 title 只留给空槽提示
              title={isOn ? undefined : `${SLOT_LABEL[slot]}: 空`}
              style={{
                gridArea: area,
                color: bc,
                flexDirection: isOn ? 'column' : undefined,
                gap: isOn ? 2 : undefined,
                padding: isOn ? '2px 1px' : undefined,
                backgroundColor: isLowDur ? 'rgba(180, 30, 20, 0.25)' : undefined,
                position: isOn && item?.tooltipData?.sockets ? 'relative' as const : undefined,
              }}>
              {isOn && item && (
                <span
                  className="d2emu-wax-seal"
                  style={{ background: bc }}
                >{item.quality?.charAt(0).toUpperCase() ?? '?'}</span>
              )}
              {isOn && item ? (
                <>
                <div style={{ position: 'relative', display: 'inline-flex', alignItems: 'center', justifyContent: 'center', width: '90%', height: '70%' }}>
                  <img src={resolveItemIcon(item)} alt={item.code} data-code={item.code}
                    style={{ width: '100%', height: '100%', objectFit: 'contain', imageRendering: 'pixelated' }}
                    onError={handleImgError} />
                  {item.tooltipData?.sockets && (
                    <SocketsOverlay sockets={item.tooltipData.sockets} />
                  )}
                </div>
                  <span
                    className="item-code-fallback"
                    style={{
                      color: bc,
                      font: '700 9px/1 "Source Sans 3", monospace',
                      textTransform: 'uppercase',
                      display: 'none',
                    }}
                  >{item.code}</span>
                  <span className="d2emu-slot-itemname" style={{ color: bc }}>
                    {displayItemName(item, displayLanguage)}
                  </span>
                </>
              ) : (
                <span style={{
                  color: 'var(--color-d2emu-muted, #555)',
                  font: '700 9px/1 "Source Sans 3", sans-serif',
                  letterSpacing: '0.05em',
                }}>{SLOT_LABEL[slot].slice(0, 2)}</span>
              )}
              {isOn && item ? (
                <ItemTooltip
                  tooltipData={item.tooltipData}
                  quality={item.quality}
                  itemCode={item.code}
                  nameZh={item.name_zh}
                  english={item.name_en}
                  stats={item.stats ?? null}
                  skillBonuses={item.skill_bonuses}
                  mode="hover"
                  position={slot === 'helm' || slot === 'amulet' ? 'bottom' : 'top'}
                  style={{
                    right: slot === 'shield_main' ? 0 : 'auto',
                    left: slot === 'shield_alt' ? 0 : 'auto',
                  }}
                />
              ) : null}
            </button>
          )
        })}
      </div>

      {/* 装饰底框 */}
      <div style={{ marginTop: 8, width: '100%', height: 1, background: 'var(--color-d2emu-line, #252525)' }} />
    </div>
  )
}
