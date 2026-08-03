import { useMemo, useState } from 'react'
import ItemTooltip from './ItemTooltip'
import { resolveItemIcon, handleImgError } from '../utils/itemImages'
import D2EmuCard from './D2EmuCard'
import { craftedItems, cubeRecipes, CRAFTED_INPUT_HINT, translateInputCode, type CraftedItem, type CubeRecipe } from '../data/d2core/affixesIndex'
import { CUBE_RECIPE_DESC_ZH } from '../data/d2core/cubeRecipesZh'
import { modCodeZh } from '../data/d2core/affix_zh'
import { ITEM_CODES_TO_NAME } from '../data/itemNames'
import { tauriInvoke } from '../tauri'

interface CubeContext {
  owned_codes: string[]
  has_magic: boolean
  has_upgrades: boolean
}

// ── 风格常量 ──
const GOLD = '#c7b377'
const MUTED = '#8a7e5f'
const TEXT = '#e8dcc4'
const LINE = '#2a2a2a'
const ACCENT = '#52b465'

type Mode = 'all' | 'by-input' | 'by-output'

const MODE_TABS: { value: Mode; label: string; icon: string }[] = [
  { value: 'all', label: '浏览全部', icon: 'fa-list' },
  { value: 'by-input', label: '按输入查', icon: 'fa-arrow-right-to-bracket' },
  { value: 'by-output', label: '按输出查', icon: 'fa-arrow-right-from-bracket' },
]

/** 配方类别判断 & 颜色配置 */
interface CategoryStyle { label: string; bg: string; border: string; color: string; cardBg: string }

