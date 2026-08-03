import { useState, useEffect, useRef, useMemo } from 'react'
import { tauriInvoke } from '../tauri'
import { runeWordStore } from '../cache/runewords'
import { showToast } from '../components/Toast'
import { ITEM_CODES_TO_NAME } from '../data/itemNames'
import D2EmuCard from '../components/D2EmuCard'
import RunewordStatBanner from '../components/RunewordStatBanner'
import BaseQualityFilter, { type BaseQuality } from '../components/BaseQualityFilter'
import runewordMeta from '../data/runewordMeta.json'
import {
  loadAllRunewordsFromStorage, saveAllRunewordsToStorage,
  loadRunewordContextFromStorage, saveRunewordContextToStorage,
  clearRunewordContextCache,
  type RunewordContextCache,
} from '../utils/runewordCache'

// ── Types ──
interface RunewordMeta {
  en: string
  runes: string[]
  bases: string[]
  sockets: number
  stars: number | null
  phase: string | null
  slots: string[]
  rec_best: string[]
  rec_interim: string[]
  req_lvl: number | null
  ladder_only: boolean
  max_roll: string | null
  notes: string[]
  affixes: string[]
  affixes_zh: string[]
}

type RunewordMetaMap = Record<string, RunewordMeta>

// ── Constants ──
const RUNE_NAMES: Record<string, string> = {
  r01:'El', r02:'Eld', r03:'Tir', r04:'Nef', r05:'Eth', r06:'Ith',
  r07:'Tal', r08:'Ral', r09:'Ort', r10:'Thul', r11:'Amn', r12:'Sol',
  r13:'Shael', r14:'Dol', r15:'Hel', r16:'Io', r17:'Lum', r18:'Ko',
  r19:'Fal', r20:'Lem', r21:'Pul', r22:'Um', r23:'Mal', r24:'Ist',
  r25:'Gul', r26:'Vex', r27:'Ohm', r28:'Lo', r29:'Sur', r30:'Ber',
  r31:'Jah', r32:'Cham', r33:'Zod',
}
const RUNEWORD_ZH: Record<string, string> = {
  "Ancients' Pledge": "古代人的契约", "Armageddon": "末日决战", "Authority": "权威",
  "Beast": "野兽", "Beauty": "美貌", "Black": "黑色", "Blood": "鲜血",
  "Bone": "白骨", "Bramble": "荆棘", "Brand": "烙印",
  "Breath of the Dying": "死亡呼吸", "Broken Promise": "破碎的誓言",
  "Call to Arms": "战争召唤", "Chains of Honor": "荣耀之链", "Chance": "机会",
  "Chaos": "混沌", "Crescent Moon": "新月", "Darkness": "黑暗", "Daylight": "白昼",
  "Death": "死亡", "Deception": "欺骗", "Delirium": "精神错乱", "Desire": "渴望",
  "Despair": "绝望", "Destruction": "毁灭", "Doom": "末日", "Dragon": "龙",
  "Dread": "恐惧", "Dream": "梦境", "Duress": "强制", "Edge": "边缘",
  "Elation": "得意", "Enigma": "谜团", "Enlightenment": "启迪", "Envy": "嫉妒",
  "Eternity": "永恒", "Exile": "流亡", "Faith": "信心", "Famine": "饥荒",
  "Flickering Flame": "闪烁火焰", "Fortitude": "刚毅", "Fortune": "财富",
  "Amity": "友谊", "Fury": "狂暴", "Gloom": "幽暗", "Glory": "荣耀",
  "Grief": "悔恨", "Hand of Justice": "正义之手", "Harmony": "和谐",
  "Hatred": "仇恨", "Heart of the Oak": "橡树之心", "Heaven's Will": "天堂的意志",
  "Holy Tears": "神圣之泪", "Holy Thunder": "神圣雷霆", "Honor": "荣耀",
  "Revenge": "复仇", "Humility": "谦逊", "Hunger": "饥饿", "Ice": "冰",
  "Infinity": "无限", "Innocence": "无辜", "Insight": "洞察",
  "Jealousy": "嫉妒", "Judgment": "审判", "King's Grace": "国王的恩典",
  "Kingslayer": "弑王者", "Knight's Vigil": "骑士守夜", "Knowledge": "知识",
  "Last Wish": "最后希望", "Law": "法律", "Lawbringer": "执法者", "Leaf": "叶子",
  "Lightning": "闪电", "Lionheart": "狮心", "Lore": "学识", "Love": "爱情",
  "Loyalty": "忠诚", "Lust": "欲望", "Madness": "疯狂", "Malice": "怨恨",
  "Melody": "旋律", "Memory": "记忆", "Mist": "迷雾", "Morning": "清晨",
  "Mystery": "神秘", "Myth": "神话", "Nadir": "天底", "Nature's Kingdom": "自然王国",
  "Night": "夜晚", "Oath": "誓言", "Obedience": "遵从", "Oblivion": "遗忘",
  "Obsession": "执念", "Passion": "热情", "Patience": "耐心", "Pattern": "图案",
  "Peace": "和平", "Voice of Reason": "理性之声", "Penitence": "忏悔",
  "Peril": "危险", "Pestilence": "瘟疫", "Phoenix": "凤凰", "Piety": "虔诚",
  "Pillar of Faith": "信仰之柱", "Plague": "瘟疫", "Praise": "赞美",
  "Prayer": "祈祷", "Pride": "骄傲", "Principle": "原则",
  "Prowess in Battle": "战斗技巧", "Prudence": "审慎", "Punishment": "惩罚",
  "Purity": "纯洁", "Question": "疑问", "Radiance": "光辉", "Rain": "雨",
  "Reason": "理智", "Red": "红色", "Rhyme": "韵律", "Rift": "裂缝",
  "Sanctuary": "圣堂", "Serendipity": "意外", "Shadow": "阴影",
  "Shadow of Doubt": "怀疑之影", "Silence": "寂静", "Siren's Song": "塞壬之歌",
  "Smoke": "烟雾", "Sorrow": "悲伤", "Spirit": "精神", "Splendor": "灿烂",
  "Starlight": "星光", "Stealth": "隐秘", "Steel": "钢铁",
  "Still Water": "止水", "Sting": "刺", "Stone": "石块", "Storm": "风暴",
  "Strength": "力量", "Tempest": "暴风雨", "Temptation": "诱惑", "Terror": "恐怖",
  "Thirst": "渴望", "Thought": "思想", "Thunder": "雷霆", "Time": "时间",
  "Tradition": "传统", "Treachery": "背信", "Trust": "信任", "Truth": "真理",
  "Unbending Will": "不屈的意志", "Valor": "勇气", "Vengeance": "复仇",
  "Venom": "剧毒", "Victory": "胜利", "Voice": "声音", "Void": "虚空",
  "War": "战争", "Water": "水", "Wealth": "财富", "Whisper": "低语",
  "White": "白色", "Wind": "风", "Wings of Hope": "希望之翼", "Wisdom": "智慧",
  "Woe": "悲哀", "Wonder": "奇迹", "Wrath": "愤怒", "Youth": "青春",
  "Zephyr": "和风",
  "Hustle (armor)": "催促(盔甲)", "Hustle (weapon)": "催促(武器)",
  "Mosaic": "马赛克", "Metamorphosis": "变形", "Ground": "大地",
  "Temper": "回火", "Hearth": "壁炉", "Cure": "治愈", "Bulwark": "壁垒",
  "Coven": "契约", "Vigilance": "警觉", "Ritual": "仪式",
}

