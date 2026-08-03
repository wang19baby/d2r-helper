import { useMemo, useState, useCallback } from 'react'
import D2EmuCard from './D2EmuCard'
import {
  craftedItems,
  cubeRecipes,
  CRAFTED_INPUT_HINT,
  translateInputCode,
  type CraftedItem,
} from '../data/d2core/affixesIndex'
import { modCodeZh } from '../data/d2core/affix_zh'
import { tauriInvoke } from '../tauri.ts'

interface CraftedContext {
  owned_codes: string[]
  has_magic: boolean
  has_upgrades: boolean
}

// ── 手工类别中文名 ──
const CRAFTED_NAME_ZH: Record<string, { label: string; color: string; bg: string; border: string }> = {
  Hit:    { label: '打击',  color: '#ff8a65', bg: 'rgba(255,138,101,0.08)', border: '#8a4a35' },
  Blood:  { label: '鲜血',  color: '#ef5350', bg: 'rgba(239,83,80,0.08)',  border: '#7a2a28' },
  Caster: { label: '施法',  color: '#64b5f6', bg: 'rgba(100,181,246,0.08)', border: '#2a5a7a' },
  Safety: { label: '安全',  color: '#81c784', bg: 'rgba(129,199,132,0.08)', border: '#2a6a2a' },
}

// ── 物品类型代码 → 中文（手工配方特有） ──
const CRAFTED_ITEM_TYPE_ZH: Record<string, string> = {
  fhl:  '高级头盔', mbt:  '轻型靴', mgl:  '轻型铁手套', tbl:  '饰带',
  gts:  '哥特盾',   fld:  '板甲',   amul: '项链',       ring: '戒指',
  blun: '钝击武器', hlm:  '头盔',   tbt:  '重型靴',     vgl:  '重型铁手套',
  mbl:  '锁子甲',   spk:  '尖刺盾牌', plt: '铠甲',       axe:  '斧',
  msk:  '面具',     lbt:  '轻型靴', lgl:  '轻型铁手套', vbl:  '重型腰带',
  sml:  '小盾',     ltp:  '轻板甲', rod:  '权杖',       crn:  '皇冠',
  hbt:  '重靴',     hgl:  '重型铁手套', lbl: '锁子甲',   kit:  '塔盾',
  brs:  '胸甲',     spea: '矛',
}

function inputCodeZh(code: string): string {
  return CRAFTED_ITEM_TYPE_ZH[code] ?? CRAFTED_INPUT_HINT[code]?.label ?? code
}

/** 格式化 prop 范围 */
function propRange(p: { min?: number | null; max?: number | null }): string {
  if (p.min != null && p.max != null) return p.min === p.max ? String(p.min) : `${p.min}-${p.max}`
  if (p.min != null) return String(p.min)
  if (p.max != null) return String(p.max)
  return ''
}

// ── 风格常量 ──
const GOLD = '#c7b377'
const MUTED = '#8a7e5f'
const TEXT = '#e8dcc4'
const LINE = '#2a2a2a'

// ── 按底材代码查找合成公式的详细材料 ──
const RUNE_NAMES: Record<string, string> = {
  r01: 'El', r02: 'Eld', r03: 'Tir', r04: 'Nef', r05: 'Eth',
  r06: 'Ith', r07: 'Tal', r08: 'Ral', r09: 'Ort', r10: 'Thul',
  r11: 'Amn', r12: 'Sol', r13: 'Shael', r14: 'Dol', r15: 'Hel',
  r16: 'Io', r17: 'Lum', r18: 'Ko', r19: 'Fal', r20: 'Lem',
  r21: 'Pul', r22: 'Um', r23: 'Mal', r24: 'Ist', r25: 'Gul',
  r26: 'Vex', r27: 'Ohm', r28: 'Lo', r29: 'Sur', r30: 'Ber',
  r31: 'Jah', r32: 'Cham', r33: 'Zod',
}
const GEM_NAMES: Record<string, string> = {
  gpb: '完美蓝宝石', gpr: '完美红宝石', gpv: '完美紫宝石',
  gpy: '完美黄宝石', gpg: '完美绿宝石', gpw: '完美钻石',
}
const CUBE_INPUTS_BY_BASE: Record<string, { rune: string; gem: string }> = {}
cubeRecipes.forEach(r => {
  const baseCode = r.inputs[0]
  if (!baseCode || CUBE_INPUTS_BY_BASE[baseCode]) return
  if (r.output === 'usetype,crf' && r.inputs.length >= 6) {
    const runeCode = r.inputs[4]
    const gemCode = r.inputs[5]
    CUBE_INPUTS_BY_BASE[baseCode] = {
      rune: RUNE_NAMES[runeCode] ? `${RUNE_NAMES[runeCode]}(${runeCode})` : runeCode?.toUpperCase() ?? '',
      gem: GEM_NAMES[gemCode] ?? gemCode?.toUpperCase() ?? '',
    }
  }
})


