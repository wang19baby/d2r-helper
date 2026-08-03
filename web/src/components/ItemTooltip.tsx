import { useState, useRef, useEffect, type CSSProperties } from 'react'
import { createPortal } from 'react-dom'
import type { TooltipData, SkillBonus, ItemStats, StashSocketedItemInfo } from '../types'

/** code→中文 fallback（后端无 resolver 时 name_zh 会是英文） */
const CODE_ZH: Record<string, string> = {
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
  dag: '匕首', dgr: '匕首', dir: '短刀', kri: '波形刀', bld: '刃',
  r01: '艾尔符文', r02: '艾德符文', r03: '特尔符文', r04: '那夫符文',
  r05: '爱斯符文', r06: '伊司符文', r07: '塔尔符文', r08: '拉尔符文',
  r09: '欧特符文', r10: '书尔符文', r11: '安姆符文', r12: '索尔符文',
  r13: '沙尔符文', r14: '多尔符文', r15: '海尔符文', r16: '埃欧符文',
  r17: '卢姆符文', r18: '科符文', r19: '法尔符文', r20: '蓝姆符文',
  r21: '普尔符文', r22: '乌姆符文', r23: '马尔符文', r24: '伊司特符文',
  r25: '古尔符文', r26: '伐克斯符文', r27: '欧姆符文', r28: '罗符文',
  r29: '瑟符文', r30: '贝符文', r31: '乔符文', r32: '查姆符文', r33: '萨德符文',
}

export interface ItemTooltipProps {
  /** 结构化 tooltip 数据（后端已分类） */
  tooltipData?: TooltipData | null
  quality?: string | null
  itemCode?: string | null
  quantity?: number
  nameZh?: string | null
  english?: string | null
  modAdded?: boolean
  socketedItems?: StashSocketedItemInfo[]
  /** 原始 tooltip 行（当 tooltipData 不可用时的 fallback） */
  tooltipLines?: string[] | null
  /** 原始 stat 分类数据 */
  stats?: ItemStats | null
  /** 技能词缀（hover tooltip 展示用） */
  skillBonuses?: SkillBonus[] | null
  mode?: 'hover' | 'inline'
  position?: 'top' | 'bottom'
  className?: string
  style?: CSSProperties
}

const QUALITY_HEX: Record<string, string> = {
  unique: '#a08030', set: '#45b84a', rare: '#c4a847',
  magic: '#5d6cff', superior: '#e8e8e8', normal: '#e8e8e8',
  socketed: '#6a4a8a', runeword: '#f09020',
}
const qColor = (q?: string | null) => QUALITY_HEX[q || 'normal'] || QUALITY_HEX.normal

/** 渲染分类 stat 原始数据为文本列表。 */

export default function ItemTooltip({
  tooltipData, quality, itemCode, quantity,
  nameZh, english: englishProp, modAdded, socketedItems, stats, skillBonuses,
  tooltipLines,
  mode = 'inline',
  position = 'top',
  className,
  style,
}: ItemTooltipProps) {
  // tooltipLines fallback — wrap raw lines as pseudo-TooltipData
  const displayData = tooltipData
    ? tooltipData
    : (tooltipLines?.length
      ? { base_info: tooltipLines, stats: [], hidden_info: [], set_info: [] }
      : undefined)
  if (mode === 'hover') {
    return (
      <HoverTooltipWrapper
        tooltipData={displayData}
        quality={quality} itemCode={itemCode} nameZh={nameZh} english={englishProp}
        socketedItems={socketedItems} stats={stats} skillBonuses={skillBonuses}
        position={position} className={className} style={style}
      />
    )
  }
  return (
    <TooltipContent
      tooltipData={displayData}
      quality={quality} itemCode={itemCode} nameZh={nameZh} english={englishProp}
      socketedItems={socketedItems} stats={stats} skillBonuses={skillBonuses}
      className={className} style={style}
    />
  )
}