const BASE_TYPE_CODES_TO_ZH: Record<string, string> = {
  shld: '盾牌', weap: '武器', tors: '护甲', miss: '远程武器', mele: '近战武器',
  h2h: '爪类', pole: '长柄武器', spea: '矛', staf: '法杖', helm: '头盔',
  axe: '斧', scep: '权杖', hamm: '锤', mace: '钉锤', club: '棒',
  swor: '剑', knif: '匕首', pala: '圣骑士盾牌', wand: '法杖（死灵）',
}

const RUNE_TIERS = [
  { label: '低级 (r01-r10)', range: [1,10] },
  { label: '中级 (r11-r20)', range: [11,20] },
  { label: '高级 (r21-r33)', range: [21,33] },
]

const BASE_TYPE_OPTIONS = Object.entries(BASE_TYPE_CODES_TO_ZH)
  .map(([code, name]) => ({ code, name }))

/** 阶段标签配置 */
const PHASE_CONFIG: Record<string, { label: string; color: string; bg: string; border: string }> = {
  farm1:  { label: '开荒过渡', color: '#b0b0b0', bg: '#1a1a1a', border: '#3a3a3a' },
  farm2:  { label: '开荒必备', color: '#4fc3f7', bg: '#0d2b3e', border: '#1565c0' },
  hot1:   { label: '后期热门', color: '#ff8a65', bg: '#2d1a0e', border: '#bf6b3a' },
  useless2: { label: '然并卵', color: '#666', bg: '#111', border: '#333' },
}

/** 查找符文之语元数据 */
function findMeta(rw: any): RunewordMeta | undefined {
  const meta = runewordMeta as RunewordMetaMap
  for (const key of Object.keys(meta)) {
    if (meta[key].en === rw.name_en) return meta[key]
  }
  return undefined
}

function localizedRuneName(code: string, lang: string): string {
  const entry = ITEM_CODES_TO_NAME[code]
  if (!entry) return RUNE_NAMES[code] || code
  let name: string
  if (lang === 'zhCN') name = entry.zh
  else if (lang === 'zhTW') name = entry.zh_tw
  else return RUNE_NAMES[code] || code
  // 去掉末尾的"符文"，只留精华名: "艾尔" 而非 "艾尔符文"
  if (name.endsWith('符文')) name = name.slice(0, -2)
  return name
}

/** 根据 language 国际化 runeword 名称(原 fix:#677 总是显示中文) */
function localizedRunewordName(rw: { name_zh?: string; name_en: string; name_zh_tw?: string }, lang: string): string {
  if (lang === 'zhCN') return rw.name_zh || RUNEWORD_ZH[rw.name_en] || rw.name_en
  if (lang === 'zhTW') return rw.name_zh_tw || rw.name_zh || RUNEWORD_ZH[rw.name_en] || rw.name_en
  return rw.name_en
}

function stripColorTags(s: string): string {
  return s.replace(/ÿc[0-9a-f]?/gi, '').trim()
}

function starsDisplay(n: number | null): string {
  if (n == null) return ''
  return '★'.repeat(n) + '☆'.repeat(5 - n)
}

/** 符文词缀：英→中缩写对照 */
const AFFIX_ABBR: [RegExp, string][] = [
  [/^str (\d.*)/, '力量 $1'],
  [/^hp%/, '生命%'],
  [/^hp\/lvl/, '生命/等级'],
  [/^mana%/, '法力%'],
  [/^dmg ([\d-]+)/, '伤害 $1'],
  [/^dmg-to-mana ([\d.]+)/, '受伤转法 $1%'],
  [/^dmg-ac /, '减防 '],
  [/^dmg-elem ([\d-]+)/, '元素伤 $1'],
  [/^dmg-max (\d+)/, '最大伤害 +$1'],
  [/^dmg-min (\d+)/, '最小伤害 +$1'],
  [/^crush (\d+)/, '粉碎打击 $1%'],
  [/^openwounds (\d+)/, '撕开伤口 $1%'],
  [/^deadly (\d+)/, '致命打击 $1%'],
  [/^pierce-(fire|cold|ltng|pois) ([\d-]+)/, '穿透$1 $2%'],
  [/^pierce-fire ([\d-]+)/, '火焰穿透 $1%'],
  [/^pierce-pois ([\d-]+)/, '毒素穿透 $1%'],
  [/^nofreeze/, '无法冰冻'],
  [/^indestruct/, '无法破坏'],
  [/^ignore-ac/, '忽视目标防御'],
  [/^ethereal/, '无形'],
  [/^reanimate (\d+)/, '复活 +$1'],
  [/^stamdrain (-?\d+)/, '耐力消耗 $1%'],
  [/^att-skill ([\d-]+)/, '攻击时施展 $1级'],
  [/^death-skill ([\d-]+)/, '死亡时施展 $1级'],
  [/^levelup-skill ([\d-]+)/, '升级时施展 $1级'],
  [/^red-dmg% ([\d.-]+)/, '物理减免 $1%'],
  [/^red-dmg (\d+)/, '物理减免 +$1'],
  [/^abs-(fire|cold|ltng|pois)% ([\d.-]+)/, '吸收$1 $2%'],
  [/^abs-(fire|cold|ltng|pois) ([\d.]+)/, '吸收$1 $2'],
  [/^extra-(fire|cold|ltng|pois) ([\d.-]+)/, '额外$1 $2'],
  [/^block (\d+)/, '格挡 +$1%'],
  [/^ac\/lvl (\d+)/, '防御/等级 +$1'],
  [/^res-pois-len (\d+)/, '毒抗 +$1%'],
  [/^fireskill (\d+)/, '火焰技能 +$1'],
  [/^move3 (\d+)/, '移动速度 +$1%'],
  [/^cheap (\d+)/, '装备需求 -$1%'],
  [/^stupidity (\d+)/, '致盲 +$1%'],
  [/^charge-noconsume (\d+)/, '不消耗充能 $1%'],
  [/^explosivearrow (\d+)/, '爆裂箭 $1级'],
]
const SKILL_EN2ZH: Record<string, string> = {
  'Tornado': '龙卷风', 'Volcano': '火山爆', 'Venom': '涂毒', 'Fade': '消隐',
  'Firestorm': '火焰风暴', 'Cyclone Armor': '气旋护甲', 'Molten Boulder': '熔岩巨石',
  'Twister': '小旋风', 'Quickness': '速攻', 'Mark of the Bear': '巨熊印记',
  'Mark of the Wolf': '恶狼印记', 'Blood Golem': '精华傀儡', 'Iron Golem': '钢铁傀儡',
  'Heart of Wolverine': '狼獾之心', 'Oak Sage': '橡木智灵', 'Summon Spirit Wolf': '召唤灵狼',
  'Spirit of Barbs': '棘灵', 'Summon Grizzly': '召唤灰熊', 'Raven': '渡鸦',
  'Delerium Change': '精神错乱变身', 'Mind Blast': '心灵爆震', 'enchant': '附魔',
  'Shape Shifting': '变形技能', 'Wearbear': '熊人变化',
}
function translateAffixZh(s: string): string {
  let r = s
  for (const [pat, repl] of AFFIX_ABBR) { r = r.replace(pat, repl) }
  for (const [en, zh] of Object.entries(SKILL_EN2ZH)) {
    r = r.replace(new RegExp(`\\b${en}\\b`, 'g'), zh)
  }
  return r
}