/** 按 recipe ID 硬编码分类（不依赖文字匹配） */
const CATEGORY_BY_ID: Record<string, string> = {
  // 0-2: 任务
  cube000: '任务', cube001: '任务', cube002: '任务',
  // 3-5: 药剂
  cube003: '药剂', cube004: '药剂', cube005: '药剂',
  // 6: 首饰（棱镜项链）
  cube006: '首饰',
  // 7-10: 首饰（戒指附魔）
  cube007: '首饰', cube008: '首饰', cube009: '首饰', cube010: '首饰',
  // 11-12: 装备（武器转换）
  cube011: '装备', cube012: '装备',
  // 13-14: 首饰（戒指/项链重铸）
  cube013: '首饰', cube014: '首饰',
  // 15-19: 打孔+装备
  cube015: '打孔', cube016: '打孔', cube017: '装备', cube018: '装备', cube019: '装备',
  // 20: 药剂
  cube020: '药剂',
  // 21-22: 装备（弹药转换）
  cube021: '装备', cube022: '装备',
  // 23-50: 宝石
  cube023: '宝石', cube024: '宝石', cube025: '宝石', cube026: '宝石',
  cube027: '宝石', cube028: '宝石', cube029: '宝石', cube030: '宝石',
  cube031: '宝石', cube032: '宝石', cube033: '宝石', cube034: '宝石',
  cube035: '宝石', cube036: '宝石', cube037: '宝石', cube038: '宝石',
  cube039: '宝石', cube040: '宝石', cube041: '宝石', cube042: '宝石',
  cube043: '宝石', cube044: '宝石', cube045: '宝石', cube046: '宝石',
  cube047: '宝石', cube048: '宝石', cube049: '宝石', cube050: '宝石',
  // 51-59: 符文（低级升级）
  cube051: '符文', cube052: '符文', cube053: '符文', cube054: '符文',
  cube055: '符文', cube056: '符文', cube057: '符文', cube058: '符文', cube059: '符文',
  // 60-63: 宝石+装备（重铸/打孔）
  cube060: '宝石', cube061: '装备', cube062: '装备', cube063: '打孔',
  // 64-99: 手工
  cube064: '手工', cube065: '手工', cube066: '手工', cube067: '手工',
  cube068: '手工', cube069: '手工', cube070: '手工', cube071: '手工',
  cube072: '手工', cube073: '手工', cube074: '手工', cube075: '手工',
  cube076: '手工', cube077: '手工', cube078: '手工', cube079: '手工',
  cube080: '手工', cube081: '手工', cube082: '手工', cube083: '手工',
  cube084: '手工', cube085: '手工', cube086: '手工', cube087: '手工',
  cube088: '手工', cube089: '手工', cube090: '手工', cube091: '手工',
  cube092: '手工', cube093: '手工', cube094: '手工', cube095: '手工',
  cube096: '手工', cube097: '手工', cube098: '手工', cube099: '手工',
  // 100-122: 符文（高级升级）
  cube100: '符文', cube101: '符文', cube102: '符文', cube103: '符文',
  cube104: '符文', cube105: '符文', cube106: '符文', cube107: '符文',
  cube108: '符文', cube109: '符文', cube110: '符文', cube111: '符文',
  cube112: '符文', cube113: '符文', cube114: '符文', cube115: '符文',
  cube116: '符文', cube117: '符文', cube118: '符文', cube119: '符文',
  cube120: '符文', cube121: '符文', cube122: '符文',
  // 123-126: 打孔
  cube123: '打孔', cube124: '打孔', cube125: '打孔', cube126: '打孔',
  // 127-128: 装备（修复低品质）
  cube127: '装备', cube128: '装备',
  // 129-136: 装备（独特/稀有升级）
  cube129: '装备', cube130: '装备', cube131: '装备', cube132: '装备',
  cube133: '装备', cube134: '装备', cube135: '装备', cube136: '装备',
  // 137-140: 装备（修复/充能）
  cube137: '装备', cube138: '装备', cube139: '装备', cube140: '装备',
  // 141: 打孔（清除孔）
  cube141: '打孔',
  // 147: 打孔
  cube147: '打孔',
  // 148-150: 任务（黑暗神殿/终极黑暗）
  cube148: '任务', cube149: '任务', cube150: '任务',
  // 151-154: 装备（套装升级）
  cube151: '装备', cube152: '装备', cube153: '装备', cube154: '装备',
  // 155-225: 世界石碎片（mod 特有）
  cube155: '装备', cube156: '装备', cube157: '装备', cube158: '装备',
  cube159: '装备', cube160: '装备', cube161: '装备', cube162: '装备',
  cube163: '装备', cube164: '装备', cube165: '任务',
  cube166: '装备', cube167: '装备', cube168: '装备', cube169: '装备',
  cube170: '装备', cube171: '装备', cube172: '装备', cube173: '装备',
  cube174: '装备', cube175: '装备', cube176: '装备', cube177: '任务',
  cube178: '装备', cube179: '装备', cube180: '装备', cube181: '装备',
  cube182: '装备', cube183: '装备', cube184: '装备', cube185: '装备',
  cube186: '装备', cube187: '装备', cube188: '装备', cube189: '任务',
  cube190: '装备', cube191: '装备', cube192: '装备', cube193: '装备',
  cube194: '装备', cube195: '装备', cube196: '装备', cube197: '装备',
  cube198: '装备', cube199: '装备', cube200: '装备', cube201: '任务',
  cube202: '装备', cube203: '装备', cube204: '装备', cube205: '装备',
  cube206: '装备', cube207: '装备', cube208: '装备', cube209: '装备',
  cube210: '装备', cube211: '装备', cube212: '装备', cube213: '任务',
  cube214: '装备', cube215: '装备', cube216: '装备', cube217: '装备',
  cube218: '装备', cube219: '装备', cube220: '装备', cube221: '装备',
  cube222: '装备', cube223: '装备', cube224: '装备', cube225: '任务',
  // 226: 任务
  cube226: '任务',
}