/** Render a crafted item card (matching RunewordCalc card style) */
function CraftedCard({ item }: { item: CraftedItem }) {
  const cat = CRAFTED_NAME_ZH[item.name]
  const baseCode = item.inputs[0]
  const baseZh = inputCodeZh(baseCode)
  const detail = CUBE_INPUTS_BY_BASE[baseCode]
  return (
    <div style={{
      padding: '10px 12px', borderRadius: 6, background: '#0c0a08',
      border: `1px solid ${cat?.border ?? LINE}`,
      display: 'flex', flexDirection: 'column', gap: 6,
      height: 'auto', minHeight: 150,
    }}>
      {/* 类别名 */}
      <div style={{ fontWeight: 600, color: cat?.color ?? TEXT, fontSize: 14 }}>
        {cat?.label ?? item.name}({item.name})
      </div>
      {/* 配方 */}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 3, fontSize: 12, color: MUTED }}>
        {item.inputs.map((code, i) => {
          const hint = CRAFTED_INPUT_HINT[code]
          return (
            <span key={i} style={{
              padding: '1px 6px', borderRadius: 3,
              background: hint ? `${hint.color}15` : '#1f1810',
              border: hint ? `1px solid ${hint.color}40` : `1px solid ${LINE}`,
              color: hint?.color ?? MUTED,
            }}>
              {inputCodeZh(code)}
              {!hint && <span style={{ fontFamily: 'monospace', marginLeft: 2, fontSize: 10 }}>({code})</span>}
            </span>
          )
        })}
      </div>
      {/* 属性 */}
      {/* 详细材料（来自合成公式） */}
      {detail && (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 3, fontSize: 12, color: '#888' }}>
          <span style={{ padding: '1px 5px', borderRadius: 3, background: 'rgba(0,0,0,0.3)', border: `1px solid #333` }}>
            <span style={{ color: '#d2c691', fontFamily: 'monospace' }}>{detail.rune}</span>
          </span>
          <span style={{ padding: '1px 5px', borderRadius: 3, background: 'rgba(0,0,0,0.3)', border: `1px solid #333` }}>
            <span style={{ color: '#e3a76f' }}>{detail.gem}</span>
          </span>
        </div>
      )}
      {item.props.length > 0 && (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 3, marginTop: 2 }}>
          {item.props.map((p, i) => {
            const zhMod = modCodeZh(p.code)
            const hasZh = zhMod !== p.code
            return (
              <span key={i} style={{
                padding: '2px 6px', borderRadius: 3, fontSize: 12,
                background: '#1f1810', display: 'inline-flex', alignItems: 'center', gap: 2,
              }}>
                <span style={{ color: GOLD }}>{zhMod}</span>
                <span style={{ color: MUTED }}> {propRange(p)}</span>
                {!hasZh && <span style={{ color: '#555', fontSize: 10, fontFamily: 'monospace' }}>{p.code}</span>}
              </span>
            )
          })}
        </div>
      )}
    </div>
  )
}