function hasMatchingBase(rw: any, socketedTypes: Set<string>): boolean {
  if (socketedTypes.size === 0) return false
  return (rw.allowed_bases || []).some((b: string) => socketedTypes.has(b))
}

const GOLD = 'var(--color-d2emu-gold)'
const TEXT = 'var(--color-d2emu-text)'
const MUTED = 'var(--color-d2emu-muted)'
const LINE = 'var(--color-d2emu-line)'

export default function RunewordCalc() {
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [results, setResults] = useState<any[] | null>(() => loadAllRunewordsFromStorage() ?? null)
  const [language, setLanguage] = useState('zhCN')
  const [socketedTypes, setSocketedTypes] = useState<Set<string>>(new Set())
  const [loadingContext, setLoadingContext] = useState(false)
  const [carouselIndex, setCarouselIndex] = useState(0)
  const autoLoaded = useRef(false)
  const hasSearched = useRef(false)
  const autoPausedRef = useRef(false)

  // Filter state
  const [nameFilter, setNameFilter] = useState('')
  const [requiredRunesFilter, setRequiredRunesFilter] = useState<Set<string>>(new Set())
  const [socketFilter, setSocketFilter] = useState<number | null>(null)
  const [baseTypeFilter, setBaseTypeFilter] = useState<string | null>(null)
  const [phaseFilter, setPhaseFilter] = useState<string | null>(null)
  const [minStarsFilter, setMinStarsFilter] = useState<number | null>(null)
  const [baseQualityFilter, setBaseQualityFilter] = useState<BaseQuality>('any')

  const pointerDownRune = useRef<string | null>(null)
  const dragMoved = useRef(false)
  const runewordCache = useRef<Map<string, any[]>>(new Map())

  // Load language from config on mount
  useEffect(() => {
    tauriInvoke('get_app_config')
      .then(cfg => setLanguage((cfg as any).language || 'zhCN'))
      .catch(() => {})
  }, [])

  /** 所有符文（用于默认展示全量结果） */
  const ALL_RUNES = useMemo(() =>
    Array.from({length: 33}, (_, i) => `r${String(i + 1).padStart(2, '0')}`), []
  )

  // 默认全量展示：不必等存档读取，直接算
  useEffect(() => {
    if (autoLoaded.current) return
    autoLoaded.current = true
    hasSearched.current = true

    const allKey = [...ALL_RUNES].sort().join(',')

    // 1. 同步读 localStorage → 立即填充 UI(零延迟,首屏渲染就拿到数据)
    const cachedRW = loadAllRunewordsFromStorage()
    if (cachedRW) {
      runewordCache.current.set(allKey, cachedRW)
      setResults(cachedRW)
    }
    const cachedCtx = loadRunewordContextFromStorage()
    if (cachedCtx) {
      setSocketedTypes(new Set(cachedCtx.socketed_base_types))
    }

    // 2. 后台 invoke Rust 刷新 + 持久化(仅在 cache 为空时 setState,避免覆盖用户已筛选状态)
    // Sprint 2 T1.3 call-site migration: tauriInvoke → runeWordStore.fetchResults (走 L1 cache, 10min TTL)
    runeWordStore.fetchResults(ALL_RUNES)
      .then(res => {
        const arr = res as any[]
        runewordCache.current.set(allKey, arr)
        saveAllRunewordsToStorage(arr)
        if (!cachedRW) setResults(arr)
      })
      .catch(() => {})

    tauriInvoke('get_runeword_context')
      .then(ctx => {
        const c = ctx as RunewordContextCache
        saveRunewordContextToStorage(c)
        if (!cachedCtx) setSocketedTypes(new Set(c.socketed_base_types))
      })
      .catch(() => {})
  }, [ALL_RUNES])

  // Auto re-search on rune toggle (debounced)
  useEffect(() => {
    if (hasSearched.current) {
      const timer = setTimeout(() => search(), 400)
      return () => clearTimeout(timer)
    }
  }, [selected])
  useEffect(() => {
    const up = () => {
      const code = pointerDownRune.current
      const wasDrag = dragMoved.current
      pointerDownRune.current = null
      dragMoved.current = false
      if (code && !wasDrag) {
        setSelected(prev => {
          const next = new Set(prev)
          if (next.has(code)) next.delete(code)
          else next.add(code)
          return next
        })
      }
    }
    window.addEventListener('pointerup', up)
    return () => window.removeEventListener('pointerup', up)
  }, [])


  const onRunePointerDown = (code: string) => {
    pointerDownRune.current = code
    dragMoved.current = false
  }

  /** pointerenter on a rune while button is down: drag-toggle */
  const onRunePointerEnter = (code: string) => {
    const start = pointerDownRune.current
    if (start === null) return
    if (!dragMoved.current) {
      // First time entering a different rune — drag started, toggle starting rune
      dragMoved.current = true
      setSelected(prev => {
        const next = new Set(prev)
        if (next.has(start)) next.delete(start); else next.add(start)
        if (next.has(code)) next.delete(code); else next.add(code)
        return next
      })
    } else {
      // Already dragging — toggle current rune
      setSelected(prev => {
        if (prev.has(code)) {
          if (prev.size <= 1) return prev
          const next = new Set(prev)
          next.delete(code)
          return next
        }
        const next = new Set(prev)
        next.add(code)
        return next
      })
    }
  }

  /** 从当前角色 d2s cache + stash cache 合并提取拥有的符文 */
  function extractRunesFromCharCache(): string[] {
    try {
      const savedName = localStorage.getItem('d2r-last-character')
      if (!savedName) return []
      const codes: string[] = []
      const push = (code?: string) => {
        if (!code) return
        const c = code.toLowerCase()
        if (/^r(0[1-9]|[12][0-9]|3[0-3])$/.test(c)) codes.push(c)
      }

      // 1. 角色 d2s:equipment / backpack / belt / personal_stash + 各自的 socketed_items
      const charRaw = localStorage.getItem('d2r-char-full-' + savedName)
      if (charRaw) {
        const char = JSON.parse(charRaw) as {
          equipment?: Array<{ code?: string; socketed_items?: Array<{ code?: string }> }>
          backpack_items?: Array<{ code?: string; socketed_items?: Array<{ code?: string }> }>
          belt_items?: Array<{ code?: string; socketed_items?: Array<{ code?: string }> }>
          personal_stash_items?: Array<{ code?: string; socketed_items?: Array<{ code?: string }> }>
        }
        ;(char.equipment ?? []).forEach(e => {
          push(e.code); e.socketed_items?.forEach(s => push(s.code))
        })
        ;(char.backpack_items ?? []).forEach(b => {
          push(b.code); b.socketed_items?.forEach(s => push(s.code))
        })
        ;(char.belt_items ?? []).forEach(b => {
          push(b.code); b.socketed_items?.forEach(s => push(s.code))
        })
        // 个人仓库 (Page=5):d2s 文件内嵌的 16x16 stash
        ;(char.personal_stash_items ?? []).forEach(p => {
          push(p.code); p.socketed_items?.forEach(s => push(s.code))
        })
      }

      // 2. 共享仓库 (d2i):CharacterPanel fetchStash 时已写入
      const stashRaw = localStorage.getItem('d2r-char-stash-' + savedName)
      if (stashRaw) {
        const stash = JSON.parse(stashRaw) as {
          items?: Array<{ code?: string; socketed_items?: Array<{ code?: string }> }>
        }
        ;(stash.items ?? []).forEach(it => {
          push(it.code); it.socketed_items?.forEach(s => push(s.code))
        })
      }

      return Array.from(new Set(codes)).sort()
    } catch { return [] }
  }

  /** 从仓库加载符文和底材(同步读角色 d2s cache + invoke 拿底材) */
  const loadContext = async () => {
    setLoadingContext(true)
    // invalidate 旧 context cache,确保拿到的是当前存档最新数据
    clearRunewordContextCache()
    try {
      // 1. 同步从当前角色 d2s cache 提取符文(零延迟)
      const runesFromChar = extractRunesFromCharCache()

      // 2. invoke Rust 拿底材 + 仓库符文
      // Sprint 2 T1.3: tauriInvoke → runeWordStore.fetchContext (L1 + L2 cache)
      const ctx = await runeWordStore.fetchContext() as RunewordContextCache

      // 3. 合并 runes(角色 + 仓库收藏),去重排序
      const merged = Array.from(new Set([...runesFromChar, ...ctx.owned_runes])).sort()
      saveRunewordContextToStorage({
        owned_runes: merged,
        socketed_base_types: ctx.socketed_base_types,
      })

      if (merged.length > 0) {
        setSelected(new Set(merged))
        const fromChar = runesFromChar.length
        const fromWh = merged.length - fromChar
        const hint = fromChar > 0 && fromWh > 0
          ? `已加载 ${merged.length} 个符文 (角色 ${fromChar} + 仓库 ${fromWh})`
          : fromChar > 0 ? `已加载 ${merged.length} 个符文 (来自当前角色)`
          : `已加载 ${merged.length} 个符文 (来自仓库收藏)`
        showToast(hint, 'success')
      } else {
        showToast('当前角色和仓库中均无符文', 'info')
      }
      setSocketedTypes(new Set(ctx.socketed_base_types))
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      showToast(msg || '加载失败', 'error')
    }
    setLoadingContext(false)
  }
  const search = async () => {
    hasSearched.current = true
    console.log('loading:', true)
    try {
      const ownedRunes = selected.size > 0 ? Array.from(selected) : ALL_RUNES

      // 拿全量符文之语（缓存）
      const allKey = [...ALL_RUNES].sort().join(',')
      let allRes = runewordCache.current.get(allKey)
      if (!allRes) {
        // Sprint 2 T1.3: tauriInvoke → runeWordStore.fetchResults (走 L1 cache)
        allRes = await runeWordStore.fetchResults(ALL_RUNES) as any[]
        runewordCache.current.set(allKey, allRes)
      }

      // 筛选出包含所选符文的结果
      let matched = selected.size > 0
        ? allRes.filter((rw: any) => (rw.runes as string[]).some(r => ownedRunes.includes(r)))
        : allRes


      setResults(matched)
      if (matched.length === 0) showToast('没有包含所选符文的符文之语', 'info')
      else showToast(`找到 ${matched.length} 个符文之语`, 'success')
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      showToast(msg || '查询失败', 'error')
    }
    console.log('loading:', false)
  }

  /** Filtered + sorted results */
  const filteredResults = useMemo(() => {
    if (!results) return null
    let list = results

    const q = nameFilter.trim().toLowerCase()
    if (q) {
      list = list.filter(rw => {
        const zh = (rw.name_zh || RUNEWORD_ZH[rw.name_en] || '') ? stripColorTags(rw.name_zh || RUNEWORD_ZH[rw.name_en] || '').toLowerCase() : ''
        const en = rw.name_en.toLowerCase()
        return zh.includes(q) || en.includes(q)
      })
    }
    if (requiredRunesFilter.size > 0) {
      list = list.filter(rw => {
        const codes = new Set(rw.runes as string[])
        for (const r of requiredRunesFilter) {
          if (!codes.has(r)) return false
        }
        return true
      })
    }
    if (socketFilter !== null) {
      list = list.filter(rw => rw.sockets === socketFilter)
    }
    if (baseTypeFilter) {
      list = list.filter(rw => (rw.allowed_bases as string[]).includes(baseTypeFilter))
    }
    if (phaseFilter) {
      list = list.filter(rw => {
        const meta = findMeta(rw)
        return meta?.phase === phaseFilter
      })
    }
    if (minStarsFilter !== null) {
      list = list.filter(rw => {
        const meta = findMeta(rw)
        return meta?.stars != null && meta.stars >= minStarsFilter
      })
    }
    if (baseQualityFilter !== 'any') {
      // metadata 缺失 → 该符文之语忽略品质过滤 (向后兼容)
      // metadata 存在 → 仅当 qualityFilter ∈ base_quality 时保留
      list = list.filter(rw => {
        const meta = findMeta(rw)
        const ok = (meta as unknown as { base_quality?: BaseQuality[] })?.base_quality
        if (!ok || ok.length === 0) return true
        return ok.includes(baseQualityFilter)
      })
    }

    // Sort: 🟡暗金(符文齐全+有底材) > 🟢绿(符文齐全) > 🔵蓝(有底材) > ⚫暗，同组内按星级降序
    list.sort((a, b) => {
      const metaA = findMeta(a)
      const metaB = findMeta(b)
      const aRunes = a.runes as string[]
      const bRunes = b.runes as string[]
      const hasBaseA = hasMatchingBase(a, socketedTypes)
      const hasBaseB = hasMatchingBase(b, socketedTypes)
      const allRunesA = aRunes.every(r => selected.has(r))
      const allRunesB = bRunes.every(r => selected.has(r))
      const tierA = hasBaseA && allRunesA ? 3 : !hasBaseA && allRunesA ? 2 : hasBaseA && !allRunesA ? 1 : 0
      const tierB = hasBaseB && allRunesB ? 3 : !hasBaseB && allRunesB ? 2 : hasBaseB && !allRunesB ? 1 : 0
      if (tierA !== tierB) return tierB - tierA
      const starA = metaA?.stars ?? -1
      const starB = metaB?.stars ?? -1
      return starB - starA
    })
    return list
  }, [results, nameFilter, requiredRunesFilter, socketFilter, baseTypeFilter, phaseFilter, minStarsFilter, socketedTypes, selected])

  /** Top picks: 用于推荐轮播 */
  const topPicks = useMemo(() => {
    if (!results || filteredResults === null) return []
    const candidates = filteredResults.slice(0, 6)
    const withBase = candidates.filter(rw => hasMatchingBase(rw, socketedTypes))
    if (withBase.length >= 3) return withBase.slice(0, 3)
    const withoutBase = candidates.filter(rw => !hasMatchingBase(rw, socketedTypes))
    return [...withBase, ...withoutBase].slice(0, 3)
  }, [results, filteredResults, socketedTypes])

  // Carousel auto-rotate
  useEffect(() => {
    if (!results || !topPicks.length || topPicks.length <= 1) return
    const timer = setInterval(() => {
      if (!autoPausedRef.current) {
        setCarouselIndex(prev => (prev + 1) % topPicks.length)
      }
    }, 4000)
    return () => clearInterval(timer)
  }, [results, topPicks.length])


  const toggleRequiredRune = (code: string) => {
    setRequiredRunesFilter(prev => {
      const next = new Set(prev)
      if (next.has(code)) next.delete(code)
      else next.add(code)
      return next
    })
  }

  const clearFilters = () => {
    setNameFilter('')
    setRequiredRunesFilter(new Set())
    setSocketFilter(null)
    setBaseTypeFilter(null)
    setPhaseFilter(null)
    setMinStarsFilter(null)
    setBaseQualityFilter('any')
  }

  const socketOptions = useMemo(() => {
    if (!results) return []
    const set = new Set<number>()
    results.forEach(rw => set.add(rw.sockets))
    return Array.from(set).sort((a, b) => a - b)
  }, [results])

  const hasAnyFilter = nameFilter || requiredRunesFilter.size > 0 || socketFilter !== null || baseTypeFilter || phaseFilter || minStarsFilter !== null || baseQualityFilter !== 'any'

  /** Render a runeword card (reusable for results + carousel) */
  const renderRuneCard = (rw: any, opts?: { compact?: boolean }) => {
    const meta = findMeta(rw)
    const phaseCfg = meta?.phase ? PHASE_CONFIG[meta.phase] : null
    const hasBase = hasMatchingBase(rw, socketedTypes)
    const hasAllRunes = (rw.runes as string[]).every(r => selected.has(r))
    const isCompact = opts?.compact

    const isGold = hasBase && hasAllRunes    // 符文齐全 + 有底材
    const isBlue = hasBase && !hasAllRunes    // 有底材，缺符文
    const isGreen = hasAllRunes && !hasBase   // 符文齐全，缺底材

    return (
      <div key={rw.name_en} style={{
        padding: isCompact ? 10 : 12, borderRadius: 6, position: 'relative',
        background: isGold ? 'linear-gradient(135deg, #1f1908, #181206)'
                  : isBlue ? 'linear-gradient(135deg, #0d1520, #0a0f18)'
                  : isGreen ? 'linear-gradient(135deg, #0d1f0d, #0a1806)'
                  : '#0a0806',
        border: isGold ? '1.5px solid #c7b377'
              : isBlue ? '1.5px solid #4a7db5'
              : isGreen ? '1.5px solid #2e8b57'
              : '1px solid #2a2a2a',
        boxShadow: isGold ? '0 0 8px rgba(199,179,119,0.25)'
                  : isBlue ? '0 0 8px rgba(74,125,181,0.2)'
                  : isGreen ? '0 0 8px rgba(46,139,87,0.2)'
                  : 'none',
      }}>
        {(isGold || isBlue || isGreen) && meta?.stars && meta.stars >= 4 && (
          <div style={{
            position: 'absolute', top: -7, left: -5,
            fontSize: 28, lineHeight: 1,
            color: '#ffd700', textShadow: '0 0 8px rgba(255,215,0,0.7), 0 0 3px rgba(0,0,0,0.8)',
            zIndex: 1, pointerEvents: 'none',
          }}>★</div>
        )}
        {isGold && (
          <div style={{
            position: 'absolute', top: -1, right: -1,
            padding: '2px 8px', borderRadius: '0 6px 0 6px',
            background: '#c7b377', color: '#1a1406',
            fontSize: isCompact ? 10 : 11, fontWeight: 700,
          }}>可制作</div>
        )}
        {isBlue && (
          <div style={{
            position: 'absolute', top: -1, right: -1,
            padding: '2px 8px', borderRadius: '0 6px 0 6px',
            background: '#4a7db5', color: '#fff',
            fontSize: isCompact ? 10 : 11, fontWeight: 700,
          }}>有底材</div>
        )}
        {isGreen && (
          <div style={{
            position: 'absolute', top: -1, right: -1,
            padding: '2px 8px', borderRadius: '0 6px 0 6px',
            background: '#2e8b57', color: '#fff',
            fontSize: isCompact ? 10 : 11, fontWeight: 700,
          }}>符文齐全</div>
        )}
        {/* 名称行：中文名 + 阶段 */}
        <div className="flex items-center" style={{ gap: 6, marginBottom: isCompact ? 2 : 4, flexWrap: 'wrap' }}>
          <span style={{ color: GOLD, fontWeight: 700, fontSize: isCompact ? 13 : 14 }}>
            {localizedRunewordName(rw, language)}
          </span>
          {phaseCfg && (
            <span style={{ fontSize: isCompact ? 10 : 11, padding: '1px 6px', borderRadius: 3,
              background: phaseCfg.bg, border: `1px solid ${phaseCfg.border}`, color: phaseCfg.color, whiteSpace: 'nowrap' }}>
              {phaseCfg.label}
            </span>
          )}
        </div>
        {/* 英文名 + 星数 */}
        <div className="flex items-center" style={{ gap: 6, marginBottom: isCompact ? 2 : 4, flexWrap: 'wrap' }}>
          {meta?.stars != null && (
            <span style={{ fontSize: isCompact ? 11 : 12, letterSpacing: 1,
              color: meta.stars >= 4 ? '#ff8a65' : meta.stars >= 3 ? '#c7b377' : '#666' }}>
              {starsDisplay(meta.stars)}
            </span>
          )}
          {!isCompact && (rw.name_zh || RUNEWORD_ZH[rw.name_en]) && (
            <span style={{ color: '#888', fontSize: 12 }}>{rw.name_en}</span>
          )}
        </div>
        {/* 等级 + 孔数 */}
        <div className="flex items-center" style={{ gap: 12, marginBottom: isCompact ? 2 : 4 }}>
          {meta?.req_lvl && (
            <div style={{ fontSize: isCompact ? 12 : 14, color: MUTED }}>
              <span style={{ color: TEXT }}>{meta.req_lvl}</span>级
            </div>
          )}
          <div style={{ fontSize: isCompact ? 12 : 14, color: MUTED }}>
            <span style={{ color: TEXT }}>{rw.sockets}</span>孔
          </div>
        </div>
        {/* 符文 */}
        {!isCompact && (
          <div style={{ fontSize: 14, color: MUTED, marginBottom: 4 }}>
            符文: <strong style={{ color: TEXT }}>
              {rw.runes.map((r: string, i: number) => {
                const hl = requiredRunesFilter.has(r)
                return (
                  <span key={`n-${i}`}>
                    {i > 0 && <span style={{ color: MUTED }}> + </span>}
                    <span style={{ color: hl ? GOLD : '#e8d5a3', fontWeight: hl ? 700 : 400 }}>
                      {localizedRuneName(r, language)}
                    </span>
                  </span>
                )
              })}
            </strong>
            {/* 符文代码行 */}
            <div style={{ fontSize: 12, color: '#666', fontFamily: 'Roboto Mono, monospace', marginTop: 2 }}>
              {rw.runes.map((r: string, i: number) => (
                <span key={`c-${i}`}>{i > 0 && ' + '}{r}</span>
              ))}
            </div>
          </div>
        )}
        {!isCompact && (
          <div style={{ fontSize: 14, color: MUTED }}>
            底材: <strong style={{ color: TEXT, fontSize: 14 }}>
              {rw.allowed_bases.map((c: string) => BASE_TYPE_CODES_TO_ZH[c] || c).join(', ') || '任意'}
            </strong>
          </div>
        )}
        {!isCompact && meta?.rec_best && meta.rec_best.length > 0 && (
          <div style={{ fontSize: 14, color: MUTED, marginTop: 4 }}>
            推荐底材: <strong style={{ color: '#e8d5a3', fontSize: 14 }}>{meta.rec_best.join(' / ')}</strong>
            {meta.rec_interim?.length > 0 && (
              <span style={{ color: MUTED, fontSize: 12, marginLeft: 6 }}>(过渡: {meta.rec_interim.join(' / ')})</span>
            )}
          </div>
        )}
        {!isCompact && meta?.max_roll && (
          <div style={{ fontSize: 13, color: '#c7b377', marginTop: 4, lineHeight: '1.4' }}>
            <i className="fa-solid fa-chart-simple" style={{ marginRight: 4, fontSize: 11 }} />
            满变: <span style={{ fontFamily: 'Roboto Mono, monospace' }}>{meta.max_roll}</span>
          </div>
        )}
        {!isCompact && meta?.notes && meta.notes.length > 0 && (
          <div style={{ marginTop: 4, fontSize: 12, color: '#888', lineHeight: '1.4' }}>
            {meta.notes.map((n: string, i: number) => <div key={i}>• {n}</div>)}
          </div>
        )}
        {/* 词缀 */}
        {!isCompact && (() => {
          const affixes = (language === 'zhCN' || language === 'zhTW') ? meta?.affixes_zh?.map(translateAffixZh) : meta?.affixes
          if (!affixes || affixes.length === 0) return null
          return (
            <div style={{ marginTop: 6, paddingTop: 6, borderTop: `1px solid rgba(255,255,255,0.06)` }}>
              {affixes.map((a: string, i: number) => (
                <div key={i} style={{ fontSize: 12, color: TEXT, lineHeight: '1.5', opacity: 0.85 }}>• {a}</div>
              ))}
            </div>
          )
        })()}
      </div>
    )
  }

  return (
    <div className="font-d2emu-ui" style={{
      display: 'grid', gap: 12, userSelect: 'none',
      gridTemplateColumns: 'minmax(400px, 500px) 1fr',
      gridTemplateRows: 'auto 1fr',
      flexGrow: 1, minHeight: 0,
      overflow: 'hidden',
    }}>
      <div style={{ display: 'contents', maxHeight: 85, overflow: 'hidden' }}>
      <RunewordStatBanner mode="stats" ownedRunes={selected} results={results ?? []} language={language} />
      <RunewordStatBanner mode="bottlenecks" ownedRunes={selected} results={results ?? []} language={language} />
      </div>
      <div className="left-col-rune" style={{
        overflowY: 'auto', scrollbarWidth: 'none',
      }}>
      <style>{`
        .left-col-rune::-webkit-scrollbar { display: none; }
      `}</style>
        <D2EmuCard title="符文之语计算器"
          lede="选择你拥有的符文，查看可以制作的符文之语。"
          actions={<i className="fa-solid fa-wand-magic-sparkles" style={{ fontSize: 22, color: GOLD, opacity: 0.7 }} />}
        >
          {RUNE_TIERS.map(tier => (
            <div key={tier.label} style={{ marginBottom: 12 }}>
              <div style={{ fontSize: 14, color: MUTED, marginBottom: 6, fontWeight: 600 }}>{tier.label}</div>
              <div className="flex flex-wrap justify-center gap-1.5">
                {Array.from({ length: tier.range[1] - tier.range[0] + 1 }, (_, i) => {
                  const idx = tier.range[0] + i
                  const code = `r${String(idx).padStart(2, '0')}`
                  const name = RUNE_NAMES[code] || code
                  const isOn = selected.has(code)
                  return (
                    <button key={code}
                      onPointerDown={() => onRunePointerDown(code)}
                      onPointerEnter={() => onRunePointerEnter(code)}
                      className="transition-all duration-75"
                      style={{
                        width: 52, height: 52, borderRadius: 6, cursor: 'pointer',
                        background: isOn ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : '#0a0806',
                        border: isOn ? '2px solid #c7b377' : '1px solid #2a2a2a',
                        color: isOn ? '#c7b377' : MUTED,
                        font: '600 14px/1 Roboto Mono, monospace',
                        display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
                        gap: 2, padding: 0,
                      }}>
                      <span style={{ fontSize: 14 }}>{name.slice(0, 3)}</span>
                      <span style={{ fontSize: 14, opacity: 0.6 }}>{code}</span>
                    </button>
                  )
                })}
              </div>
            </div>
          ))}
          <div className="flex gap-3 items-center" style={{ marginTop: 12 }}>
            <span style={{ fontSize: 14, color: MUTED }}>已选 {selected.size} 个符文</span>
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={loadContext} disabled={loadingContext}>
              <i className={`fa-solid ${loadingContext ? 'fa-spinner fa-spin' : 'fa-download'}`} />
              {' '}{loadingContext ? '加载中...' : '从仓库加载'}
            </button>
            <div style={{ flex: 1 }} />
            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={() => { setSelected(new Set()); setSocketedTypes(new Set()) }}
              title="清空所有符文选择">
              <i className="fa-solid fa-eraser" /> 清空全部
            </button>
          </div>
        </D2EmuCard>

        {/* 推荐轮播（左上→左下） */}
        {topPicks.length > 0 && (
          <D2EmuCard kicker="推荐"
            actions={
              <span style={{ color: MUTED, fontSize: 12 }}>
                {carouselIndex + 1}/{topPicks.length}
              </span>
            }
          >
            <div style={{ position: 'relative', minHeight: 200, borderRadius: 6 }}
              onMouseEnter={() => { autoPausedRef.current = true }}
              onMouseLeave={() => { autoPausedRef.current = false }}>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr' }}>
                {topPicks.map((rw, i) => (
                  <div key={rw.name_en} style={{
                    gridColumn: '1 / -1', gridRow: '1 / -1',
                    opacity: i === carouselIndex ? 1 : 0,
                    pointerEvents: i === carouselIndex ? 'auto' : 'none',
                    transition: 'opacity 0.3s ease',
                  }}>
                    {renderRuneCard(rw)}
                  </div>
                ))}
              </div>
            </div>

            {topPicks.length > 1 && (
              <div className="flex items-center" style={{ gap: '0.6vh', justifyContent: 'center', marginTop: '0.8vh' }}>
                {topPicks.map((_, i) => (
                  <button key={i} onClick={() => setCarouselIndex(i)}
                    style={{
                      width: 8, height: 8, borderRadius: '50%', border: 'none', cursor: 'pointer',
                      padding: 0,
                      background: i === carouselIndex ? '#c7b377' : '#2a2a2a',
                      transition: 'background 0.3s',
                    }} />
                ))}
              </div>
            )}

            {topPicks.length > 1 && (
              <div className="flex items-center" style={{ gap: '0.4vh', justifyContent: 'center', marginTop: '0.6vh' }}>
                <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-xs"
                  onClick={() => setCarouselIndex(prev => (prev - 1 + topPicks.length) % topPicks.length)}>
                  <i className="fa-solid fa-chevron-left" />
                </button>
                <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-xs"
                  onClick={() => setCarouselIndex(prev => (prev + 1) % topPicks.length)}>
                  <i className="fa-solid fa-chevron-right" />
                </button>
              </div>
            )}
          </D2EmuCard>
        )}
      </div>

      {/* ── 右列：筛选 + 搜索结果 / 默认占位 ── */}
      <div style={{
        display: 'flex', flexDirection: 'column', gap: 12,
        minHeight: 0, overflow: 'hidden',
      }}>
        {results && results.length > 0 && (
          <div style={{ flexShrink: 0 }}>
            <D2EmuCard kicker={`筛选后 ${filteredResults!.length} / ${results.length} 个结果`} title="筛选"
              lede="按条件缩小范围。"
              actions={hasAnyFilter ? (
                <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={clearFilters}>
                  <i className="fa-solid fa-rotate" /> 清除
                </button>
              ) : undefined}
            >
              <div className="flex items-end gap-3" style={{ marginBottom: 10 }}>
                <div style={{ flex: '1 1 140px', minWidth: 120 }}>
                  <label style={{ fontSize: 12, color: MUTED, display: 'block', marginBottom: 3, fontWeight: 600 }}>
                    <i className="fa-solid fa-search" style={{ marginRight: 4 }} />名称
                  </label>
                  <input type="text" placeholder="输入名称…"
                    value={nameFilter} onChange={e => setNameFilter(e.target.value)}
                    style={{
                      width: '100%', padding: '5px 8px', borderRadius: 4, border: `1px solid ${LINE}`,
                      background: '#0a0806', color: TEXT, fontSize: 13, outline: 'none', boxSizing: 'border-box',
                    }} />
                </div>
                <div style={{ flex: '0 0 auto' }}>
                  <label style={{ fontSize: 12, color: MUTED, display: 'block', marginBottom: 3, fontWeight: 600 }}>
                    <i className="fa-solid fa-tag" style={{ marginRight: 4 }} />分类
                  </label>
                  <div className="flex flex-wrap" style={{ gap: 3 }}>
                    {Object.entries(PHASE_CONFIG).map(([key, cfg]) => (
                      <button key={key} onClick={() => setPhaseFilter(phaseFilter === key ? null : key)}
                        style={{
                          padding: '2px 7px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                          background: phaseFilter === key ? cfg.bg : 'transparent',
                          border: phaseFilter === key ? `1px solid ${cfg.color}` : `1px solid ${LINE}`,
                          color: phaseFilter === key ? cfg.color : MUTED,
                        }}>
                        {cfg.label}
                      </button>
                    ))}
                  </div>
                </div>
                <div style={{ flex: '0 0 auto' }}>
                  <label style={{ fontSize: 12, color: MUTED, display: 'block', marginBottom: 3, fontWeight: 600 }}>
                    <i className="fa-solid fa-star" style={{ marginRight: 4 }} />最低星级
                  </label>
                  <div className="flex flex-wrap" style={{ gap: 3 }}>
                    {[1,2,3,4,5].map(n => (
                      <button key={n} onClick={() => setMinStarsFilter(minStarsFilter === n ? null : n)}
                        style={{
                          padding: '2px 7px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                          background: minStarsFilter === n ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : 'transparent',
                          border: minStarsFilter === n ? '1px solid #c7b377' : `1px solid ${LINE}`,
                          color: minStarsFilter === n ? '#c7b377' : MUTED,
                        }}>
                        {starsDisplay(n)}
                      </button>
                    ))}
                  </div>
                </div>
                <div style={{ flex: '0 0 auto' }}>
                  <label style={{ fontSize: 12, color: MUTED, display: 'block', marginBottom: 3, fontWeight: 600 }}>
                    <i className="fa-regular fa-circle" style={{ marginRight: 4 }} />孔数
                  </label>
                  <div className="flex flex-wrap" style={{ gap: 3 }}>
                    {socketOptions.map(n => (
                      <button key={n} onClick={() => setSocketFilter(socketFilter === n ? null : n)}
                        style={{
                          padding: '2px 9px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                          background: socketFilter === n ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : 'transparent',
                          border: socketFilter === n ? '1px solid #c7b377' : `1px solid ${LINE}`,
                          color: socketFilter === n ? '#c7b377' : MUTED,
                        }}>
                        {n}孔
                      </button>
                    ))}
                  </div>
                </div>
                <div style={{ flex: '1 1 130px', minWidth: 110 }}>
                  <label style={{ fontSize: 12, color: MUTED, display: 'block', marginBottom: 3, fontWeight: 600 }}>
                    <i className="fa-solid fa-shield" style={{ marginRight: 4 }} />底材
                  </label>
                  <select value={baseTypeFilter ?? ''} onChange={e => setBaseTypeFilter(e.target.value || null)}
                    style={{
                      width: '100%', padding: '4px 6px', borderRadius: 4, border: `1px solid ${LINE}`,
                      background: '#0a0806', color: TEXT, fontSize: 12, outline: 'none',
                    }}>
                    <option value="">全部</option>
                    {BASE_TYPE_OPTIONS.map(opt => (
                      <option key={opt.code} value={opt.code}>{opt.name}</option>
                    ))}
                  </select>
                </div>
                {/* ── 品质 (v2 P0 玩家声音 5: 防丢三连),独立一列 ── */}
                <div style={{ flex: '0 0 auto' }}>
                  <BaseQualityFilter value={baseQualityFilter} onChange={setBaseQualityFilter} />
                </div>
              </div>
              {/* 第二行：必须包含 */}
              {selected.size > 0 && (
                <div>
                  <label style={{ fontSize: 12, color: MUTED, display: 'block', marginBottom: 3, fontWeight: 600 }}>
                    <i className="fa-solid fa-gem" style={{ marginRight: 4 }} />必须包含
                  </label>
                  <div className="flex flex-wrap" style={{ gap: 3 }}>
                    {Array.from(selected).sort().map(code => {
                      const on = requiredRunesFilter.has(code)
                      return (
                        <button key={code} onClick={() => toggleRequiredRune(code)}
                          style={{
                            padding: '2px 7px', borderRadius: 4, cursor: 'pointer', fontSize: 12,
                            background: on ? 'linear-gradient(135deg, #2a1f10, #3d2e18)' : 'transparent',
                            border: on ? '1px solid #c7b377' : `1px solid ${LINE}`,
                            color: on ? '#c7b377' : MUTED, fontFamily: 'Roboto Mono, monospace',
                          }}>
                          {localizedRuneName(code, language)}({code})
                        </button>
                      )
                    })}
                  </div>
                </div>
              )}
            </D2EmuCard>
          </div>
        )}

        <div style={{ flex: 1, overflowY: 'auto', minHeight: 0 }}>
          {results && results.length > 0 ? (
            <>
            <D2EmuCard>
              {filteredResults!.length === 0 ? (
                <div style={{ color: MUTED, fontSize: 14, padding: 24, textAlign: 'center' }}>
                  没有匹配当前筛选条件的符文之语。
                </div>
              ) : (
                <div className="grid gap-2" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))' }}>
                  {filteredResults!.map(rw => renderRuneCard(rw))}
                </div>
              )}
            </D2EmuCard>
            </>
          ) : (
            <D2EmuCard>
              <div style={{ color: MUTED, fontSize: 14, padding: 24, textAlign: 'center' }}>
                选择左侧符文，查看可制作的符文之语。
              </div>
            </D2EmuCard>
          )}
        </div>
      </div>
    </div>
  )
}