const CATEGORY_ICON: Record<string, string> = {
  '任务': 'fa-scroll', '符文': 'fa-gem', '宝石': 'fa-cube', '药剂': 'fa-kit-medical',
  '首饰': 'fa-ring', '装备': 'fa-shield-halved', '手工': 'fa-hammer', '打孔': 'fa-drill', '其他': 'fa-flask',
}
const CATEGORY_STYLES: Record<string, CategoryStyle> = {
  '任务': { label: '任务', bg: '#ff6b3515', border: '#cc4400', color: '#ff6b35', cardBg: 'linear-gradient(135deg, #1a0e08, #0d0704)' },
  '打孔': { label: '打孔', bg: '#ff8c0015', border: '#8a4a00', color: '#ff8c00', cardBg: 'linear-gradient(135deg, #1a0f05, #0d0802)' },
  '装备': { label: '装备', bg: '#6a9fd815', border: '#2a5a8a', color: '#6a9fd8', cardBg: 'linear-gradient(135deg, #080f1a, #04080d)' },
  '符文': { label: '符文', bg: '#ffd70015', border: '#b8860b', color: '#ffd700', cardBg: 'linear-gradient(135deg, #1a1508, #0d0a04)' },
  '首饰': { label: '首饰', bg: '#ff69b415', border: '#8a287a', color: '#ff69b4', cardBg: 'linear-gradient(135deg, #1a0815, #0d040a)' },
  '药剂': { label: '药剂', bg: '#52b46515', border: '#2a6a2a', color: '#52b465', cardBg: 'linear-gradient(135deg, #0a1508, #040a04)' },
  '宝石': { label: '宝石', bg: '#e3a76f15', border: '#b87333', color: '#e3a76f', cardBg: 'linear-gradient(135deg, #1a1208, #0d0a04)' },
  '手工': { label: '手工', bg: '#a76fb715', border: '#5a287a', color: '#a76fb7', cardBg: 'linear-gradient(135deg, #12081a, #08040d)' },
  '其他': { label: '其他', bg: 'transparent', border: LINE, color: MUTED, cardBg: '#0c0a08' },
}

const FILTER_CATEGORIES: { label: string; key: string }[] = [
  { label: '全部', key: '__all' },
  { label: '任务', key: '任务' },
  { label: '打孔', key: '打孔' },
  { label: '装备', key: '装备' },
  { label: '符文', key: '符文' },
  { label: '首饰', key: '首饰' },
  { label: '药剂', key: '药剂' },
  { label: '宝石', key: '宝石' },
  { label: '手工', key: '手工' },
  { label: '其他', key: '其他' },
]

/** 按底材代码查找对应手工装备属性 */
const CRAFTED_BY_BASE: Map<string, CraftedItem> = new Map()
craftedItems.forEach(ci => { CRAFTED_BY_BASE.set(ci.inputs[0], ci) })

/** 格式化 prop 范围 */
function propRange(p: { min?: number | null; max?: number | null }): string {
  if (p.min != null && p.max != null) return p.min === p.max ? String(p.min) : `${p.min}-${p.max}`
  if (p.min != null) return String(p.min)
  if (p.max != null) return String(p.max)
  return ''
}

/** 获取物品代码的中文名 */
function resolveCodeZh(code: string): string | undefined {
  if (!code || code.length < 3) return undefined
  const prefix = code.substring(0, 3)
  return ITEM_CODES_TO_NAME[prefix]?.zh
}