function TooltipContent({
  tooltipData, quality, itemCode, quantity,
  nameZh, english: englishProp, modAdded, socketedItems, skillBonuses,
  className, style,
}: ItemTooltipProps) {
  const title = (nameZh && /[\u4e00-\u9fff]/.test(nameZh)) ? nameZh : CODE_ZH[itemCode ?? ''] || nameZh || itemCode || '—'
  const english = englishProp ?? undefined
  const c = qColor(quality)
  const affixStats = tooltipData?.stats ?? []
  // 后端新路径填 base_stats(结构化), 旧 classify_tooltip 填 base_info —— 两者都兼容
  const baseStats = tooltipData?.base_stats ?? tooltipData?.base_info ?? []
  const hasBase = baseStats.length > 0
  const affixOnly = tooltipData?.affix_stats ?? []
  const hasAffix = affixOnly.length > 0
  const rwStats = tooltipData?.runeword_stats ?? []
  const hasRw = rwStats.length > 0
  const setStats = tooltipData?.set_bonus_stats ?? []
  const hasSet = setStats.length > 0
  const hasMeta = (tooltipData?.hidden_info?.length ?? 0) > 0

  return (
    <div className={`d2emu-item-tooltip-inline ${className || ''}`} style={{ display: 'flex', flexDirection: 'column', ...style }}>
      {/* 标题 + 标签 */}
      <div style={{ paddingBottom: 2, marginBottom: 0, borderBottom: (hasBase || hasAffix || hasRw || hasSet || hasMeta) ? '1px solid var(--color-d2emu-line, #252525)' : 'none' }}>
        <div style={{ fontSize: 16, fontWeight: 700, lineHeight: 1.3, color: c, wordBreak: 'break-word' }}>{title}</div>
        <div style={{ fontSize: 16, fontStyle: 'italic', color: 'var(--color-d2emu-muted, #888)', marginTop: 4 }}>{english}{itemCode ? ` (${itemCode})` : ''}</div>
        <div className="d2emu-tags" style={{ marginTop: 8 }}>
          {quantity != null && quantity > 1 && <span className="d2emu-tag" style={{ color: c, fontSize: 11, lineHeight: '14px' }}>×{quantity}</span>}
          {modAdded && <span className="d2emu-tag" style={{ fontSize: 11, lineHeight: '14px' }}>MOD</span>}
        </div>
      </div>
      {hasBase && (
        <div style={{ paddingBottom: 2, marginBottom: 0, borderBottom: hasAffix || hasMeta ? '1px solid var(--color-d2emu-line, #252525)' : 'none' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {baseStats.map((s, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'flex-start', gap: 4, fontSize: 14, lineHeight: 1.3 }}>
                <span style={{ color: 'var(--color-d2emu-muted, #888)', marginTop: 2 }}>•</span>
                <span style={{ color: '#fff' }}>{s}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 词缀（蓝色） */}
      {hasAffix && (
        <div style={{ paddingBottom: 2, marginBottom: 0, borderBottom: hasMeta ? '1px solid var(--color-d2emu-line, #252525)' : 'none' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {affixOnly.map((s, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'flex-start', gap: 4, fontSize: 14, lineHeight: 1.3 }}>
                <span style={{ color: 'var(--color-d2emu-muted, #888)', marginTop: 2 }}>•</span>
                <span style={{ color: '#6a9fd8' }}>{s}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 符文之语（蓝绿） */}
      {hasRw && (
        <div style={{ paddingBottom: 2, marginBottom: 0, borderBottom: hasSet || hasMeta ? '1px solid var(--color-d2emu-line, #252525)' : 'none' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {rwStats.map((s, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'flex-start', gap: 4, fontSize: 14, lineHeight: 1.3 }}>
                <span style={{ color: 'var(--color-d2emu-muted, #888)', marginTop: 2 }}>•</span>
                <span style={{ color: '#6a9fd8' }}>{s}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 套装加成（绿色） */}
      {hasSet && (
        <div style={{ paddingBottom: 2, marginBottom: 0, borderBottom: hasMeta ? '1px solid var(--color-d2emu-line, #252525)' : 'none' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {setStats.map((s, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'flex-start', gap: 4, fontSize: 14, lineHeight: 1.3 }}>
                <span style={{ color: 'var(--color-d2emu-muted, #888)', marginTop: 2 }}>•</span>
                <span style={{ color: '#4a8f4a' }}>{s}</span>
              </div>
            ))}
          </div>
        </div>
      )}
      {socketedItems && socketedItems.length > 0 && (
        <div style={{ marginTop: 0, paddingTop: 0, borderTop: '1px solid var(--color-d2emu-line, #252525)' }}>
          <div style={{ fontSize: 16, color: 'var(--color-d2emu-muted, #888)', textTransform: 'uppercase', letterSpacing: '0.12em', marginBottom: 6, fontWeight: 600 }}>镶嵌</div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
            {socketedItems.map((s, i) => {
              const sc = s.quality ? (QUALITY_HEX[s.quality] || QUALITY_HEX.normal) : QUALITY_HEX.normal
              return (
                <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 16, lineHeight: 1.5 }}>
                  <span style={{ color: sc, fontWeight: 600 }}>{s.item_name || s.code}</span>
                  {(s.quantity ?? 1) > 1 && <span style={{ color: 'var(--color-d2emu-muted, #888)' }}>×{s.quantity}</span>}
                  {s.quality && <span style={{ fontSize: 16, color: sc, opacity: 0.7, textTransform: 'uppercase' }}>{s.quality}</span>}
                </div>
              )
            })}
          </div>
        </div>
      )}
    </div>
  )
}
function HoverTooltipWrapper({
  tooltipData, quality, itemCode, nameZh, english,
  socketedItems, stats, skillBonuses, tooltipLines,
  position, className, style,
}: ItemTooltipProps) {
  const resolved = tooltipData ?? (tooltipLines?.length
    ? { base_info: tooltipLines, stats: [], hidden_info: [], set_info: [] }
    : undefined)
  const c = qColor(quality)
  const [visible, setVisible] = useState(false)
  const [flipped, setFlipped] = useState(position)
  const tipRef = useRef<HTMLDivElement>(null)
  const posRef = useRef(position)
  const hideTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)
  const showTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)
  const visibleRef = useRef(visible)

  const show = () => {
    clearTimeout(hideTimer.current)
    clearTimeout(showTimer.current)
    showTimer.current = setTimeout(() => setVisible(true), 80)
  }
  const hide = () => {
    clearTimeout(showTimer.current)
    // 300ms 延迟: 给鼠标从源元素过渡到 tooltip 的时间 (进入 tooltip 会取消隐藏)
    hideTimer.current = setTimeout(() => { setVisible(false); setFlipped(posRef.current) }, 300)
  }
  const toggle = () => { if (visibleRef.current) hide(); else show() }
  // 事件监听在 wrapRef 上（始终渲染，不依赖 tooltip DOM）
  const wrapRef = useRef<HTMLSpanElement>(null)
  useEffect(() => { visibleRef.current = visible }, [visible])
  useEffect(() => {
    const wrap = wrapRef.current; if (!wrap) return
    const parent = wrap.parentElement; if (!parent) return
    parent.addEventListener('mouseenter', show); parent.addEventListener('mouseleave', hide)
    parent.addEventListener('click', toggle)
    return () => {
      parent.removeEventListener('mouseenter', show); parent.removeEventListener('mouseleave', hide)
      parent.removeEventListener('click', toggle)
    }
  }, [])

  useEffect(() => {
    if (!visible) return; const el = tipRef.current; if (!el) return
    const parent = wrapRef.current?.parentElement; if (!parent) return
    const recalc = () => {
      const tr = el.getBoundingClientRect(); const pr = parent.getBoundingClientRect(); const gap = 8
      const vw = window.innerWidth; const vh = window.innerHeight
      let top: number; let p = posRef.current
      if (p === 'top') { top = pr.top - tr.height - gap; if (top < 4) { top = pr.bottom + gap; p = 'bottom' } }
      else { top = pr.bottom + gap; if (top + tr.height > vh - 4) { top = pr.top - tr.height - gap; p = 'top' } }
      top = Math.max(4, Math.min(top, vh - tr.height - 4))
      let left = pr.left + pr.width / 2 - tr.width / 2; if (left < 8) left = 8; if (left + tr.width > vw - 8) left = vw - tr.width - 8
      el.style.top = top + 'px'; el.style.left = left + 'px'; el.style.visibility = 'visible'; setFlipped(p)
    }
    recalc(); window.addEventListener('resize', recalc)
    return () => { window.removeEventListener('resize', recalc); clearTimeout(hideTimer.current) }
  }, [visible])

  return (
    <span ref={wrapRef} style={{ display: 'contents' }}>
      {visible && createPortal(
        <div ref={tipRef} className={`d2emu-item-tooltip-hover ${className || ''}`} role="tooltip"
          style={{ position: 'fixed', zIndex: 10000, background: 'rgba(0,0,0,0.96)', border: '1px solid var(--color-d2emu-line, #252525)',
            borderRadius: 4, padding: '8px 10px', fontSize: 14, lineHeight: 1.4, minWidth: 220, maxWidth: 300,
            maxHeight: 'min(400px, 60vh)', overflowY: 'auto', opacity: 1,
            // 允许鼠标进入 tooltip 滚动长内容 (内容超长出现滚动条时, pointerEvents:none 无法滚动)
            pointerEvents: 'auto', ...style } as CSSProperties}
          onMouseEnter={() => { clearTimeout(hideTimer.current); clearTimeout(showTimer.current); setVisible(true) }}
          onMouseLeave={() => { clearTimeout(showTimer.current); hide() }}
        >
          <TooltipContent
            tooltipData={resolved}
            quality={quality} itemCode={itemCode} nameZh={nameZh} english={english}
            socketedItems={socketedItems} stats={stats} skillBonuses={skillBonuses}
          />
        </div>,
        document.body,
      )}
    </span>
  )
}