export default function CraftedItemPanel() {
  const [keyword, setKeyword] = useState('')
  const [categoryFilter, setCategoryFilter] = useState<string | null>(null)
  const [craftedCtx, setCraftedCtx] = useState<CraftedContext | null>(null)
  const [loadingCtx, setLoadingCtx] = useState(false)

  const categories = useMemo(() => {
    const set = new Set<string>()
    craftedItems.forEach(it => { if (it.spawnable) set.add(it.name) })
    return Array.from(set)
  }, [])

  const ownedCodesSet = useMemo(() =>
    craftedCtx ? new Set(craftedCtx.owned_codes.map(c => c.toLowerCase())) : null,
    [craftedCtx]
  )

  const filtered = useMemo(() => {
    const kw = keyword.trim().toLowerCase()
    return craftedItems.filter((it) => {
      if (!it.spawnable) return false
      if (kw && !it.name.toLowerCase().includes(kw) && !(CRAFTED_NAME_ZH[it.name]?.label ?? '').includes(kw)) return false
      if (categoryFilter && it.name !== categoryFilter) return false
      return true
    })
  }, [keyword, categoryFilter])

  /** 匹配仓库：该配方是否可制作 */
  const craftable = useCallback((item: CraftedItem): boolean | null => {
    if (!craftedCtx || !ownedCodesSet) return null
    const baseCode = item.inputs[0].toLowerCase()
    const baseOk = ownedCodesSet.has(baseCode)
    const magicOk = craftedCtx.has_magic
    const upgOk = craftedCtx.has_upgrades
    return baseOk && magicOk && upgOk
  }, [craftedCtx, ownedCodesSet])

  const loadContext = async () => {
    setLoadingCtx(true)
    try {
      const ctx = await tauriInvoke('get_crafted_context') as CraftedContext
      setCraftedCtx(ctx)
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      console.error('加载手工艺上下文失败:', msg)
    }
    setLoadingCtx(false)
  }

  return (
    <D2EmuCard fill
      kicker={`手工装备 · ${filtered.length} 条匹配`}
      title={<span>手工艺品 <span style={{ color: MUTED, fontSize: 14, fontWeight: 400 }}>(Crafted Items)</span></span>}
      lede="底材 + 魔法物品 + 升级材料 → 固定属性手工装备。共 4 大类：打击(Hit)、鲜血(Blood)、施法(Caster)、安全(Safety)。"
      actions={
        <i className="fa-solid fa-hammer" style={{ fontSize: 22, color: '#a76fb7', opacity: 0.7 }} />
      }
    >
      {/* 工具栏 */}
      <div className="flex flex-wrap items-end gap-3" style={{ marginBottom: 10 }}>
        {/* ── 分类 ── */}
        <div style={{ flex: '0 0 auto' }}>
          <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>
            <i className="fa-solid fa-tag" style={{ marginRight: 4 }} />分类
          </div>
          <div className="flex flex-wrap" style={{ gap: 3 }}>
            <button onClick={() => setCategoryFilter(null)}
              style={{
                padding: '2px 8px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                background: categoryFilter === null ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : 'transparent',
                border: categoryFilter === null ? '1px solid #c7b377' : `1px solid ${LINE}`,
                color: categoryFilter === null ? '#c7b377' : MUTED,
              }}>
              全部
            </button>
            {categories.map(name => {
              const cfg = CRAFTED_NAME_ZH[name]
              return (
                <button key={name} onClick={() => setCategoryFilter(categoryFilter === name ? null : name)}
                  style={{
                    padding: '2px 8px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                    background: categoryFilter === name ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : 'transparent',
                    border: categoryFilter === name ? '1px solid #c7b377' : `1px solid ${LINE}`,
                    color: categoryFilter === name ? (cfg?.color ?? GOLD) : MUTED,
                  }}>
                  {cfg?.label ?? name}({name})
                </button>
              )
            })}
          </div>
        </div>

        {/* ── 关键字搜索 ── */}
        <div style={{ flex: '1 1 160px', minWidth: 120 }}>
          <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>
            <i className="fa-solid fa-search" style={{ marginRight: 4 }} />关键字
          </div>
          <input
            type="text" placeholder="搜索名称… Hit/打击/Caster/施法"
            value={keyword} onChange={(e) => setKeyword(e.target.value)}
            style={{
              width: '100%', padding: '4px 8px', borderRadius: 4, border: `1px solid ${LINE}`,
              background: '#0a0806', color: TEXT, fontSize: 13, outline: 'none', boxSizing: 'border-box',
              height: 32,
            }} />
        </div>

        {/* ── 公式提示 ── */}
        <div style={{ flex: '0 0 auto' }}>
          <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>&nbsp;</div>
          <div style={{ color: MUTED, fontSize: 13, display: 'flex', alignItems: 'center', gap: 6, height: 32 }}>
            <span style={{ color: '#a76fb7' }}>ℹ</span>
            <span>公式: <b style={{ color: TEXT }}>底材 + 魔法 + 升级材料</b></span>
          </div>
        </div>

        {/* ── 从仓库加载 ── */}
        <div style={{ flex: '0 0 auto' }}>
          <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>&nbsp;</div>
          <div style={{ display: 'flex', gap: 4 }}>
            {craftedCtx ? (
              <button onClick={() => setCraftedCtx(null)}
                style={{
                  padding: '4px 12px', borderRadius: 4, cursor: 'pointer',
                  fontSize: 13, background: '#0a0806', border: `1px solid ${LINE}`,
                  color: TEXT, display: 'flex', alignItems: 'center', gap: 4,
                  height: 32,
                }}>
                <i className="fa-solid fa-xmark" style={{ fontSize: 12 }} />
                取消
              </button>
            ) : (
              <button onClick={loadContext} disabled={loadingCtx}
                style={{
                  padding: '4px 12px', borderRadius: 4, cursor: loadingCtx ? 'not-allowed' : 'pointer',
                  fontSize: 13, background: '#0a0806', border: `1px solid ${LINE}`,
                  color: loadingCtx ? MUTED : TEXT, display: 'flex', alignItems: 'center', gap: 4,
                  height: 32,
                }}>
                <i className={`fa-solid ${loadingCtx ? 'fa-spinner fa-spin' : 'fa-download'}`}
                  style={{ fontSize: 12 }} />
                {loadingCtx ? '加载中...' : '从仓库加载'}
              </button>
            )}
          </div>
        </div>
      </div>


      {/* 匹配状态栏 */}
      {craftedCtx && (
        <div style={{
          fontSize: 12, color: MUTED, marginBottom: 8, padding: '4px 8px',
          border: `1px solid ${LINE}`, borderRadius: 4, background: '#0a0806',
        }}>
          <span style={{ color: TEXT }}>仓库:</span>{' '}
          {craftedCtx.owned_codes.length} 种物品
          {craftedCtx.has_magic && <span style={{ color: '#5a82b4', marginLeft: 8 }}>✓ 魔法物品</span>}
          {!craftedCtx.has_magic && <span style={{ color: '#555', marginLeft: 8 }}>✗ 缺魔法物品</span>}
          {craftedCtx.has_upgrades && <span style={{ color: '#a76fb7', marginLeft: 8 }}>✓ 升级材料</span>}
          {!craftedCtx.has_upgrades && <span style={{ color: '#555', marginLeft: 8 }}>✗ 缺升级材料</span>}
        </div>
      )}

      {/* 卡片网格 */}
      <div style={{
        display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0, overflow: 'hidden',
      }}>
        <div style={{
          display: 'grid', gap: 6,
          gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
          flex: 1, minHeight: 0, overflowY: 'auto', alignContent: 'start',
        }}>
          {filtered.length === 0 && (
            <div style={{ padding: 16, color: MUTED, fontSize: 13, textAlign: 'center', gridColumn: '1 / -1' }}>
              没有匹配的手工装备。
            </div>
          )}
          {filtered.map((it) => {
            const canMake = craftable(it)
            return (
              <div key={it.id} style={{
                opacity: canMake === false ? 0.4 : 1,
                position: 'relative',
              }}>
                {canMake === false && (
                  <div style={{
                    position: 'absolute', inset: 0, zIndex: 1,
                    cursor: 'not-allowed', borderRadius: 6,
                  }} />
                )}
                <CraftedCard item={it} />
                {canMake != null && (
                  <div style={{
                    fontSize: 10, color: canMake ? '#52b465' : '#555',
                    marginTop: 2, paddingLeft: 2,
                  }}>
                    {canMake ? '✓ 仓库材料齐全' : '✗ 缺材料'}
                  </div>
                )}
              </div>
            )
          })}
        </div>
      </div>
    </D2EmuCard>
  )
}