/** 解析输入/输出项：提取 code 和中文 */
const QUEST_ITEM_ZH: Record<string, string> = {
  qf1: '克林姆的连枷', qhr: '克林姆的心脏', qey: '克林姆的眼睛', qbr: '克林姆的大脑',
  qf2: '克林姆的意志',
  msf: '国王之杖', vip: '蝮蛇项链', hst: '赫拉迪克之杖',
  leg: '维特的腿', tbk: '城镇传送之书',
  pk1: '恐惧之钥', pk2: '憎恨之钥', pk3: '毁灭之钥',
  dhn: '暗黑破坏神之角', bey: '巴尔之眼', mbr: '墨菲斯托之脑',
  tes: '扭曲的痛苦精华', ceh: '充能的憎恨精华', bet: '燃烧的恐惧精华', fed: '溃烂的毁灭精华',
  toa: '赦免徽章',
  hpot: '治疗药剂', mpot: '法力药剂', rpot: '回复药剂',
  'The Stone of Jordan': '乔丹之石',
  // 品质/材料标记
  mag: '魔法物品', jew: '珠宝', upg: '升级材料', rar: '稀有物品', nos: '普通物品',
  low: '低品质', nor: '普通', exc: '扩展', eli: '精英', uni: '独特', set: '套装',
  mod: '扩展', bas: '基础',
  // 物品类型
  amu: '项链', amul: '项链', ring: '戒指', rin: '戒指',
  axe: '斧', swor: '剑', knif: '匕首', kri: '匕首', staf: '法杖',
  spea: '矛', pole: '长柄武器', jave: '标枪', blun: '钝击武器',
  rod: '权杖', spc: '钉头棒', scep: '权杖', mace: '钉头锤', hamm: '锤', club: '棒',
  bow: '弓', xbow: '弩', cqv: '箭矢', aqv: '箭袋', tax: '投射武器',
  helm: '头盔', hlm: '头盔', crn: '皇冠', msk: '面具',
  fhl: '高级头盔', mbt: '轻型靴', mgl: '轻型铁手套', tbl: '饰带',
  gts: '哥特盾', fld: '板甲', tbt: '重型靴', vgl: '重型铁手套',
  mbl: '锁子甲', spk: '尖刺盾牌', plt: '铠甲', hbt: '重靴', hgl: '重型铁手套',
  lbt: '轻型靴', lgl: '轻型铁手套', vbl: '重型腰带',
  sml: '小盾', ltp: '轻板甲', lbl: '锁子甲', kit: '塔盾', brs: '胸甲',
  boot: '靴子', glov: '手套', belt: '腰带',
  shie: '盾牌', tors: '护甲', weap: '武器',
  lcha: '超大护身符', mcha: '大型护身符', scha: '小护身符',
  // 宝石
  gcv: '碎裂紫宝石', gcb: '碎裂蓝宝石', gcr: '碎裂红宝石', gcg: '碎裂绿宝石', gcy: '碎裂黄宝石', gcw: '碎裂钻石',
  gfv: '裂开紫宝石', gfb: '裂开蓝宝石', gfr: '裂开红宝石', gfg: '裂开绿宝石', gfy: '裂开黄宝石', gfw: '裂开钻石',
  gzv: '完美紫宝石',
  glv: '完美的紫宝石', glb: '完美的蓝宝石', glr: '完美的红宝石', glg: '完美的绿宝石', gly: '完美的黄宝石', glw: '完美的钻石',
  gpv: '完美紫宝石', gpy: '完美黄宝石', gpb: '完美蓝宝石', gpg: '完美绿宝石', gpr: '完美红宝石', gpw: '完美钻石',
  skc: '碎裂骷髅', skf: '裂开骷髅', sku: '无暇骷髅', skl: '完美骷髅', skz: '完美骷髅',
  gemz: '骷髅', gemr: '无暇红宝石', gemd: '钻石',
  // 世界石碎片
  xa1: '西部世界石碎片', xa2: '东部世界石碎片', xa3: '南部世界石碎片',
  xa4: '深渊世界石碎片', xa5: '北部世界石碎片',
  // 操作代码
  sock: '打孔', uns: '已打孔', tsc: '城镇传送卷轴', opm: '爆炸药剂',
  rch: '修复充能', rep: '修复',
  gpl: '毒气药剂', wms: '解冻药剂', yps: '解毒药剂',
  ua1: '超级古代召唤物(Act1)', ua2: '超级古代召唤物(Act2)',
  ua3: '超级古代召唤物(Act3)', ua4: '超级古代召唤物(Act4)', ua5: '超级古代召唤物(Act5)',
  // 输出名称
  'cow portal': '奶牛关传送门',
  'pandemonium portal': '黑暗神殿传送门',
  'pandemonium finale portal': '终极黑暗传送门',
  'red portal': '巨型山峰传送门',
  'crafted rotting fissure': '腐毒腐烂裂口',
  'crafted cold rupture': '极寒冰寒破裂',
  'crafted crack of the heavens': '磁性天堂裂缝',
  'crafted flame rift': '爆燃火焰裂隙',
  'crafted bone break': '突破碎骨',
  'crafted black cleft': '神秘黑色裂隙',
}
function displayCode(code: string, lang: 'zhCN' | 'zhTW' | 'enUS' = 'zhCN'): { code: string; zh?: string } {
  if (QUEST_ITEM_ZH[code]) return { code, zh: QUEST_ITEM_ZH[code] }
  // gem codes
  if (code.startsWith('gem')) {
    const names: Record<string, string> = { gem1: '碎裂宝石', gem2: '标准宝石', gem3: '无暇宝石', gem4: '完美宝石' }
    return { code, zh: names[code] || code }
  }
  if (code === 'any') return { code: 'any', zh: '任意' }

  const zh = resolveCodeZh(code)
  if (zh) return { code, zh }
  // 可能输出包含数量
  const parts = code.split(',')
  if (parts.length > 1) {
    const first = parts[0].trim()
    const qty = parts[1].trim()
    const zhFirst = resolveCodeZh(first)
    return { code: `${first}×${qty}`, zh: zhFirst }
  }
  return { code, zh: lang === 'zhCN' ? undefined : undefined }
}

/** 带 tooltip 的物品代码标签 */
function CodeWithTooltip({ code, label, style }: { code: string; label: string; style?: React.CSSProperties }) {
  // Extract 3-char item code from label
  const itemCode = code.length === 3 ? code : (label.length === 3 ? label : undefined)
  return (
    <span style={{ position: 'relative', display: 'inline-flex' }}>
      <span style={style}>{label}</span>
      {itemCode && (
        <ItemTooltip
          tooltipData={undefined}
          tooltipLines={[]}
          quality="normal"
          itemCode={itemCode}
          nameZh={label}
          mode="hover"
          position="top"
        />
      )}
    </span>
  )
}

export default function CubeRecipePanel() {
  const [mode, setMode] = useState<Mode>('all')
  const [keyword, setKeyword] = useState('')
  const [filterCode, setFilterCode] = useState('')
  const [catFilter, setCatFilter] = useState('__all')
  const [cubeCtx, setCubeCtx] = useState<CubeContext | null>(null)
  const [loadingCtx, setLoadingCtx] = useState(false)

  const filtered = useMemo(() => {
    const kw = keyword.trim().toLowerCase()
    const fc = filterCode.trim().toLowerCase()
    return cubeRecipes.filter((r) => {
      if (!r.enabled) return false
      const desc = r.description.toLowerCase()
      if (kw && !desc.includes(kw) && !String(r.id).toLowerCase().includes(kw)) return false
      if (catFilter !== '__all' && (CATEGORY_BY_ID[r.id] ?? '其他') !== catFilter) return false
      if (mode === 'by-input' && fc) {
        if (!r.inputs.some((s) => s.toLowerCase().includes(fc))) return false
      }
      if (mode === 'by-output' && fc) {
        if (!String(r.output).toLowerCase().includes(fc)) return false
      }
      return true
    })
  }, [mode, keyword, filterCode, catFilter])

  const ownedCodesSet = useMemo(() =>
    cubeCtx ? new Set(cubeCtx.owned_codes.map(c => c.toLowerCase())) : null,
    [cubeCtx]
  )

  /** 检查配方所有输入材料是否都在仓库中 */
  function canMake(recipe: CubeRecipe): boolean | null {
    if (!ownedCodesSet) return null
    const inputCodes = recipe.inputs.filter((inp: string) => !inp.startsWith('qty=') && !inp.startsWith('pre=') && !inp.startsWith('suf=') && inp !== 'usetype' && inp !== 'useitem')
    return inputCodes.every((inp: string) => ownedCodesSet!.has(inp.toLowerCase()))
  }

  const loadContext = async () => {
    setLoadingCtx(true)
    try {
      const ctx = await tauriInvoke('get_crafted_context') as CubeContext
      setCubeCtx(ctx)
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      console.error('加载合成公式上下文失败:', msg)
    }
    setLoadingCtx(false)
  }

  const descZh = (r: CubeRecipe) => CUBE_RECIPE_DESC_ZH[r.description] || r.description
  const cat = (r: CubeRecipe): CategoryStyle => CATEGORY_STYLES[CATEGORY_BY_ID[r.id] ?? '其他']
  const icon = (r: CubeRecipe): string => CATEGORY_ICON[CATEGORY_BY_ID[r.id] ?? '其他'] || 'fa-flask'

  return (
    <D2EmuCard fill
      kicker={`合成公式 · ${filtered.length} 条匹配`}
      title="赫拉迪克方块合成公式"
      lede="全部赫拉迪克方块配方，支持分类浏览、代码搜索和中英文名查询。"
      actions={<i className="fa-solid fa-flask" style={{ fontSize: 18, color: ACCENT, opacity: 0.7 }} />}
    >
      {/* 工具栏 */}
      <div style={{ flexShrink: 0, marginBottom: 10 }}>
        {/* 模式 & 搜索行 */}
        <div className="flex flex-wrap items-end gap-3" style={{ marginBottom: 8 }}>
          <div className="flex" style={{ gap: 4 }}>
            {MODE_TABS.map(t => (
              <button key={t.value} onClick={() => setMode(t.value)}
                style={{ padding: '4px 10px', borderRadius: 4, cursor: 'pointer', fontSize: 12, display: 'inline-flex', alignItems: 'center', gap: 5,
                  background: mode === t.value ? 'linear-gradient(135deg, #1a2e1a, #2a3e2a)' : 'transparent',
                  border: mode === t.value ? `1px solid ${ACCENT}` : `1px solid ${LINE}`, color: mode === t.value ? ACCENT : MUTED }}>
                <i className={`fa-solid ${t.icon}`} />{t.label}
              </button>
            ))}
          </div>
          {mode !== 'all' && (
            <div style={{ flex: '1 1 160px', minWidth: 140 }}>
              <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>
                <i className="fa-solid fa-keyboard" style={{ marginRight: 4 }} />物品代码
              </div>
              <input type="text" placeholder={mode === 'by-input' ? '如 hpf, gld, msb' : '如 hst, r01, g01'}
                value={filterCode} onChange={(e) => setFilterCode(e.target.value)}
                style={{ width: '100%', padding: '5px 8px', borderRadius: 4, border: `1px solid ${LINE}`, background: '#0a0806', color: TEXT, fontSize: 13, outline: 'none', fontFamily: 'monospace', boxSizing: 'border-box', height: 32 }} />
            </div>
          )}
          <div style={{ flex: '1 1 200px', minWidth: 140 }}>
            <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>
              <i className="fa-solid fa-search" style={{ marginRight: 4 }} />关键字
            </div>
            <input type="text" placeholder="搜索配方名称、材料…"
              value={keyword} onChange={(e) => setKeyword(e.target.value)}
              style={{ width: '100%', padding: '5px 8px', borderRadius: 4, border: `1px solid ${LINE}`, background: '#0a0806', color: TEXT, fontSize: 13, outline: 'none', boxSizing: 'border-box', height: 32 }} />
          </div>

          {/* 从仓库加载 / 取消 */}
          <div style={{ flex: '0 0 auto' }}>
            <div style={{ fontSize: 12, color: MUTED, marginBottom: 3, fontWeight: 600 }}>&nbsp;</div>
            <div style={{ display: 'flex', gap: 4 }}>
              {cubeCtx ? (
                <button onClick={() => setCubeCtx(null)}
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
        {/* 分类过滤行 */}
        <div className="flex flex-wrap items-end gap-3" style={{ gap: 3 }}>
          {FILTER_CATEGORIES.map(c => (
            <button key={c.key} onClick={() => setCatFilter(catFilter === c.key ? '__all' : c.key)}
              style={{ padding: '3px 9px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                background: catFilter === c.key ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : 'transparent',
                border: catFilter === c.key ? `1px solid ${GOLD}` : `1px solid ${LINE}`,
                color: catFilter === c.key ? GOLD : MUTED }}>
              {c.label}
            </button>
          ))}
        </div>
      </div>

      {/* 仓库状态栏 */}
      {cubeCtx && (
        <div style={{
          fontSize: 12, color: MUTED, marginBottom: 8, padding: '4px 8px',
          border: `1px solid ${LINE}`, borderRadius: 4, background: '#0a0806',
        }}>
          <span style={{ color: TEXT }}>仓库:</span>{' '}
          {cubeCtx.owned_codes.length} 种物品
        </div>
      )}

      {/* 卡片网格 */}
      <div style={{ flex: 1, minHeight: 0, overflowY: 'auto' }}>
        {filtered.length === 0 ? (
          <div style={{ padding: 24, color: MUTED, fontSize: 14, textAlign: 'center' }}>
            没有匹配的配方。试试调整筛选条件。
          </div>
        ) : (
          <div className="grid gap-2" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(340px, 1fr))' }}>
            {filtered.map((r) => {
              const makeable = canMake(r)
              const craftedProps = CATEGORY_BY_ID[r.id] === '手工' ? CRAFTED_BY_BASE.get(r.inputs[0])?.props : undefined
              return (
                <div key={r.id} style={{
                  opacity: makeable === false ? 0.45 : 1,
                  position: 'relative',
                }}>
                  {makeable === false && (
                    <div style={{ position: 'absolute', inset: 0, zIndex: 1, cursor: 'not-allowed', borderRadius: 6 }} />
                  )}
                  <CubeRecipeCard recipe={r} descZh={descZh(r)} category={cat(r)} icon={icon(r)}
                    craftedProps={craftedProps} />
                  {makeable != null && (
                    <div style={{
                      fontSize: 10, color: makeable ? '#52b465' : '#555',
                      marginTop: 2, paddingLeft: 2,
                    }}>
                      {makeable ? '✓ 材料齐全' : '✗ 缺材料'}
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        )}
      </div>

    </D2EmuCard>
  )
}  // CubeRecipePanel
function CubeRecipeCard({ recipe, descZh, category, icon: catIcon, craftedProps }: {
  recipe: CubeRecipe; descZh: string; category: CategoryStyle; icon: string
  craftedProps?: { code: string; min?: number | null; max?: number | null; param?: number | null }[]
}) {
  const arrowIdx = descZh.indexOf('→')

  return (
    <div style={{
      padding: 12, borderRadius: 6, position: 'relative',
      background: category.cardBg,
      border: `1.5px solid ${category.border}`,
      boxShadow: category.border !== LINE ? `0 0 8px ${category.border}22` : 'none',
      transition: 'border-color 0.2s',
      height: 'auto', minHeight: 150,
    }}>
      {/* 类别徽标 */}
      <div style={{
        position: 'absolute', top: -1, right: -1,
        padding: '2px 8px', borderRadius: '0 6px 0 6px',
        background: category.border, color: '#0a0806',
        fontSize: 11, fontWeight: 700, display: 'flex', alignItems: 'center', gap: 4, zIndex: 1,
      }}>
        <i className={`fa-solid ${catIcon}`} style={{ fontSize: 10 }} />
        {category.label}
      </div>

      {/* 配方描述（中文） */}
      <div style={{ marginBottom: 8, paddingRight: 70 }}>
        <div style={{ color: TEXT, fontSize: 13, lineHeight: 1.5, fontWeight: 600 }}>
          {descZh}
        </div>
      </div>

      {/* 输入 → 输出 */}
      <div style={{ marginBottom: 6 }}>
        <div style={{ fontSize: 12, color: MUTED, marginBottom: 3 }}>
          <i className="fa-solid fa-arrow-right-to-bracket" style={{ marginRight: 4, fontSize: 10 }} />
          输入
        </div>
        <div className="flex flex-wrap" style={{ gap: 3 }}>
          {recipe.inputs
            .filter((inp: string) => !inp.startsWith('qty=') && !inp.startsWith('pre=') && !inp.startsWith('suf=') && inp !== 'usetype' && inp !== 'useitem')
            .map((inp: string, i: number) => {
              const { code, zh } = displayCode(inp)
              const label = `${code}${zh ? ` · ${zh}` : ''}`
              return (
                <CodeWithTooltip key={i} code={code} label={label} style={{
                  padding: '1px 5px', borderRadius: 3, fontFamily: 'monospace', fontSize: 12,
                  background: 'rgba(0,0,0,0.4)', border: `1px solid rgba(255,255,255,0.08)`, color: '#d2c691',
                  whiteSpace: 'nowrap',
                }} />
              )
            })}
        </div>
        </div>
      <div style={{ marginBottom: 0 }}>
        <div style={{ fontSize: 12, color: MUTED, marginBottom: 3 }}>
          <i className="fa-solid fa-arrow-right-from-bracket" style={{ marginRight: 4, fontSize: 10 }} />
          输出
        </div>
        <div className="flex items-center flex-wrap" style={{ gap: 3 }}>
          {(() => {
            const outStr = recipe.output
            if (outStr === 'usetype,crf' || outStr === 'usetype' || outStr === 'useitem') return null
            const outCode = outStr.split(',')[0].trim()
            const { zh } = displayCode(outStr)
            return (
              <CodeWithTooltip code={outCode} label={`${outCode}${zh ? ` · ${zh}` : ''}`} style={{
                padding: '2px 8px', borderRadius: 3, fontFamily: 'monospace', fontSize: 12, fontWeight: 600,
                background: 'linear-gradient(135deg, #1a2e1a, #2a3e2a)', border: `1px solid ${ACCENT}`,
                color: ACCENT, whiteSpace: 'nowrap',
              }} />
            )
          })()}
          {recipe.qty > 1 && (
            <span style={{ fontSize: 11, color: MUTED, fontFamily: 'monospace' }}>
              ×{recipe.qty}
            </span>
          )}
        </div>
      </div>

      {/* 手工固定属性（来自 crafted_items.json） */}
      {craftedProps && craftedProps.length > 0 && (
        <div style={{ marginTop: 6, paddingTop: 6, borderTop: `1px solid rgba(255,255,255,0.06)` }}>
          <div style={{ fontSize: 12, color: MUTED, marginBottom: 3 }}>
            <i className="fa-solid fa-chart-line" style={{ marginRight: 4, fontSize: 10 }} />
            固定属性
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 3 }}>
            {craftedProps.map((p, i) => {
              const zhMod = modCodeZh(p.code)
              const hasZh = zhMod !== p.code
              return (
                <span key={i} style={{
                  padding: '2px 6px', borderRadius: 3, fontSize: 12,
                  background: 'rgba(0,0,0,0.4)', display: 'inline-flex', alignItems: 'center', gap: 2,
                }}>
                  <span style={{ color: GOLD }}>{zhMod}</span>
                  <span style={{ color: MUTED }}> {propRange(p)}</span>
                  {!hasZh && <span style={{ color: '#555', fontSize: 10, fontFamily: 'monospace' }}>{p.code}</span>}
                </span>
              )
            })}
          </div>
        </div>
      )}

      {/* 底部等级信息 */}
      <div className="flex items-center" style={{ gap: 8, marginTop: 8, paddingTop: 6, borderTop: `1px solid rgba(255,255,255,0.06)` }}>
        <span style={{ fontSize: 11, color: MUTED, fontFamily: 'monospace' }}>
          lvl <span style={{ color: GOLD }}>{recipe.lvl}</span>
        </span>
        {recipe.plvl > 0 && (
          <span style={{ fontSize: 11, color: MUTED, fontFamily: 'monospace' }}>
            plvl <span style={{ color: GOLD }}>{recipe.plvl}</span>
          </span>
        )}
        {recipe.op !== 28 && recipe.op !== 0 && (
          <span style={{ fontSize: 10, color: '#555', fontFamily: 'monospace' }}>
            op <span style={{ color: '#888' }}>{recipe.op}</span>
          </span>
        )}
      </div>
    </div>
  )
}
