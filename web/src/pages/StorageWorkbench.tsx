import { useCallback, useEffect, useMemo, useRef, useState, type CSSProperties, type DragEvent } from 'react'

import D2ConfirmModal from '../components/D2ConfirmModal'
import D2EmuCard from '../components/D2EmuCard'
import D2EmuLoading from '../components/D2EmuLoading'
import EmptyState from '../components/EmptyState'
import ItemTooltip from '../components/ItemTooltip'
import SocketsOverlay from '../components/SocketsOverlay'
import StackablePageView from '../components/StackablePageView'
import { showToast } from '../components/Toast'
import { stashStore } from '../cache/stash'
import { warehouseStore, resolveDefault as resolveDefaultPage, setCodeDefault } from '../cache/warehouse'
import QuantityConfirmModal from '../components/QuantityConfirmModal'
import { tauriInvoke } from '../tauri'
import type { AppConfig, AutoBackupEntry, StashItem, StashPageInfo, StashResult, WarehouseItem } from '../types'
import { handleImgError, resolveItemIcon } from '../utils/itemImages'

const GRID_CELL = 44
const GRID_GAP = 1
// Must match `.storage-grid { padding: 10px; border: 1px solid }` in index.css;
// used to (a) align absolutely-positioned items with the first cell of the
// grid-template, and (b) size the scaler wrapper to include the full visual box.
const GRID_PADDING = 10
const GRID_BORDER = 1
const PAGE_SWITCH_HOVER_MS = 420
const STORAGE_CUSTOM_PAGE_KEY = 'd2r-marketplace:storage-custom-pages'
const DEFAULT_PAGE_NAME = '默认收藏'

type DragPayload =
  | { source: 'stash'; item: StashItem }
  | { source: 'warehouse'; item: WarehouseItem }

type PlacementState = {
  pageIndex: number
  x: number
  y: number
  width: number
  height: number
  valid: boolean
  reason?: string
}

type RenameDialogState = {
  kind: 'rename'
  pageName: string
  draft: string
}

type DeleteDialogState = {
  kind: 'delete'
  pageName: string
}

type PageDialogState = RenameDialogState | DeleteDialogState | null

const QUALITY_COLOR: Record<string, string> = {
  unique: '#a08030',
  set: '#45b84a',
  rare: '#c4a847',
  magic: '#5d6cff',
  superior: '#e8e8e8',
  normal: '#e8e8e8',
  crafted: '#c06820',
}

function colorForQuality(quality?: string | null): string {
  return QUALITY_COLOR[quality || 'normal'] || QUALITY_COLOR.normal
}

/** 品质中文标签(用 tooltip base_info 第一行匹配,fallback quality 原文) */
function qualityLabel(quality?: string | null): string {
  const q = (quality || 'normal').toLowerCase()
  const labels: Record<string, string> = {
    normal: '普通',
    superior: '超等',
    magic: '魔法',
    set: '套装',
    rare: '稀有',
    unique: '暗金',
    crafted: '手工',
    gem: '宝石',
    rune: '符文',
  }
  return labels[q] || q
}

/**
 * 物品种类标签:基于 code 前缀/后缀的简单启发式分类。
 * 不追求 100% 精确,只是给玩家一个视觉提示。
 */
function itemTypeLabel(code: string): string {
  if (!code) return '物品'
  // 3-char 码在前缀判大类;4-char 在末尾(最后一个数字或字母)
  const c = code.toLowerCase()
  // 武器(axe/sword/mace/hammer/bow/crossbow/staff/wand/spear/polearm/throw/javelin)
  if (/axe|swo|mac|ham|bow|xbo|stf|wnd|spc|plt|jar|thr|axe|9ax|7ax/.test(c)) return '武器'
  // 防具(armor/torso)
  if (/toa|arm|brs/.test(c)) return '胸甲'
  // 头盔
  if (/hlm|crn|skl|msk/.test(c)) return '头盔'
  // 盾牌
  if (/shd|shc|shp|kit/.test(c)) return '盾牌'
  // 手套
  if (/lgl|glt/.test(c)) return '手套'
  // 腰带
  if (/bts|blt/.test(c)) return '腰带'
  // 鞋
  if (/bts|boot/.test(c)) return '鞋子'
  // 戒指
  if (/rin/.test(c)) return '戒指'
  // 项链
  if (/amu/.test(c)) return '项链'
  // 符文(r01-r33)
  if (/^r\d{2}$/.test(c)) return '符文'
  // 宝石(gcv/gcw 等)
  if (/^g/.test(c)) return '宝石'
  // 药剂
  if (/potion|pot|hp\d|mp\d|rps/.test(c)) return '药剂'
  // 卷轴(scroll)
  if (/scr/.test(c)) return '卷轴'
  // 钥匙
  if (/key/.test(c)) return '钥匙'
  // 精华
  if (/ess|twisted|fest|burning/.test(c)) return '精华'
  // 重置卷轴
  if (/tok/.test(c)) return '重置卷轴'
  return '物品'
}

function normalizeWarehousePageName(pageName?: string | null): string {
  const normalized = pageName?.trim()
  return normalized || DEFAULT_PAGE_NAME
}

function validateWarehousePageName(name: string, existingNames: string[], currentName?: string): string | null {
  const normalized = name.trim()
  if (!normalized) return '收藏页名称不能为空'
  if (normalized.length > 64) return '收藏页名称不能超过 64 个字符'
  if (/[\r\n\t\0-\x1F]/.test(normalized)) return '收藏页名称不能包含控制字符'
  if (normalized === DEFAULT_PAGE_NAME && currentName !== DEFAULT_PAGE_NAME) return '默认收藏不可重复创建或重命名'
  if (normalized !== currentName && existingNames.includes(normalized)) return `收藏页已存在：${normalized}`
  return null
}

function formatTime(value?: string | null): string {
  if (!value) return '未知时间'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString('zh-CN', { hour12: false })
}

function matchesWarehouseQuery(item: WarehouseItem, query: string): boolean {
  const kw = query.trim().toLowerCase()
  if (!kw) return true
  return (
    item.item_name?.toLowerCase().includes(kw)
    || item.item_code?.toLowerCase().includes(kw)
    || item.tags?.toLowerCase().includes(kw)
    || item.notes?.toLowerCase().includes(kw)
    || normalizeWarehousePageName(item.page_name).toLowerCase().includes(kw)
  )
}

function buildOccupiedSet(items: StashItem[], skipId?: string): Set<string> {
  const occupied = new Set<string>()
  for (const item of items) {
    if (item.id === skipId) continue
    for (let dx = 0; dx < (item.inv_width || 1); dx++) {
      for (let dy = 0; dy < (item.inv_height || 1); dy++) {
        occupied.add(`${item.position_x + dx}-${item.position_y + dy}`)
      }
    }
  }
  return occupied
}

function validatePlacement(
  page: StashPageInfo | undefined,
  items: StashItem[],
  width: number,
  height: number,
  x: number,
  y: number,
): PlacementState {
  if (!page) return { pageIndex: 0, x, y, width, height, valid: false, reason: '共享仓库页不存在' }
  if (x < 0 || y < 0) return { pageIndex: page.index, x, y, width, height, valid: false, reason: '坐标无效' }
  if (x + width > page.grid_width || y + height > page.grid_height) {
    return { pageIndex: page.index, x, y, width, height, valid: false, reason: '超出当前页边界' }
  }
  const occupied = buildOccupiedSet(items)
  for (let dx = 0; dx < width; dx++) {
    for (let dy = 0; dy < height; dy++) {
      if (occupied.has(`${x + dx}-${y + dy}`)) {
        return { pageIndex: page.index, x, y, width, height, valid: false, reason: '目标区域有遮挡' }
      }
    }
  }
  return { pageIndex: page.index, x, y, width, height, valid: true }
}

function markerLabel(placement: PlacementState): string {
  if (placement.valid) return `可放置 · 页${placement.pageIndex + 1} · (${placement.x}, ${placement.y})`
  if (placement.reason?.includes('边界')) return `越界 · 页${placement.pageIndex + 1} · (${placement.x}, ${placement.y})`
  if (placement.reason?.includes('遮挡')) return `遮挡 · 页${placement.pageIndex + 1} · (${placement.x}, ${placement.y})`
  return `不可放置 · 页${placement.pageIndex + 1} · (${placement.x}, ${placement.y})`
}

function statusText(
  dragPayload: DragPayload | null,
  placement: PlacementState | null,
  activeStashPage: StashPageInfo | undefined,
  activeWarehousePage: string,
  submitting: boolean,
): string {
  if (submitting) return '写入中，请稍候...'
  if (!dragPayload) return '左侧拖到右侧收藏页可入仓；右侧拖回左侧网格可放回共享仓库。'
  if (dragPayload.source === 'stash') return `拖到右侧目录或当前收藏页「${activeWarehousePage}」即可存入 SQLite。`
  if (!placement || placement.pageIndex !== activeStashPage?.index) return '将仓库物品拖到左侧网格，系统会实时校验位置。'
  return markerLabel(placement)
}

export default function StorageWorkbench() {
  const [stash, setStash] = useState<StashResult | null>(null)
  const [warehouse, setWarehouse] = useState<WarehouseItem[]>([])
  const [stashLoading, setStashLoading] = useState(true)
  const [warehouseLoading, setWarehouseLoading] = useState(true)
  const [renderLimit, setRenderLimit] = useState(50)
  const sentinelRef = useRef<HTMLDivElement | null>(null)
  const [activeStashPageIndex, setActiveStashPageIndex] = useState(0)
  const [activeWarehousePage, setActiveWarehousePage] = useState(DEFAULT_PAGE_NAME)
  const [activeProfileKey, setActiveProfileKey] = useState('default')
  const [stashFile, setStashFile] = useState<string | null>(null)
  const [query, setQuery] = useState('')
  const [searchScope, setSearchScope] = useState<'all' | 'current'>('current')
  const [newPageName, setNewPageName] = useState('')
  const [customWarehousePages, setCustomWarehousePages] = useState<string[]>([])
  const [dragPayload, setDragPayload] = useState<DragPayload | null>(null)
  const [dropZoneActive, setDropZoneActive] = useState(false)
  const [activeWarehouseDropGroup, setActiveWarehouseDropGroup] = useState<string | null>(null)
  const [placement, setPlacement] = useState<PlacementState | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [selectedKey, setSelectedKey] = useState<string | null>(null)
  // Resolved per-code default page for the currently selected stash item.
  // null = 未设置,string = 当前默认页(可能等于 activeWarehousePage 也可能不等于)。
  const [defaultPageName, setDefaultPageName] = useState<string | null>(null)
  // 数量确认 modal:打开时携带触发它的 stash item + 目标 pageName。
  // pageName 为 null 表示走「存默认」入口(后端 fallback 到 per-code default)。
  const [pendingDeposit, setPendingDeposit] = useState<{
    item: StashItem
    pageName: string | null
  } | null>(null)
  const [ghostPointer, setGhostPointer] = useState<{ x: number; y: number } | null>(null)
  const [metaDraft, setMetaDraft] = useState({ tags: '', notes: '' })
  const [pageSwitchHoverIndex, setPageSwitchHoverIndex] = useState<number | null>(null)
  const [pageDialog, setPageDialog] = useState<PageDialogState>(null)
  const [deletePageMode, setDeletePageMode] = useState<'move' | 'delete'>('move')
  const [autoBackups, setAutoBackups] = useState<AutoBackupEntry[]>([])
  const [showBackupPanel, setShowBackupPanel] = useState(false)
  const [restoringBak, setRestoringBak] = useState<string | null>(null)

  /**
   * 取回暂存槽:仅在堆叠页(is_stackable=true)生效。
   * 流程:用户从右侧 SQLite 拖物品 → 落入本槽 → 槽中预览 → 点"确认"实际写入共享仓库堆叠页。
   * 设计原因:堆叠页无坐标概念,直接调 warehouse_withdraw 会让用户在没有任何反馈的情况下完成写盘;
   * 暂存槽提供"看到→确认→写盘"的明确步骤,避免误操作。
   */
  const [stagingItems, setStagingItems] = useState<Array<{ item: WarehouseItem; requestedQuantity: number }>>([])
  const [confirmingStagingId, setConfirmingStagingId] = useState<string | null>(null)
  // 数量确认 modal:打开时携带触发它的 warehouse item。用户确认后 staging 携带 quantity。
  // 与 deposit 的 pendingDeposit 共用一个 modal,但语义不同:deposit 来自 stash → 收藏;
  // withdraw 来自收藏 → stash 暂存槽。
  const [pendingWithdraw, setPendingWithdraw] = useState<WarehouseItem | null>(null)
  // 同步串行化门:React state 提交异步,快速双击可能两次都通过守卫。
  // 用 ref 在同步路径立即拦截,保证 warehouse_withdraw 不会被并发触发两次。
  const confirmingRef = useRef(false)
  // drop flash:成功落入暂存槽后,让槽位短暂高亮一下,用户感受"落下"反馈
  const [dropFlash, setDropFlash] = useState(false)
  const dropFlashTimerRef = useRef<number | null>(null)

  const gridRef = useRef<HTMLDivElement | null>(null)
  const fitRef = useRef<HTMLDivElement | null>(null)
  const pageSwitchTimerRef = useRef<number | null>(null)
  const pageSwitchTargetRef = useRef<number | null>(null)
  // Ref-based drag payload for synchronous access in onDragOver/onDrop
  // (React state updates are async and may not reflect in time for
  // subsequent dragover events fired by the browser).
  const dragPayloadRef = useRef<DragPayload | null>(null)
  const customPageStorageKey = useMemo(
    () => `${STORAGE_CUSTOM_PAGE_KEY}:${activeProfileKey}:${stashFile || stash?.stash_name || 'default'}`,
    [activeProfileKey, stash?.stash_name, stashFile],
  )

  const loadAutoBackups = useCallback(async (stashPath: string | null) => {
    if (!stashPath) { setAutoBackups([]); return }
    try {
      const stashFilename = stashPath.split(/[\\/]/).pop() || ''
      if (!stashFilename) { setAutoBackups([]); return }
      const all = await tauriInvoke('list_auto_backups') as AutoBackupEntry[]
      const filtered = all
        .filter(ab => ab.original_stash === stashFilename)
        .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
        .slice(0, 5)
      setAutoBackups(filtered)
    } catch { /* auto-backup list is non-critical */ }
  }, [])

  const loadAll = useCallback(async (force = false) => {
    setStashLoading(true)
    setWarehouseLoading(true)
    try {
      try {
        const updated = await tauriInvoke('warehouse_backfill_dims') as number
        if (updated > 0) {
          warehouseStore.invalidateAll()
          console.info('[StorageWorkbench] backfilled inv_width/inv_height for ${updated} warehouse rows')
        }
      } catch (e) {
        console.warn('[StorageWorkbench] warehouse_backfill_dims failed (non-fatal):', e)
      }
      // Load warehouse (fast - SQLite) in parallel, show ASAP
      const warehousePromise = warehouseStore.search({}, { force }).then(data => {
        setWarehouse(data || [])
        setWarehouseLoading(false)
        return data
      })
      // Load stash (slow - d2i parse) in parallel, show loading on left panel
      const stashPromise = stashStore.fetch('shared', { force }).then(stashData => {
        const stashPath = stashData.stash_file
        setStash(stashData)
        setStashFile(stashPath ?? stashData.stash_file)
        const appConfigPromise = tauriInvoke('get_app_config') as Promise<AppConfig>
        appConfigPromise.then(cfg => setActiveProfileKey(cfg.profile_key || 'default')).catch(() => {})
        setActiveStashPageIndex(prev => stashData.pages.some(page => page.index === prev) ? prev : (stashData.pages[0]?.index ?? 0))
        loadAutoBackups(stashPath ?? stashData.stash_file)
        setStashLoading(false)
        return stashData
      })
      await Promise.all([warehousePromise, stashPromise])
    } catch (error: unknown) {
      console.error('[StorageWorkbench] loadAll failed:', error)
      showToast(error instanceof Error ? error.message : String(error), 'error', { position: 'top' })
      setStashLoading(false)
      setWarehouseLoading(false)
    }
  }, [loadAutoBackups])

  const refreshAfterWrite = useCallback(async () => {
    stashStore.invalidateAll()
    warehouseStore.invalidateAll()
    await loadAll(true)
  }, [loadAll])

  useEffect(() => { loadAll() }, [loadAll])

  useEffect(() => {
    try {
      const raw = window.localStorage.getItem(customPageStorageKey)
      if (!raw) {
        setCustomWarehousePages([])
        return
      }
      const parsed = JSON.parse(raw)
      if (Array.isArray(parsed)) {
        const next = parsed
          .map(v => String(v).trim())
          .filter(Boolean)
          .filter(v => v !== DEFAULT_PAGE_NAME)
        setCustomWarehousePages(Array.from(new Set(next)))
      }
    } catch {
      // Ignore malformed custom pages.
      setCustomWarehousePages([])
    }
  }, [customPageStorageKey])

  useEffect(() => {
    window.localStorage.setItem(customPageStorageKey, JSON.stringify(customWarehousePages))
  }, [customPageStorageKey, customWarehousePages])

  // Debug: log any dragstart to find what the browser is dragging
  useEffect(() => {
    const handler = (e: globalThis.DragEvent) => {
      const target = e.target as HTMLElement | null
      console.log('[drag] GLOBAL dragstart: target=', target?.tagName,
        'class=', target?.className?.slice(0, 60),
        'id=', target?.id || '(none)',
        'data-code=', target?.getAttribute('data-code'),
      )
    }
    window.addEventListener('dragstart', handler)
    return () => window.removeEventListener('dragstart', handler)
  }, [])

  // Track raw cursor for stash→warehouse (ghost follows cursor) vs
  // warehouse→stash (ghost follows grid-aligned placement via updatePlacementFromPointer).
  useEffect(() => {
    if (!dragPayload) {
      setGhostPointer(null)
      return
    }
    // Only the stash→warehouse path uses raw cursor tracking.
    // For warehouse→stash the ghostPointer is kept in sync by updatePlacementFromPointer.
    if (dragPayload.source !== 'stash') return
    const onDragOver = (event: globalThis.DragEvent) => setGhostPointer({ x: event.clientX, y: event.clientY })
    const onDrop = () => setGhostPointer(null)
    window.addEventListener('dragover', onDragOver)
    window.addEventListener('drop', onDrop)
    return () => {
      window.removeEventListener('dragover', onDragOver)
      window.removeEventListener('drop', onDrop)
    }
  }, [dragPayload])

  useEffect(() => () => {
    if (pageSwitchTimerRef.current) window.clearTimeout(pageSwitchTimerRef.current)
  }, [])

  const activeStashPage = useMemo(
    () => stash?.pages.find(page => page.index === activeStashPageIndex) ?? stash?.pages[0],
    [stash?.pages, activeStashPageIndex],
  )

  // Grid auto-fit to available space (avoids vertical scrollbar on 16x16 stashes).
  // Computes an actual cell pixel size so the grid layout, item positions, and
  // visual box all agree — no `transform: scale()` (which would mismatch layout vs visual).
  // Must run AFTER activeStashPage declaration to avoid TDZ ReferenceError.
  const activeGridWidth = activeStashPage?.grid_width ?? 0
  const activeGridHeight = activeStashPage?.grid_height ?? 0
  const MIN_CELL = 8
  const [scaledCell, setScaledCell] = useState(GRID_CELL)
  useEffect(() => {
    const el = fitRef.current
    if (!el || activeGridWidth === 0 || activeGridHeight === 0) return
    const recompute = () => {
      const cw = el.clientWidth
      const ch = el.clientHeight
      if (cw <= 0 || ch <= 0) return
      const availW = cw - 2 * GRID_PADDING - 2 * GRID_BORDER
      const availH = ch - 2 * GRID_PADDING - 2 * GRID_BORDER
      if (availW <= 0 || availH <= 0) return
      const cellW = (availW - (activeGridWidth - 1) * GRID_GAP) / activeGridWidth
      const cellH = (availH - (activeGridHeight - 1) * GRID_GAP) / activeGridHeight
      setScaledCell(Math.max(MIN_CELL, Math.min(GRID_CELL, Math.floor(Math.min(cellW, cellH)))))
    }
    recompute()
    const ro = new ResizeObserver(recompute)
    ro.observe(el)
    return () => ro.disconnect()
  }, [activeGridWidth, activeGridHeight])
  const scaledPitch = scaledCell + GRID_GAP
  const scaledGridW = scaledCell * activeGridWidth + Math.max(0, activeGridWidth - 1) * GRID_GAP + 2 * GRID_PADDING + 2 * GRID_BORDER
  const scaledGridH = scaledCell * activeGridHeight + Math.max(0, activeGridHeight - 1) * GRID_GAP + 2 * GRID_PADDING + 2 * GRID_BORDER

  const activeStashItems = useMemo(
    () => (stash?.items || []).filter(item => item.page_index === activeStashPageIndex),
    [stash?.items, activeStashPageIndex],
  )

  const warehouseItemsByPage = useMemo(() => {
    const map = new Map<string, WarehouseItem[]>()
    for (const item of warehouse) {
      const pageName = normalizeWarehousePageName(item.page_name)
      if (!map.has(pageName)) map.set(pageName, [])
      map.get(pageName)!.push(item)
    }
    return map
  }, [warehouse])

  const warehousePageOrder = useMemo(() => {
    const dynamicPages = Array.from(warehouseItemsByPage.keys())
      .filter(page => page !== DEFAULT_PAGE_NAME && !customWarehousePages.includes(page))
      .sort((a, b) => a.localeCompare(b, 'zh-CN'))
    return [DEFAULT_PAGE_NAME, ...customWarehousePages, ...dynamicPages]
  }, [customWarehousePages, warehouseItemsByPage])

  useEffect(() => {
    if (!warehousePageOrder.includes(activeWarehousePage)) {
      setActiveWarehousePage(warehousePageOrder[0] || DEFAULT_PAGE_NAME)
    }
  }, [activeWarehousePage, warehousePageOrder])

  const currentWarehouseItems = useMemo(
    () => [...(warehouseItemsByPage.get(activeWarehousePage) || [])]
      .sort((a, b) => b.imported_at.localeCompare(a.imported_at)),
    [activeWarehousePage, warehouseItemsByPage],
  )

  const filteredWarehouseItems = useMemo(() => {
    const source = searchScope === 'all' ? warehouse : currentWarehouseItems
    return [...source]
      .filter(item => matchesWarehouseQuery(item, query))
      .sort((a, b) => b.imported_at.localeCompare(a.imported_at))
  }, [currentWarehouseItems, query, searchScope, warehouse])

  // 分页加载:每当 sentinel 进入可视区,增加渲染上限
  useEffect(() => {
    const el = sentinelRef.current
    if (!el) return
    const io = new IntersectionObserver(entries => {
      if (entries[0].isIntersecting && renderLimit < filteredWarehouseItems.length) {
        setRenderLimit(prev => Math.min(prev + 50, filteredWarehouseItems.length))
      }
    }, { rootMargin: '200px' })
    io.observe(el)
    return () => io.disconnect()
  }, [renderLimit, filteredWarehouseItems.length])

  // 搜索/翻页时重置渲染上限
  useEffect(() => {
    setRenderLimit(50)
  }, [query, searchScope, activeWarehousePage])

  const displayWarehouseItems = useMemo(
    () => filteredWarehouseItems.slice(0, renderLimit),
    [filteredWarehouseItems, renderLimit],
  )

  const selectedWarehouseItem = useMemo(() => {
    if (!selectedKey?.startsWith('warehouse:')) return null
    const id = selectedKey.slice('warehouse:'.length)
    return warehouse.find(item => item.id === id) ?? null
  }, [selectedKey, warehouse])

  const selectedStashItem = useMemo(() => {
    if (!selectedKey?.startsWith('stash:')) return null
    const id = selectedKey.slice('stash:'.length)
    return activeStashItems.find(item => item.id === id) ?? null
  }, [selectedKey, activeStashItems])

  useEffect(() => {
    if (selectedWarehouseItem) {
      setMetaDraft({
        tags: selectedWarehouseItem.tags || '',
        notes: selectedWarehouseItem.notes || '',
      })
    }
  }, [selectedWarehouseItem?.id])

  // 选中 stash 物品时,异步查 per-code 默认收藏页。
  // 防抖 200ms,避免切页 / 拖拽触发的中间选中导致抖动。
  const stashKeyForDefault = selectedStashItem?.id ?? null
  useEffect(() => {
    if (!selectedStashItem) {
      setDefaultPageName(null)
      return
    }
    const code = selectedStashItem.code
    let cancelled = false
    const timer = window.setTimeout(() => {
      resolveDefaultPage(code)
        .then(info => { if (!cancelled) setDefaultPageName(info.code_default ?? null) })
        .catch(() => { if (!cancelled) setDefaultPageName(null) })
    }, 200)
    return () => { cancelled = true; window.clearTimeout(timer) }
  }, [stashKeyForDefault, selectedStashItem?.code])

  /**
   * "改默认" 按钮 handler:把当前 activeWarehousePage 写为该 code 的 per-code 默认。
   * 后续相同 code 的物品入库(走 page_name=None 路径)会自动落入此处。
   */
  const handleSetDefault = useCallback(async () => {
    if (!selectedStashItem) return
    if (submitting) return
    const code = selectedStashItem.code
    const page = activeWarehousePage
    if (defaultPageName === page) return // 已是本页默认,disable 兜底
    setSubmitting(true)
    try {
      await setCodeDefault(code, page)
      setDefaultPageName(page)
      showToast(`已把 ${code} 的默认收藏页设为 ${page}`, 'success', { position: 'top' })
    } catch (error: unknown) {
      showToast(error instanceof Error ? error.message : '设置默认页失败', 'error', { position: 'top' })
    } finally {
      setSubmitting(false)
    }
  }, [selectedStashItem, activeWarehousePage, defaultPageName, submitting])
  const beginDragStash = useCallback((item: StashItem, event: DragEvent<HTMLDivElement>) => {
    const payload: DragPayload = { source: 'stash', item }
    dragPayloadRef.current = payload
    setDragPayload(payload)
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', item.id)
    setGhostPointer({ x: event.clientX, y: event.clientY })
    setPlacement(null)
    setSelectedKey(`stash:${item.id}`)
  }, [])

  /**
   * 堆叠/普通页共用的点击选中 handler。点击不设 dragPayload,
   * 让 performDeposit 走 B 方案(参数传 item)而不是依赖全局 drag 状态。
   * 若当前有进行中的拖动,顺手 clear 掉,避免 ghost 残留。
   */
  const handleSelectStashItem = useCallback((item: StashItem) => {
    setSelectedKey(prev => {
      const key = `stash:${item.id}`;
      // 点同一个物品 → 取消选中
      return prev === key ? null : key;
    });
    if (dragPayloadRef.current) {
      dragPayloadRef.current = null
      setDragPayload(null)
      setGhostPointer(null)
      setPlacement(null)
    }
  }, [])
  const updateWarehouseMeta = useCallback(async (item: WarehouseItem, pageName: string, tags: string, notes: string) => {
    await tauriInvoke('warehouse_update_meta', {
      itemId: item.id,
      pageName,
      tags,
      notes,
    })
  }, [])

  const beginDragWarehouse = useCallback((item: WarehouseItem, event: DragEvent<HTMLDivElement>) => {
    const payload: DragPayload = { source: 'warehouse', item }
    dragPayloadRef.current = payload
    setDragPayload(payload)
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', item.id)
    setGhostPointer({ x: event.clientX, y: event.clientY })
    setPlacement(null)
    setSelectedKey(`warehouse:${item.id}`)
  }, [])

  const clearDrag = useCallback(() => {
    dragPayloadRef.current = null
    setDragPayload(null)
    setDropZoneActive(false)
    setActiveWarehouseDropGroup(null)
    setPlacement(null)
    setGhostPointer(null)
    setPageSwitchHoverIndex(null)
    if (pageSwitchTimerRef.current) {
      window.clearTimeout(pageSwitchTimerRef.current)
      pageSwitchTimerRef.current = null
    }
    pageSwitchTargetRef.current = null
  }, [])
  const schedulePageSwitch = useCallback((targetPageIndex: number) => {
    if (dragPayload?.source !== 'warehouse' || targetPageIndex === activeStashPageIndex) return
    if (pageSwitchTargetRef.current === targetPageIndex) return
    setPageSwitchHoverIndex(targetPageIndex)
    if (pageSwitchTimerRef.current) window.clearTimeout(pageSwitchTimerRef.current)
    pageSwitchTargetRef.current = targetPageIndex
    pageSwitchTimerRef.current = window.setTimeout(() => {
      setActiveStashPageIndex(targetPageIndex)
      setPlacement(null)
      pageSwitchTimerRef.current = null
      pageSwitchTargetRef.current = null
    }, PAGE_SWITCH_HOVER_MS)
  }, [activeStashPageIndex, dragPayload?.source])

  const cancelPageSwitch = useCallback((targetPageIndex?: number) => {
    if (targetPageIndex != null && pageSwitchTargetRef.current !== targetPageIndex) return
    if (pageSwitchTimerRef.current) {
      window.clearTimeout(pageSwitchTimerRef.current)
      pageSwitchTimerRef.current = null
    }
    pageSwitchTargetRef.current = null
    setPageSwitchHoverIndex(null)
  }, [])

  const performDeposit = useCallback(async (item: StashItem, targetPageName?: string | null, quantity?: number) => {
    if (submitting) return
    if (!stashFile) {
      showToast('未找到共享仓库文件，无法存入 SQLite', 'error', { position: 'top' })
      clearDrag()
      return
    }
    // quantity: 未传或 <= 0 → 退到 item.quantity(全量);>max → clamp 到 max。
    const maxQty = Math.max(1, item.quantity ?? 1)
    const finalQty = quantity == null || quantity <= 0
      ? maxQty
      : Math.min(quantity, maxQty)
    // targetPageName: undefined/null → 让后端 fallback(per-code default → 内置默认)。
    const pageName = targetPageName ?? null
    setSubmitting(true)
    setDropZoneActive(false)
    try {
      await tauriInvoke('warehouse_deposit', {
        stashPath: stashFile,
        itemCode: item.code,
        pageIndex: item.page_index,
        positionX: item.position_x,
        positionY: item.position_y,
        quantity: finalQty,
        pageName,
      })
      const pageLabel = pageName ?? '默认收藏页'
      const qtyLabel = item.quantity != null && item.quantity > 1 && finalQty < item.quantity
        ? ` x${finalQty}/${item.quantity}`
        : ''
      showToast(`已存入 ${item.item_name}${qtyLabel} · ${pageLabel}`, 'success', { position: 'top' })
      clearDrag()
      await refreshAfterWrite()
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error ?? '')
      showToast(msg || '存入 SQLite 失败', 'error', { position: 'top' })
      clearDrag()
    } finally {
      setSubmitting(false)
    }
  }, [clearDrag, refreshAfterWrite, stashFile, submitting])

  const moveWarehouseItemToPage = useCallback(async (item: WarehouseItem, targetPageName: string) => {
    const normalizedTarget = normalizeWarehousePageName(targetPageName)
    if (normalizeWarehousePageName(item.page_name) === normalizedTarget) return
    setSubmitting(true)
    try {
      await updateWarehouseMeta(item, normalizedTarget, item.tags || '', item.notes || '')
      showToast(`已移动到收藏页：${normalizedTarget}`, 'success', { position: 'top' })
      setActiveWarehousePage(normalizedTarget)
      await refreshAfterWrite()
    } catch (error: unknown) {
      showToast(error instanceof Error ? error.message : '移动收藏页失败', 'error', { position: 'top' })
    } finally {
      setSubmitting(false)
      clearDrag()
    }
  }, [clearDrag, refreshAfterWrite, updateWarehouseMeta])

  const saveSelectedWarehouseMeta = useCallback(async () => {
    if (!selectedWarehouseItem) return
    setSubmitting(true)
    try {
      await updateWarehouseMeta(
        selectedWarehouseItem,
        normalizeWarehousePageName(selectedWarehouseItem.page_name),
        metaDraft.tags.trim(),
        metaDraft.notes.trim(),
      )
      showToast('收藏备注已更新', 'success', { position: 'top' })
      await refreshAfterWrite()
    } catch (error: unknown) {
      showToast(error instanceof Error ? error.message : '保存收藏备注失败', 'error', { position: 'top' })
    } finally {
      setSubmitting(false)
    }
  }, [metaDraft.notes, metaDraft.tags, refreshAfterWrite, selectedWarehouseItem, updateWarehouseMeta])

  const removeSelectedWarehouseItem = useCallback(async (item: WarehouseItem) => {
    setSubmitting(true)
    try {
      await tauriInvoke('warehouse_remove', { itemId: item.id })
      showToast(`已删除：${item.item_name || item.item_code}`, 'success', { position: 'top' })
      setSelectedKey(null)
      await refreshAfterWrite()
    } catch (error: unknown) {
      showToast(error instanceof Error ? error.message : '删除仓库物品失败', 'error', { position: 'top' })
    } finally {
      setSubmitting(false)
    }
  }, [refreshAfterWrite])

  /**
   * 把 SQLite 物品落入取回暂存槽(仅堆叠页触发)。
   * 直接调 onDragOver / onDrop 进入,本地状态,尚未写盘。
   * 后端 `warehouse_withdraw` 在 stackable 分支下保留原 raw_bits 的 x/y,
   * 因此 position 0,0 仅用于 collision check 占位(1×1 物品不会冲突)。
   *
   * 返回 `{added: boolean}` 让 drop handler 区分"新加入"与"已存在",
   * 避免重复拖同一物品时弹出与状态不符的 toast。
   */
  const stageItemForWithdraw = useCallback((item: WarehouseItem, requestedQuantity: number): { added: boolean } => {
    let added = false
    setStagingItems(prev => {
      if (prev.some(s => s.item.id === item.id)) return prev
      added = true
      return [...prev, { item, requestedQuantity }]
    })
    return { added }
  }, [])

  const clearStagedItem = useCallback((itemId: string) => {
    setStagingItems(prev => prev.filter(s => s.item.id !== itemId))
    setConfirmingStagingId(current => current === itemId ? null : current)
  }, [])

  const clearAllStagedItems = useCallback(() => {
    setStagingItems([])
    setConfirmingStagingId(null)
  }, [])

  /** 触发 drop flash 视觉反馈(200ms),让用户感到"物品成功落入"。 */
  const flashDropSuccess = useCallback(() => {
    if (dropFlashTimerRef.current !== null) {
      window.clearTimeout(dropFlashTimerRef.current)
    }
    setDropFlash(true)
    dropFlashTimerRef.current = window.setTimeout(() => {
      setDropFlash(false)
      dropFlashTimerRef.current = null
    }, 220)
  }, [])

  // 卸载时清理 timer
  useEffect(() => () => {
    if (dropFlashTimerRef.current !== null) window.clearTimeout(dropFlashTimerRef.current)
  }, [])

  const confirmStagedItem = useCallback(async (staged: { item: WarehouseItem; requestedQuantity: number }) => {
    if (!activeStashPage?.is_stackable) {
      showToast('当前页不是堆叠页,请切到堆叠页后再确认取回', 'error', { position: 'top' })
      return
    }
    if (!stashFile) {
      showToast('未找到共享仓库文件，无法写入', 'error', { position: 'top' })
      return
    }
    if (submitting || confirmingStagingId || confirmingRef.current) return
    confirmingRef.current = true
    setConfirmingStagingId(staged.item.id)
    setSubmitting(true)
    try {
      await tauriInvoke('warehouse_withdraw', {
        itemId: staged.item.id,
        stashPath: stashFile,
        pageIndex: activeStashPage.index,
        positionX: 0,
        positionY: 0,
        quantity: staged.requestedQuantity,
      })
      const maxQty = Math.max(1, staged.item.quantity ?? 1)
      const qtyLabel = staged.requestedQuantity < maxQty
        ? ` ×${staged.requestedQuantity} (留 ${maxQty - staged.requestedQuantity})`
        : ''
      showToast(`已取回：${staged.item.item_name || staged.item.item_code}${qtyLabel} · 堆叠页 ${activeStashPage.label}`, 'success', { position: 'top' })
      clearStagedItem(staged.item.id)
      await refreshAfterWrite()
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : '取回失败'
      // 物品在 SQLite 收藏页被外部删除时,后端返回 'Warehouse item not found',
      // 此时应自动从 staging 移除,避免用户陷入"重试死循环"。
      if (msg.toLowerCase().includes('not found') || msg.includes('Warehouse item not found')) {
        clearStagedItem(staged.item.id)
        showToast(`${staged.item.item_name || staged.item.item_code} 已不存在,已自动从暂存槽移除`, 'warning', { position: 'top' })
      } else {
        showToast(msg, 'error', { position: 'top' })
      }
    } finally {
      confirmingRef.current = false
      setSubmitting(false)
      setConfirmingStagingId(null)
    }
  }, [activeStashPage, clearStagedItem, refreshAfterWrite, stashFile, submitting, confirmingStagingId])

  /**
   * 批量确认:顺序触发所有 stagingItems 的 confirmStagedItem。
   * confirmStagedItem 内部已用 confirmingRef 串行化,这里只需 for-await。
   * 单个失败不会中断后续(confirmStagedItem 内部 try/catch 已吞错)。
   * 注意:使用 snapshot 的 stagingItems,避免 setState 后 list 变化引起迭代错乱。
   */
  const confirmAllStagedItems = useCallback(async () => {
    if (stagingItems.length <= 1) return
    const snapshot = stagingItems.slice()
    showToast(`开始批量取回 ${snapshot.length} 件`, 'info', { position: 'top' })
    for (const staged of snapshot) {
      await confirmStagedItem(staged)
    }
  }, [stagingItems, confirmStagedItem])

  // ── Auto-backup quick restore ──
  const restoreFromBackup = useCallback(async (ab: AutoBackupEntry) => {
    if (restoringBak) return
    setRestoringBak(ab.filename)
    try {
      await tauriInvoke('restore_auto_backup', { backupFilename: ab.filename })
      showToast(`已恢复：${ab.original_stash}（操作前快照）`, 'success', { position: 'top' })
      await refreshAfterWrite()
      // Reload auto-backups after restore (the restore creates a safety copy too)
      loadAutoBackups(stashFile)
    } catch (error: unknown) {
      showToast(error instanceof Error ? error.message : '恢复失败', 'error', { position: 'top' })
    } finally {
      setRestoringBak(null)
    }
  }, [loadAutoBackups, refreshAfterWrite, restoringBak, stashFile])

  const createWarehousePage = useCallback(() => {
    const validation = validateWarehousePageName(newPageName, warehousePageOrder)
    if (validation) return showToast(validation, 'error', { position: 'top' })
    const normalized = newPageName.trim()
    setCustomWarehousePages(prev => [...prev, normalized])
    setActiveWarehousePage(normalized)
    setNewPageName('')
    showToast(`已创建收藏页：${normalized}`, 'success', { position: 'top' })
  }, [newPageName, warehousePageOrder])

  const moveCustomPage = useCallback((pageName: string, direction: -1 | 1) => {
    setCustomWarehousePages(prev => {
      const index = prev.indexOf(pageName)
      const nextIndex = index + direction
      if (index < 0 || nextIndex < 0 || nextIndex >= prev.length) return prev
      const next = [...prev]
      ;[next[index], next[nextIndex]] = [next[nextIndex], next[index]]
      return next
    })
  }, [])

  const openRenamePage = useCallback((pageName: string) => {
    setPageDialog({ kind: 'rename', pageName, draft: pageName })
  }, [])

  const openDeletePage = useCallback((pageName: string) => {
    setDeletePageMode('move')
    setPageDialog({ kind: 'delete', pageName })
  }, [])

  const confirmRenamePage = useCallback(async () => {
    if (!pageDialog || pageDialog.kind !== 'rename') return
    const nextName = pageDialog.draft.trim()
    const previousName = pageDialog.pageName
    const validation = validateWarehousePageName(nextName, warehousePageOrder, previousName)
    if (validation) return showToast(validation, 'error', { position: 'top' })
    const items = warehouseItemsByPage.get(previousName) || []
    setSubmitting(true)
    try {
      if (items.length > 0) {
        await tauriInvoke('warehouse_rename_page', {
          oldPageName: previousName,
          newPageName: nextName,
        })
      }
      setCustomWarehousePages(prev => {
        const withoutPrev = prev.filter(page => page !== previousName)
        return previousName === DEFAULT_PAGE_NAME ? withoutPrev : [...withoutPrev, nextName]
      })
      setActiveWarehousePage(nextName)
      setPageDialog(null)
      showToast(`已重命名收藏页：${previousName} -> ${nextName}`, 'success', { position: 'top' })
      await refreshAfterWrite()
    } catch (error: unknown) {
      showToast(error instanceof Error ? error.message : '重命名收藏页失败', 'error', { position: 'top' })
    } finally {
      setSubmitting(false)
    }
  }, [pageDialog, refreshAfterWrite, updateWarehouseMeta, warehouseItemsByPage, warehousePageOrder])

  const confirmDeletePage = useCallback(async () => {
    if (!pageDialog || pageDialog.kind !== 'delete') return
    const pageName = pageDialog.pageName
    const items = warehouseItemsByPage.get(pageName) || []
    if (pageName === DEFAULT_PAGE_NAME) {
      showToast('默认收藏不可删除', 'error', { position: 'top' })
      return
    }
    setSubmitting(true)
    try {
      if (items.length > 0) {
        await tauriInvoke('warehouse_delete_page', {
          pageName,
          deleteItems: deletePageMode === 'delete',
        })
      }
      setCustomWarehousePages(prev => prev.filter(page => page !== pageName))
      setActiveWarehousePage(DEFAULT_PAGE_NAME)
      setPageDialog(null)
      showToast(deletePageMode === 'move' ? `已删除收藏页并迁移物品：${pageName}` : `已删除收藏页及其中物品：${pageName}`, 'success', { position: 'top' })
      await refreshAfterWrite()
    } catch (error: unknown) {
      showToast(error instanceof Error ? error.message : '删除收藏页失败', 'error', { position: 'top' })
    } finally {
      setSubmitting(false)
    }
  }, [deletePageMode, pageDialog, refreshAfterWrite, warehouseItemsByPage])

  const updatePlacementFromPointer = useCallback((clientX: number, clientY: number) => {
    if (!gridRef.current || !activeStashPage || !dragPayload || dragPayload.source !== 'warehouse') return
    const rect = gridRef.current.getBoundingClientRect()
    // pitch/padX/padY are in the same units as the visual (since cell size is computed
    // from the same fit dimensions), so clientX → cell index maps correctly.
    const padX = GRID_PADDING
    const padY = GRID_PADDING
    const x = Math.floor((clientX - rect.left - padX) / scaledPitch)
    const y = Math.floor((clientY - rect.top - padY) / scaledPitch)
    const width = dragPayload.item.inv_width || 1
    const height = dragPayload.item.inv_height || 1
    const newPlacement = validatePlacement(activeStashPage, activeStashItems, width, height, x, y)
    setPlacement(newPlacement)
    // Keep ghost aligned with the grid-aligned preview: derive screen position from
    // the same rect+pitch that the preview uses, so the ghost never drifts from the
    // preview regardless of dragover throttle lag.
    if (newPlacement.pageIndex === activeStashPage.index) {
      const ghostScreenX = rect.left + padX + newPlacement.x * scaledPitch + (newPlacement.width * scaledCell + Math.max(0, newPlacement.width - 1) * GRID_GAP) / 2
      const ghostScreenY = rect.top + padY + newPlacement.y * scaledPitch + (newPlacement.height * scaledCell + Math.max(0, newPlacement.height - 1) * GRID_GAP) * 0.25
      setGhostPointer({ x: ghostScreenX, y: ghostScreenY })
    }
  }, [activeStashItems, activeStashPage, dragPayload, scaledCell, scaledPitch])

  const handleGridDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    if (!dragPayload || dragPayload.source !== 'warehouse' || submitting) return
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
    updatePlacementFromPointer(event.clientX, event.clientY)
  }, [dragPayload, submitting, updatePlacementFromPointer])

  const handleGridDrop = useCallback(async (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault()
    if (!dragPayload || dragPayload.source !== 'warehouse' || !placement || submitting) return
    if (!placement.valid) {
      showToast(placement.reason || '当前位置不可放置', 'error', { position: 'top' })
      clearDrag()
      return
    }
    if (!stashFile) {
      showToast('未找到共享仓库文件，无法写回 D2I', 'error', { position: 'top' })
      clearDrag()
      return
    }
    setSubmitting(true)
    try {
      await tauriInvoke('warehouse_withdraw', {
        itemId: dragPayload.item.id,
        stashPath: stashFile,
        pageIndex: placement.pageIndex,
        positionX: placement.x,
        positionY: placement.y,
      })
      showToast(`已放回共享仓库：${dragPayload.item.item_name}`, 'success', { position: 'top' })
      setSelectedKey(null)
      clearDrag()
      await refreshAfterWrite()
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error ?? '')
      showToast(msg || '写回共享仓库失败', 'error', { position: 'top' })
      clearDrag()
    } finally {
      setSubmitting(false)
    }
  }, [clearDrag, dragPayload, placement, refreshAfterWrite, stashFile, submitting])

  const gridSizeStyle = useMemo((): CSSProperties => {
    const cols = activeStashPage?.grid_width ?? 16
    const rows = activeStashPage?.grid_height ?? 16
    return {
      width: cols * GRID_CELL + Math.max(0, cols - 1) * GRID_GAP,
      height: rows * GRID_CELL + Math.max(0, rows - 1) * GRID_GAP,
    }
  }, [activeStashPage?.grid_height, activeStashPage?.grid_width])

  const renderPreview = placement && dragPayload?.source === 'warehouse' && placement.pageIndex === activeStashPage?.index
    ? (
      <div
        className={`storage-grid-preview ${placement.valid ? 'is-valid' : 'is-invalid'}`}
        style={{
          left: GRID_PADDING + placement.x * scaledPitch,
          top: GRID_PADDING + placement.y * scaledPitch,
          width: placement.width * scaledCell + Math.max(0, placement.width - 1) * GRID_GAP,
          height: placement.height * scaledCell + Math.max(0, placement.height - 1) * GRID_GAP,
        }}
      />
    )
    : null

  const renderPlacementMarker = placement && dragPayload?.source === 'warehouse' && placement.pageIndex === activeStashPage?.index
    ? (
      <div
        className={`storage-grid-marker ${placement.valid ? 'is-valid' : 'is-invalid'}`}
        style={{
          left: GRID_PADDING + placement.x * scaledPitch,
          top: Math.max(0, GRID_PADDING + placement.y * scaledPitch - 28),
        }}
      >
        {markerLabel(placement)}
      </div>
    )
    : null

  const ghostItem = dragPayload?.item ?? null
  const ghostQuality = dragPayload?.source === 'stash' ? dragPayload.item.quality : dragPayload?.item.quality
  const ghostLabel = dragPayload?.source === 'stash' ? dragPayload.item.item_name : dragPayload?.item.item_name
  const ghostCode = dragPayload?.source === 'stash' ? dragPayload.item.code : dragPayload?.item.item_code
  const ghostIcon = dragPayload?.source === 'stash'
    ? { code: dragPayload.item.code, icon: dragPayload.item.icon }
    : dragPayload ? { code: dragPayload.item.item_code, icon: dragPayload.item.icon } : null
  // 全部 ghost 都用 scaledCell(自动适配网格尺寸),装备页小屏缩放后 ghost 不再偏小。
  // warehouse→stash 的 ghost 由 updatePlacementFromPointer 用 grid 对齐坐标(精确贴格);
  // stash→warehouse 的 ghost 跟随原始光标(略偏离 cell 是预期的)。
  // warehouse→stash: ghost 直接跟随光标,中心对齐。
  // stash→warehouse: ghost 跟随光标原位(略偏离 cell 是预期的)。
  const ghostWidth = ghostItem
    ? ghostItem.inv_width * scaledCell + Math.max(0, ghostItem.inv_width - 1) * GRID_GAP
    : 0
  const ghostHeight = ghostItem
    ? ghostItem.inv_height * scaledCell + Math.max(0, ghostItem.inv_height - 1) * GRID_GAP
    : 0
  // stash→warehouse 用原偏移公式;warehouse→stash 直接居中,避免网格对齐后的位置偏移。
  const ghostStyle: CSSProperties = dragPayload?.source === 'warehouse' && ghostPointer
    ? {
        left: ghostPointer.x - ghostWidth / 2,
        top: ghostPointer.y - ghostHeight / 2,
        width: ghostWidth,
        height: ghostHeight,
        borderColor: colorForQuality(ghostQuality),
      }
    : {
        left: (ghostPointer?.x ?? 0) - Math.max(ghostWidth, 72) / 2 - 5,
        top: (ghostPointer?.y ?? 0) - Math.max(ghostHeight, 72) * 0.25 - 5,
        width: Math.max(ghostWidth, 72),
        height: Math.max(ghostHeight, 72),
        borderColor: colorForQuality(ghostQuality),
      }
  const ghostTargetText = dragPayload?.source === 'warehouse'
    ? placement ? markerLabel(placement) : `当前页${activeStashPage?.index != null ? activeStashPage.index + 1 : '-'} · 等待定位`
    : ''

  const selectedItem = selectedWarehouseItem || selectedStashItem
  const activeWarehouseIsCustom = customWarehousePages.includes(activeWarehousePage)
  const activeWarehouseCount = currentWarehouseItems.length

  return (
    <div className="font-d2emu-ui storage-workbench">
      <section className="d2emu-card storage-workbench__hero">
        <div className="storage-workbench__hero-main">
          <div>
            <h1 className="font-d2emu-title storage-workbench__title">仓储工作台</h1>
            <div className="d2emu-tags">
              <span className="d2emu-tag">{stash?.item_count ?? 0} 件共享物品</span>
              <span className="d2emu-tag">{warehouse.length} 件 SQLite 收藏</span>
              <span className="d2emu-tag">{stash?.pages.length ?? 0} 页共享仓库</span>
              <span className="d2emu-tag storage-workbench__path-tag">{stashFile?.split(/[\\/]/).pop() || '未找到 stash 文件'}</span>
            </div>
          </div>
          <div className="storage-workbench__hero-actions">
            <section className="d2emu-card storage-status-bar" style={{ margin: 0, background: 'transparent', border: 'none', padding: 0 }}>
              <div className="storage-status-bar__main" style={{ gap: 6 }}>
                <strong style={{ fontSize: 12, whiteSpace: 'nowrap' }}>拖拽状态</strong>
                <span style={{ fontSize: 12, color: 'rgba(255,255,255,0.5)' }}>{statusText(dragPayload, placement, activeStashPage, activeWarehousePage, submitting)}</span>
              </div>
            </section>
            <button className="d2emu-btn d2emu-btn-ghost" onClick={() => loadAll(true)} disabled={stashLoading || warehouseLoading || submitting}>
              <i className="fa-solid fa-rotate-right" /> 刷新
            </button>
          </div>
        </div>
      </section>

      <section className="storage-workbench__layout">
            <D2EmuCard
              className="storage-workbench__panel storage-workbench__panel--left"
              fill
              title={`共享仓库 · ${activeStashPage?.label ?? '加载中...'}`}
              actions={
                <span style={{ display: 'inline-flex', gap: 6, alignItems: 'center' }}>
                  {activeStashPage?.is_stackable && (
                    <span className="d2emu-tag" style={{ borderColor: 'var(--color-d2emu-gold)', color: 'var(--color-d2emu-gold)' }}>
                      堆叠页
                    </span>
                  )}
                  <span className="d2emu-tag">{activeStashPage?.grid_width ?? '-'}x{activeStashPage?.grid_height ?? '-'}</span>
                </span>
              }
            >
              <div className="storage-page-tabs" role="tablist" aria-label="共享仓库页签">
                {stash?.pages.map(page => {
                  const isActive = activeStashPageIndex === page.index
                  return (
                    <button
                      key={page.index}
                      type="button"
                      role="tab"
                      aria-selected={isActive}
                      className={`storage-page-tab ${isActive ? 'is-active' : ''} ${pageSwitchHoverIndex === page.index ? 'is-hover' : ''}`}
                      title={`${page.label} · ${page.grid_width}x${page.grid_height}${page.is_stackable ? ' · 堆叠页' : ''}`}
                      onClick={() => setActiveStashPageIndex(page.index)}
                      onDragEnter={() => schedulePageSwitch(page.index)}
                      onDragOver={(event) => {
                        if (dragPayload?.source !== 'warehouse') return
                        event.preventDefault()
                        schedulePageSwitch(page.index)
                      }}
                      onDragLeave={() => cancelPageSwitch(page.index)}
                    >
                      <span>{page.label}</span>
                      <span className="storage-page-tab__count">{page.item_count}</span>
                    </button>
                  )
                })}
              </div>

              {stashLoading ? (
                <div style={{ padding: 24, textAlign: 'center', color: 'var(--color-d2emu-muted)' }}>
                  <D2EmuLoading text="正在加载共享仓库..." />
                </div>
              ) : activeStashPage?.is_stackable ? (
                /* ── 堆叠页:分区展示 + 取回暂存槽 ── */
                <>
                  <div
                    className={`storage-stackable-area ${dragPayloadRef.current?.source === 'warehouse' && !submitting ? 'is-drop-mode' : ''}`}
                    onDragOver={(event) => {
                      const dp = dragPayloadRef.current
                      if (!dp || dp.source !== 'warehouse' || submitting) return
                      event.preventDefault()
                      event.dataTransfer.dropEffect = 'move'
                    }}
                  >
                    {activeStashItems.length === 0 ? (
                      <div className="storage-stackable-empty">
                        <i className="fa-solid fa-gem" />
                        <span>当前堆叠页为空。把物品从右侧 SQLite 拖到下方「取回暂存槽」即可入库。</span>
                      </div>
                    ) : (
                      <StackablePageView
                        items={activeStashItems}
                        beginDragStash={beginDragStash}
                        draggingItemId={dragPayload?.source === 'stash' ? dragPayload.item.id : null}
                        selectedItemId={selectedKey?.startsWith('stash:') ? selectedKey.slice('stash:'.length) : null}
                        onSelectItem={handleSelectStashItem}
                      />
                    )}
                  </div>

                  {/* ── 取回暂存槽(仅堆叠页) ── */}
                  <div className={`storage-staging-slot ${dragPayloadRef.current?.source === 'warehouse' && !submitting ? 'is-drop-mode' : ''} ${dropFlash ? 'is-drop-flash' : ''}`}
                    onDragOver={(event) => {
                      const dp = dragPayloadRef.current
                      if (!dp || dp.source !== 'warehouse' || submitting) return
                      event.preventDefault()
                      event.dataTransfer.dropEffect = 'move'
                    }}
                    onDrop={(event) => {
                      event.preventDefault()
                      const dp = dragPayloadRef.current
                      if (!dp || dp.source !== 'warehouse' || submitting) return
                      // 打开数量确认 modal,确认后才真正 stage。
                      // modal 关闭/取消时由 modal 自己调用 clearDrag()。
                      setPendingWithdraw(dp.item)
                    }}
                  >
                    <div className="storage-staging-slot__head">
                      <div className="storage-staging-slot__title">
                        <i className="fa-solid fa-inbox" />
                        <strong>取回暂存槽</strong>
                        <span className="storage-staging-slot__count">
                          {stagingItems.length > 0 ? `${stagingItems.length} 件待确认` : '空'}
                        </span>
                      </div>
                      <div className="storage-staging-slot__hint">
                        {stagingItems.length === 0
                          ? '把 SQLite 物品拖到这里预览,确认后写入当前堆叠页 amount。'
                          : '点 ✓ 确认写入当前堆叠页,点 ✕ 取消单个,或清空全部。'}
                      </div>
                      {stagingItems.length > 1 && (
                        <button type="button" className="d2emu-btn d2emu-btn-action d2emu-btn-sm"
                          disabled={submitting}
                          onClick={() => void confirmAllStagedItems()}
                          title="依次确认所有待写入物品(已用 confirmingRef 串行化)"
                        >
                          <i className="fa-solid fa-check-double" /> 全部确认 ({stagingItems.length})
                        </button>
                      )}
                      {stagingItems.length > 0 && (
                        <button type="button" className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm"
                          onClick={clearAllStagedItems}>
                          <i className="fa-solid fa-trash-can" /> 清空
                        </button>
                      )}
                    </div>

                    <div className={`storage-staging-slot__body ${stagingItems.length === 0 ? 'is-empty' : ''}`}>
                      {stagingItems.length === 0 ? (
                        <div className="storage-staging-slot__placeholder">
                          <i className="fa-solid fa-arrow-down" />
                          <span>暂存区</span>
                        </div>
                      ) : (
                        stagingItems.map(staged => {
                          const { item, requestedQuantity } = staged
                          const qualityColor = colorForQuality(item.quality)
                          const isConfirming = confirmingStagingId === item.id
                          const maxQty = Math.max(1, item.quantity ?? 1)
                          const isPartial = requestedQuantity < maxQty
                          return (
                            <div key={item.id}
                              className={`storage-staging-item ${isConfirming ? 'is-confirming' : ''}`}
                              style={{ borderColor: qualityColor }}
                            >
                              <div className="storage-staging-item__icon" style={{ borderColor: `${qualityColor}77` }}>
                                <img src={resolveItemIcon({ code: item.item_code, icon: item.icon })}
                                  alt={item.item_name || item.item_code} data-code={item.item_code}
                                  onError={handleImgError} />
                                {(item.quantity ?? 0) > 1 && (
                                  <span className="storage-grid-item__qty">
                                    {isPartial ? `×${requestedQuantity}` : `×${item.quantity}`}
                                  </span>
                                )}
                                <ItemTooltip
                                  mode="hover"
                                  position="top"
                                  tooltipLines={item.tooltip_lines}
                                  tooltipData={item.tooltip}
                                  quality={item.quality}
                                  itemCode={item.item_code}
                                  nameZh={item.item_name}
                                  english={item.name_en}
                                  quantity={item.quantity}
                                />
                              </div>
                              <div className="storage-staging-item__meta">
                                <div className="storage-staging-item__name" style={{ color: qualityColor }}>
                                  {item.item_name || item.item_code}
                                </div>
                                <div className="storage-staging-item__sub">
                                  {qualityLabel(item.quality)} · {itemTypeLabel(item.item_code)} · {item.page_name}
                                </div>
                                <div className="storage-staging-item__sub">
                                  {item.item_code} · 入仓 {formatTime(item.imported_at)}
                                  {isPartial && ` · 取回 ${requestedQuantity} / ${maxQty}`}
                                </div>
                                {item.tags && (
                                  <div className="storage-staging-item__tags">
                                    {item.tags.split(',').slice(0, 3).map(tag => (
                                      <span key={tag} className="storage-staging-item__tag">{tag.trim()}</span>
                                    ))}
                                  </div>
                                )}
                              </div>
                              <div className="storage-staging-item__actions">
                                <button type="button"
                                  className="d2emu-btn d2emu-btn-action d2emu-btn-sm"
                                  disabled={submitting}
                                  onClick={() => void confirmStagedItem(staged)}
                                  title="写入当前堆叠页 amount"
                                >
                                  <i className={`fa-solid ${isConfirming ? 'fa-spinner fa-spin' : 'fa-check'}`} />
                                  {' '}{isConfirming ? '写入中' : '确认'}
                                </button>
                                <button type="button"
                                  className="d2emu-btn d2emu-btn-danger d2emu-btn-sm"
                                  disabled={submitting}
                                  onClick={() => clearStagedItem(item.id)}
                                  title="从暂存槽移除(未写盘)"
                                >
                                  <i className="fa-solid fa-xmark" />
                                </button>
                              </div>
                            </div>
                          )
                        })
                      )}
                    </div>
                  </div>
                </>
              ) : (
                /* ── 装备页:坐标网格 + 拖拽落点校验(原逻辑保留) ── */
                <div className="storage-grid-wrap">
                  <div ref={fitRef} className="storage-grid-fit">
                    <div
                      className="storage-grid-scaler"
                      style={{ width: scaledGridW, height: scaledGridH }}
                    >
                      <div
                        ref={gridRef}
                        className={`storage-grid ${dragPayload?.source === 'warehouse' ? 'is-drop-mode' : ''}`}
                        style={{
                          position: 'absolute',
                          top: 0,
                          left: 0,
                          width: scaledGridW,
                          height: scaledGridH,
                          boxSizing: 'border-box',
                          gridTemplateColumns: `repeat(${activeStashPage?.grid_width ?? 0}, ${scaledCell}px)`,
                          gridTemplateRows: `repeat(${activeStashPage?.grid_height ?? 0}, ${scaledCell}px)`,
                          padding: GRID_PADDING,
                          gap: GRID_GAP,
                          borderRadius: 8,
                        }}
                        onDragOver={handleGridDragOver}
                        onDrop={handleGridDrop}
                        onDragLeave={() => {
                          if (dragPayload?.source === 'warehouse') setPlacement(null)
                        }}
                      >
                        {Array.from({ length: (activeStashPage?.grid_width ?? 0) * (activeStashPage?.grid_height ?? 0) }).map((_, index) => {
                          const x = index % (activeStashPage?.grid_width ?? 1)
                          const y = Math.floor(index / (activeStashPage?.grid_width ?? 1))
                          return <div key={`${x}-${y}`} className="storage-grid__cell" />
                        })}
                        {renderPreview}
                        {renderPlacementMarker}
                        {activeStashItems.map(item => {
                          const qc = colorForQuality(item.quality)
                          const bw = (item.quality === 'unique' || item.quality === 'rare') ? 2 : 1
                          return (
                            <div
                              key={item.id}
                              className={`storage-grid-item ${selectedKey === `stash:${item.id}` ? 'is-selected' : ''} ${dragPayload?.source === 'stash' && dragPayload.item.id === item.id ? 'is-dragging' : ''}`}
                              style={{
                                left: GRID_PADDING + item.position_x * scaledPitch,
                                top: GRID_PADDING + item.position_y * scaledPitch,
                                width: item.inv_width * scaledCell + Math.max(0, item.inv_width - 1) * GRID_GAP,
                                height: item.inv_height * scaledCell + Math.max(0, item.inv_height - 1) * GRID_GAP,
                                background: 'rgba(40,40,40,0.9)',
                                border: `${bw}px solid ${qc}`,
                              }}
                              draggable
                              onDragStart={(event) => beginDragStash(item, event)}
                              onDragEnd={clearDrag}
                              onClick={() => setSelectedKey(prev => prev === `stash:${item.id}` ? null : `stash:${item.id}`)}
                            >
                              <div style={{ position: 'relative', width: '100%', height: '100%' }}>
                                <img
                                  draggable={false}
                                  src={resolveItemIcon({ code: item.code, icon: item.icon })}
                                  alt={item.item_name}
                                  data-code={item.code}
                                  onError={handleImgError}
                                  style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'contain', pointerEvents: 'none' }}
                                />
                                {(item.quantity ?? 0) > 1 && (
                                  <span style={{
                                    position: 'absolute', bottom: 0, right: 0,
                                    background: '#000', color: '#fff', fontSize: 12, fontWeight: 700,
                                    padding: '1px 4px', lineHeight: 1, borderRadius: 2,
                                    border: '1px solid rgba(255,255,255,0.3)',
                                  }}>&times;{item.quantity ?? 0}</span>
                                )}
                                {item.tooltip?.sockets && (
                                  <SocketsOverlay sockets={item.tooltip.sockets} />
                                )}
                              </div>
                            <ItemTooltip
                                mode="hover"
                                position="top"
                                tooltipData={item.tooltip}
                                tooltipLines={item.tooltip_lines}
                                english={item.name_en}
                                itemCode={item.code}
                                nameZh={item.item_name}
                                quality={item.quality}
                                socketedItems={item.socketed_items?.length ? item.socketed_items : undefined}
                              />
                            </div>
                          )
                        })}
                      </div>
                    </div>
                  </div>
                  <p className="storage-workbench__hint">拖拽仓库物品悬停到页签约 {Math.round(PAGE_SWITCH_HOVER_MS / 10) / 100} 秒会自动切页。
                  </p>
                </div>
              )}
            </D2EmuCard>

            <D2EmuCard
              className="storage-workbench__panel storage-workbench__panel--center"
              fill
              title="操作上下文"
            >
              <div className="storage-context">
                <section className="storage-context__section">
                  <div className="storage-context__section-title">当前目标</div>
                  <div className="storage-context__summary-grid">
                    <div className="storage-context__summary-card">
                      <span>共享仓库页</span>
                      <strong>{activeStashPage?.label ?? '加载中...'}</strong>
                      <small>{activeStashPage ? `${activeStashPage.grid_width}x${activeStashPage.grid_height}` : '-'}</small>
                    </div>
                    <div className="storage-context__summary-card">
                      <span>当前收藏页</span>
                      <strong>{activeWarehousePage}</strong>
                      <small>{activeWarehouseCount} 件物品</small>
                    </div>
                  </div>
                </section>

                <section className="storage-context__section">
                  <div className="storage-context__section-title">拖拽状态</div>
                  <div className="storage-context__status">
                    {statusText(dragPayload, placement, activeStashPage, activeWarehousePage, submitting)}
                  </div>
                </section>

                <section className="storage-context__section">
                  <div className="storage-context__section-title">当前选中</div>
                  {!selectedItem ? (
                    <div className="storage-context__empty">
                      先选择一件共享仓库物品或收藏仓库物品，这里会显示详情、当前目标和可执行动作。
                    </div>
                  ) : (
                    <div className="storage-context__item">
                      <div className="storage-context__item-head">
                        <div className="storage-context__item-icon" style={{ borderColor: colorForQuality(selectedItem.quality) }}>
                          <img
                            src={resolveItemIcon(
                              selectedWarehouseItem
                                ? { code: selectedWarehouseItem.item_code, icon: selectedWarehouseItem.icon }
                                : { code: selectedStashItem!.code, icon: selectedStashItem!.icon },
                            )}
                            alt={selectedItem.item_name}
                            data-code={selectedWarehouseItem ? selectedWarehouseItem.item_code : selectedStashItem!.code}
                            onError={handleImgError}
                          />
                        </div>
                        <div className="storage-context__item-meta">
                          <strong style={{ color: colorForQuality(selectedItem.quality) }}>{selectedItem.item_name}</strong>
                          <span>
                            {'item_code' in selectedItem ? selectedItem.item_code : selectedItem.code}
                            {' · '}
                            {selectedItem.inv_width}x{selectedItem.inv_height}
                            {' · '}
                            ×{selectedItem.quantity ?? 1}
                          </span>
                          <small>
                            {selectedWarehouseItem
                              ? `收藏页：${normalizeWarehousePageName(selectedWarehouseItem.page_name)} · 入仓：${formatTime(selectedWarehouseItem.imported_at)}`
                              : `共享页：${selectedStashItem!.page_index + 1} · 坐标：(${selectedStashItem!.position_x}, ${selectedStashItem!.position_y})`}
                          </small>
                        </div>
                      </div>

                      {/* 顶部操作栏:无论物品详情多少,核心按钮永远在 top 位置。
                          点 "存入" / "存默认" 都先打开 QuantityConfirmModal 选数量,确认后才真正调 performDeposit。 */}
                      {selectedStashItem && (
                        <>
                          <div className="storage-context__actions storage-context__actions--top">
                            <button className="d2emu-btn d2emu-btn-action d2emu-btn-sm storage-context__action-btn"
                              disabled={submitting} onClick={() => setPendingDeposit({ item: selectedStashItem!, pageName: activeWarehousePage })}>
                              <i className="fa-solid fa-box-open" /> 存入
                            </button>
                            <button className="d2emu-btn d2emu-btn-primary d2emu-btn-sm storage-context__action-btn"
                              disabled={submitting || !defaultPageName}
                              title={defaultPageName ? `存入到默认收藏页: ${defaultPageName}` : '请先设置 per-code 默认收藏页'}
                              onClick={() => setPendingDeposit({ item: selectedStashItem!, pageName: null })}>
                              <i className="fa-solid fa-box-archive" /> 存默认
                            </button>
                          </div>
                          <div className="storage-context__actions">
                            <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm storage-context__action-btn"
                              disabled={submitting || defaultPageName === activeWarehousePage}
                              onClick={() => void handleSetDefault()}>
                              <i className="fa-solid fa-folder-tree" />
                              {defaultPageName === activeWarehousePage
                                ? '已是本页默认'
                                : defaultPageName
                                  ? `改默认 → ${activeWarehousePage}`
                                  : `设为默认 → ${activeWarehousePage}`}
                            </button>
                          </div>
                          {/* 当前 per-code 默认收藏页状态(灰字小行) */}
                          <small style={{ color: 'var(--color-d2emu-text-dim, #888)', fontSize: 12 }}>
                            默认收藏页:{defaultPageName ?? <span style={{ opacity: 0.6 }}>未设置</span>}
                          </small>
                        </>
                      )}

                      {/* ① 品质徽章 + 物品种类 */}
                      <div className="storage-context__badges">
                        <span className="storage-context__badge" style={{ color: colorForQuality(selectedItem.quality), borderColor: colorForQuality(selectedItem.quality) + '77' }}>
                          {qualityLabel(selectedItem.quality)}
                        </span>
                        <span className="storage-context__badge storage-context__badge--type">
                          {itemTypeLabel((selectedWarehouseItem ? selectedWarehouseItem.item_code : selectedStashItem!.code) || '')}
                        </span>
                        <button type="button"
                          className="storage-context__copy-btn"
                          title="复制物品 code 到剪贴板"
                          onClick={() => {
                            const code = (selectedWarehouseItem ? selectedWarehouseItem.item_code : selectedStashItem!.code) || ''
                            navigator.clipboard.writeText(code).then(
                              () => showToast(`已复制 code: ${code}`, 'success', { position: 'top' }),
                              () => showToast('复制失败', 'error', { position: 'top' }),
                            )
                          }}
                        >
                          <i className="fa-solid fa-copy" /> 复制 code
                        </button>
                      </div>

                      {/* ② 基础属性(tooltip.base_stats) */}
                      {(() => {
                        const tip = selectedWarehouseItem?.tooltip ?? selectedStashItem?.tooltip
                        const baseStats = tip?.base_stats
                        if (!baseStats || baseStats.length === 0) return null
                        return (
                          <div className="storage-context__stats">
                            <div className="storage-context__stats-label">基础属性</div>
                            {baseStats.map((line, idx) => (
                              <div key={`base-${idx}`} className="storage-context__stat-line storage-context__stat-line--base">{line}</div>
                            ))}
                          </div>
                        )
                      })()}

                      {/* ③ 词缀属性(分色:前缀蓝/后缀黄/手工橙) */}
                      {(() => {
                        const tip = selectedWarehouseItem?.tooltip ?? selectedStashItem?.tooltip
                        const affix = tip?.affix_stats
                        if (!affix || affix.length === 0) return null
                        return (
                          <div className="storage-context__stats">
                            <div className="storage-context__stats-label">词缀 ({affix.length})</div>
                            {affix.map((line, idx) => {
                              // 简单按色彩约定分类:前缀 (+/蓝色)、后缀 (黄色)、手工词缀 (橙色)
                              // 实际 D2R 用 stat 前缀 ID 分类,这里用行首关键词近似
                              const isCrafted = /of the|to [A-Z]/.test(line) && line.length > 12
                              const color = isCrafted ? '#c06820' : line.startsWith('+') ? '#5d6cff' : '#c4a847'
                              return (
                                <div key={`affix-${idx}`} className="storage-context__stat-line" style={{ color }}>{line}</div>
                              )
                            })}
                          </div>
                        )
                      })()}

                      {/* ④ 符文之语加成 */}
                      {(() => {
                        const tip = selectedWarehouseItem?.tooltip ?? selectedStashItem?.tooltip
                        const rw = tip?.runeword_stats
                        if (!rw || rw.length === 0) return null
                        return (
                          <div className="storage-context__stats">
                            <div className="storage-context__stats-label" style={{ color: '#ffaa33' }}>符文之语加成</div>
                            {rw.map((line, idx) => (
                              <div key={`rw-${idx}`} className="storage-context__stat-line" style={{ color: '#ffaa33' }}>{line}</div>
                            ))}
                          </div>
                        )
                      })()}

                      {/* ⑤ 套装加成 */}
                      {(() => {
                        const tip = selectedWarehouseItem?.tooltip ?? selectedStashItem?.tooltip
                        const sb = tip?.set_bonus_stats
                        if (!sb || sb.length === 0) return null
                        return (
                          <div className="storage-context__stats">
                            <div className="storage-context__stats-label" style={{ color: '#45b84a' }}>套装加成</div>
                            {sb.map((line, idx) => (
                              <div key={`sb-${idx}`} className="storage-context__stat-line" style={{ color: '#45b84a' }}>{line}</div>
                            ))}
                          </div>
                        )
                      })()}

                      {/* ⑥ 孔位 + 镶嵌物品 */}
                      {(() => {
                        const socketsInfo = selectedWarehouseItem?.tooltip?.sockets ?? selectedStashItem?.tooltip?.sockets
                        // 归一化两种镶嵌物数据源 (warehouse: SocketedItemInfo / stash: StashSocketedItem)
                        const socketedRaw = selectedStashItem?.socketed_items ?? selectedWarehouseItem?.tooltip?.sockets?.items ?? []
                        const socketed: Array<{ code: string; name: string; quality: string; qty: number }> = socketedRaw.map((s: any) => ({
                          code: s.code ?? '',
                          name: s.name_zh ?? s.item_name ?? s.code ?? '',
                          quality: String(s.quality ?? 0),
                          qty: s.quantity ?? s.amount ?? 1,
                        }))
                        const count = socketsInfo?.count ?? socketed.length
                        if (!count) return null
                        return (
                          <div className="storage-context__sockets">
                            <div className="storage-context__stats-label">孔位 ({count})</div>
                            <div className="storage-context__socket-row">
                              {Array.from({ length: count }).map((_, idx) => {
                                const fitted = socketed[idx]
                                return (
                                  <div key={`sock-${idx}`} className={`storage-context__socket ${fitted ? 'is-filled' : 'is-empty'}`}>
                                    {fitted ? (
                                      <>
                                        <img src={resolveItemIcon({ code: fitted.code })}
                                          alt={fitted.name || fitted.code}
                                          data-code={fitted.code}
                                          onError={handleImgError} />
                                        {fitted.qty > 1 && (
                                          <span className="storage-context__socket-qty">×{fitted.qty}</span>
                                        )}
                                        <ItemTooltip
                                          mode="hover"
                                          position="top"
                                          itemCode={fitted.code}
                                          nameZh={fitted.name}
                                          quality={fitted.quality}
                                          quantity={fitted.qty}
                                        />
                                      </>
                                    ) : (
                                      <i className="fa-solid fa-gem" />
                                    )}
                                  </div>
                                )
                              })}
                            </div>
                          </div>
                        )
                      })()}

                      {selectedWarehouseItem && (
                        <>
                          <div className="storage-context__actions">
                            <button
                              className="d2emu-btn d2emu-btn-action"
                              disabled={submitting || normalizeWarehousePageName(selectedWarehouseItem.page_name) === activeWarehousePage}
                              onClick={() => void moveWarehouseItemToPage(selectedWarehouseItem, activeWarehousePage)}
                            >
                              <i className="fa-solid fa-folder-open" /> 移到当前收藏页
                            </button>
                            <button
                              className="d2emu-btn d2emu-btn-danger"
                              disabled={submitting}
                              onClick={() => void removeSelectedWarehouseItem(selectedWarehouseItem)}
                            >
                              <i className="fa-solid fa-trash" /> 删除物品
                            </button>
                          </div>
                          <div className="storage-context__form">
                            <div className="d2emu-field">
                              <label>标签</label>
                              <input value={metaDraft.tags} onChange={(event) => setMetaDraft(prev => ({ ...prev, tags: event.target.value }))} />
                            </div>
                            <div className="d2emu-field">
                              <label>备注</label>
                              <textarea
                                className="storage-context__textarea"
                                value={metaDraft.notes}
                                onChange={(event) => setMetaDraft(prev => ({ ...prev, notes: event.target.value }))}
                                rows={4}
                              />
                            </div>
                            <button className="d2emu-btn d2emu-btn-ghost" disabled={submitting} onClick={() => void saveSelectedWarehouseMeta()}>
                              <i className="fa-solid fa-floppy-disk" /> 保存标签与备注
                            </button>
                          </div>
                        </>
                      )}
                    </div>
                  )}
                </section>
              </div>
            </D2EmuCard>

            <D2EmuCard
              className="storage-workbench__panel storage-workbench__panel--right"
              fill
              title="收藏仓库"
            >
              <div className="storage-warehouse-shell">
                <aside className="storage-warehouse-sidebar">
                  <div className="storage-warehouse-sidebar__header">
                    <strong>收藏页目录</strong>
                    <span>{warehousePageOrder.length} 个收藏页</span>
                  </div>
                  <div className="storage-warehouse-sidebar__list">
                    {warehousePageOrder.map(pageName => {
                      const count = warehouseItemsByPage.get(pageName)?.length ?? 0
                      const isActive = activeWarehousePage === pageName
                      const isCustom = customWarehousePages.includes(pageName)
                      return (
                        <button
                          key={pageName}
                          type="button"
                          className={`storage-warehouse-page ${isActive ? 'is-active' : ''} ${activeWarehouseDropGroup === pageName ? 'is-drop-target' : ''}`}
                          onClick={() => setActiveWarehousePage(pageName)}
                          onDragOver={(event) => {
                            if (!dragPayload || submitting) return
                            event.preventDefault()
                            event.dataTransfer.dropEffect = 'move'
                            setActiveWarehouseDropGroup(pageName)
                            setDropZoneActive(false)
                          }}
                          onDragLeave={() => {
                            setActiveWarehouseDropGroup(current => current === pageName ? null : current)
                          }}
                          onDrop={async (event) => {
                            event.preventDefault()
                            if (!dragPayload) return
                            if (dragPayload.source === 'stash') {
                              // 拖到 page tab:打开数量确认 modal,让用户选多少个 → 确认后才入库
                              setPendingDeposit({ item: dragPayload.item, pageName })
                            } else {
                              await moveWarehouseItemToPage(dragPayload.item, pageName)
                            }
                          }}
                        >
                          <span className="storage-warehouse-page__label">
                            <b>{pageName}</b>
                            <small>{pageName === DEFAULT_PAGE_NAME ? '系统默认页' : isCustom ? '自定义页' : '数据页'}</small>
                          </span>
                          <span className="storage-warehouse-page__count">{count}</span>
                        </button>
                      )
                    })}
                  </div>
                  <div className="storage-warehouse-sidebar__create">
                    <div className="d2emu-field">
                      <label>新建收藏页</label>
                      <input
                        value={newPageName}
                        onChange={(event) => setNewPageName(event.target.value)}
                        onKeyDown={(event) => {
                          if (event.key === 'Enter') {
                            event.preventDefault()
                            createWarehousePage()
                          }
                        }}
                        placeholder="例如：符文页 / 武器页 / 圣骑士"
                      />
                    </div>
                    <button type="button" className="d2emu-btn d2emu-btn-ghost" onClick={createWarehousePage}>
                      <i className="fa-solid fa-folder-plus" /> 新建
                    </button>
                  </div>
                </aside>

                <div className="storage-warehouse-main">
                  <div className="storage-warehouse-main__toolbar">
                    <div>
                      <div className="storage-warehouse-main__title">{activeWarehousePage}</div>
                      <div className="storage-warehouse-main__meta">
                        共 {activeWarehouseCount} 件
                        {query.trim() ? ` · 搜索命中 ${filteredWarehouseItems.length} 件` : ''}
                      </div>
                    </div>
                    <div className="storage-warehouse-main__actions">
                      <div className="d2emu-field storage-warehouse-main__search">
                        <label>搜索当前收藏页</label>
                        <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="名称 / code / 标签 / 备注" />
                      </div>
                      <div className="storage-search-scope">
                        <button
                          type="button"
                          className={`storage-search-scope__chip ${searchScope === 'all' ? 'is-active' : ''}`}
                          onClick={() => setSearchScope('all')}
                        >
                          全仓库
                        </button>
                        <button
                          type="button"
                          className={`storage-search-scope__chip ${searchScope === 'current' ? 'is-active' : ''}`}
                          onClick={() => setSearchScope('current')}
                        >
                          当前页
                        </button>
                      </div>
                      {activeWarehousePage !== DEFAULT_PAGE_NAME && (
                        <>
                          <button className="d2emu-btn d2emu-btn-ghost" disabled={submitting} onClick={() => openRenamePage(activeWarehousePage)}>
                            <i className="fa-solid fa-pen" /> 重命名
                          </button>
                          <button className="d2emu-btn d2emu-btn-danger" disabled={submitting} onClick={() => openDeletePage(activeWarehousePage)}>
                            <i className="fa-solid fa-trash" /> 删除
                          </button>
                        </>
                      )}
                      {activeWarehouseIsCustom && (
                        <>
                          <button className="d2emu-btn d2emu-btn-ghost" disabled={submitting} onClick={() => moveCustomPage(activeWarehousePage, -1)}>
                            <i className="fa-solid fa-arrow-up" />
                          </button>
                          <button className="d2emu-btn d2emu-btn-ghost" disabled={submitting} onClick={() => moveCustomPage(activeWarehousePage, 1)}>
                            <i className="fa-solid fa-arrow-down" />
                          </button>
                        </>
                      )}
                    </div>
                  </div>

                  <div
                    className={`d2emu-drop storage-warehouse-drop storage-warehouse-drop--compact ${dropZoneActive ? 'is-drag' : ''}`}
                    onDragOver={(event) => {
                      const dp = dragPayloadRef.current
                      if (!dp || dp.source !== 'stash' || submitting) {
                        return
                      }
                      event.preventDefault()
                      event.dataTransfer.dropEffect = 'move'
                      setActiveWarehouseDropGroup(null)
                      setDropZoneActive(true)
                    }}
                    onDragLeave={() => setDropZoneActive(false)}
                    onDrop={(event) => {
                      event.preventDefault()
                      const dp = dragPayloadRef.current
                      if (!dp || dp.source !== 'stash') return
                      // dropZone:打开数量确认 modal
                      setPendingDeposit({ item: dp.item, pageName: activeWarehousePage })
                    }}
                  >
                    <i className="fa-solid fa-box-archive d2emu-drop-icon" />
                    <strong>拖到这里存入当前收藏页</strong>
                    <span className="d2emu-drop-hint">当前目标：{activeWarehousePage}</span>
                  </div>

                  {filteredWarehouseItems.length === 0 ? (
                    <div className="storage-warehouse-main__empty">
                      {activeWarehouseCount === 0 && searchScope === 'current'
                        ? `当前收藏页为空。你可以把共享仓库物品拖到目录项或当前投放区，直接存入「${activeWarehousePage}」。`
                        : '没有匹配当前搜索条件的物品。'}
                    </div>
                  ) : (<>
                    <div className="storage-warehouse-list storage-warehouse-list--content">
                      {displayWarehouseItems.map(item => {
                        const qualityColor = colorForQuality(item.quality)
                        return (
                          <div
                            key={item.id}
                            className={`storage-warehouse-item ${selectedKey === `warehouse:${item.id}` ? 'is-selected' : ''} ${dragPayload?.source === 'warehouse' && dragPayload.item.id === item.id ? 'is-dragging' : ''}`}
                            draggable
                            onDragStart={(event) => beginDragWarehouse(item, event)}
                            onDragEnd={clearDrag}
                            onClick={() => {
                              setSelectedKey(`warehouse:${item.id}`)
                              setActiveWarehousePage(normalizeWarehousePageName(item.page_name))
                            }}
                          >
                            <div className="storage-warehouse-item__icon" style={{ borderColor: `${qualityColor}77` }}>
                              <img draggable={false}
                                src={resolveItemIcon({ code: item.item_code, icon: item.icon })}
                                alt={item.item_name || item.item_code}
                                data-code={item.item_code}
                                onError={handleImgError}
                              />
                              {(item.quantity ?? 0) > 1 && <span className="storage-grid-item__qty">×{item.quantity}</span>}
                            </div>
                            <div className="storage-warehouse-item__meta">
                              <div className="storage-warehouse-item__name" style={{ color: qualityColor }}>
                                {item.item_name || item.item_code}
                              </div>
                              <div className="storage-warehouse-item__sub">
                                {item.item_code} · {item.inv_width}x{item.inv_height} · {formatTime(item.imported_at)}
                              </div>
                              {!!item.tags && <div className="storage-warehouse-item__tags">标签：{item.tags}</div>}
                              {!!item.notes && <div className="storage-warehouse-item__tags">备注：{item.notes}</div>}
                            </div>
                            <ItemTooltip
                              mode="hover"
                              position="top"
                              tooltipData={item.tooltip}
                              tooltipLines={item.tooltip_lines}
                              quality={item.quality}
                              itemCode={item.item_code}
                              quantity={item.quantity}
                              nameZh={item.item_name}
                              socketedItems={item.tooltip?.sockets?.items?.map(s => ({
                                code: s.code,
                                item_name: s.name_zh ?? s.code,
                                quality: (s.quality ?? 0).toString(),
                                quantity: s.amount,
                              }))}
                            />
                          </div>
                        )
                      })}
                    </div>
                    {/* 底部哨兵:滚动到此处时加载更多 */}
                    <div ref={sentinelRef} style={{ height: 1 }} />
                    {renderLimit < filteredWarehouseItems.length && (
                      <div style={{ textAlign: 'center', padding: 12, color: 'var(--color-d2emu-muted)', fontSize: 14 }}>
                        加载更多…
                      </div>
                    )}
                    </>
                  )}
                </div>
              </div>
            </D2EmuCard>
          </section>


          {/* ── 最近操作 / 快速恢复 ── */}
          {autoBackups.length > 0 && (
            <section className="d2emu-card" style={{ marginTop: 8, padding: '12px 16px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, cursor: 'pointer' }}
                onClick={() => setShowBackupPanel(v => !v)}
              >
                <i className={`fa-solid fa-chevron-${showBackupPanel ? 'down' : 'right'}`} style={{ fontSize: 15, color: 'var(--color-d2emu-muted)' }} />
                <span style={{ fontSize: 16, color: 'var(--color-d2emu-muted)' }}>最近操作（可恢复）</span>
                <span style={{ fontSize: 15, color: 'var(--color-d2emu-text-dim)' }}>{autoBackups.length} 条</span>
              </div>
              {showBackupPanel && (
                <div style={{ marginTop: 10, display: 'flex', flexDirection: 'column', gap: 6 }}>
                  {autoBackups.map(ab => (
                    <div key={ab.filename}
                      style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: 16, padding: '7px 10px', background: 'rgba(0,0,0,0.15)', borderRadius: 4 }}
                    >
                      <span>
                        <span style={{ color: 'var(--color-d2emu-gold)', fontSize: 16 }}>
                          {ab.operation === 'deposit' ? '存入' : ab.operation === 'withdraw' ? '取出' : ab.operation}
                        </span>
                        <span style={{ color: 'var(--color-d2emu-text-dim)', marginLeft: 12, fontSize: 15 }}>{ab.timestamp}</span>
                      </span>
                      <button
                        className="d2emu-btn d2emu-btn-ghost d2emu-btn-xs"
                        disabled={restoringBak === ab.filename}
                        onClick={() => restoreFromBackup(ab)}
                      >
                        {restoringBak === ab.filename ? '恢复中...' : '恢复'}
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </section>
          )}

      {dragPayload && ghostPointer && ghostIcon && (
        <div
          className={`storage-drag-ghost ${dragPayload.source === 'warehouse' ? 'is-from-warehouse' : 'is-from-stash'} ${placement?.valid ? 'is-valid' : placement && !placement.valid ? 'is-invalid' : ''}`}
          style={ghostStyle}
          aria-hidden="true"
        >
          <img
            src={resolveItemIcon(ghostIcon)}
            alt={ghostLabel || ghostCode || 'drag-preview'}
            data-code={ghostCode || ''}
            onError={handleImgError}
          />
          <span className="storage-drag-ghost__meta">
            <span className="storage-drag-ghost__title">
              <b>{ghostLabel || ghostCode}</b>
              <small>{ghostItem?.inv_width}x{ghostItem?.inv_height}</small>
            </span>
            {ghostTargetText && <small>{ghostTargetText}</small>}
          </span>
        </div>
      )}

      {pageDialog?.kind === 'rename' && (
        <D2ConfirmModal
          title={`重命名收藏页：${pageDialog.pageName}`}
          confirmText="保存名称"
          loading={submitting}
          onConfirm={() => void confirmRenamePage()}
          onClose={() => setPageDialog(null)}
        >
          <div className="d2emu-field">
            <label>新名称</label>
            <input
              autoFocus
              value={pageDialog.draft}
              onChange={(event) => setPageDialog({ ...pageDialog, draft: event.target.value })}
              placeholder="请输入新的收藏页名称"
            />
          </div>
        </D2ConfirmModal>
      )}

      {/* 数量确认 modal:所有 deposit 入口(中间栏位按钮 / 拖到 page tab / dropZone)都先打开这个,
          让用户选 N → 确认后才真正调 performDeposit。 */}
      {pendingDeposit && (
        <QuantityConfirmModal
          mode="deposit"
          item={{
            item_name: pendingDeposit.item.item_name,
            code: pendingDeposit.item.code,
            icon: pendingDeposit.item.icon,
            quantity: pendingDeposit.item.quantity ?? 1,
            subtitle: `坐标 (${pendingDeposit.item.position_x}, ${pendingDeposit.item.position_y})`,
          }}
          pageName={pendingDeposit.pageName}
          pageLabel={pendingDeposit.pageName ?? '默认收藏页(后端解析)'}
          loading={submitting}
          onConfirm={async (quantity) => {
            const target = pendingDeposit
            setPendingDeposit(null)
            await performDeposit(target.item, target.pageName, quantity)
          }}
          onClose={() => setPendingDeposit(null)}
        />
      )}

      {/* 数量确认 modal:从 warehouse 拖到 staging slot 时打开,
          让用户选 N → 确认后才真正 stage 进暂存槽(再点确认才写 stash)。 */}
      {pendingWithdraw && (
        <QuantityConfirmModal
          mode="withdraw"
          item={{
            item_name: pendingWithdraw.item_name,
            code: pendingWithdraw.item_code,
            icon: pendingWithdraw.icon,
            quantity: pendingWithdraw.quantity ?? 1,
          }}
          pageLabel={`堆叠页：${activeStashPage?.label ?? '(未知)'}`}
          loading={false}
          onConfirm={async (quantity) => {
            const item = pendingWithdraw
            setPendingWithdraw(null)
            const result = stageItemForWithdraw(item, quantity)
            showToast(
              result.added
                ? `已暂存：${item.item_name || item.item_code}${quantity < (item.quantity ?? 1) ? ` ×${quantity}` : ''}`
                : `${item.item_name || item.item_code} 已在暂存槽中`,
              result.added ? 'info' : 'warning',
              { position: 'top' },
            )
            if (result.added) flashDropSuccess()
            clearDrag()
          }}
          onClose={() => {
            setPendingWithdraw(null)
            clearDrag()
          }}
        />
      )}

      {pageDialog?.kind === 'delete' && (
        <D2ConfirmModal
          title={`删除收藏页：${pageDialog.pageName}`}
          confirmText={deletePageMode === 'move' ? '删除并迁移物品' : '删除页与全部物品'}
          danger
          loading={submitting}
          onConfirm={() => void confirmDeletePage()}
          onClose={() => setPageDialog(null)}
        >
          <div className="storage-delete-dialog">
            <p>当前收藏页共有 {warehouseItemsByPage.get(pageDialog.pageName)?.length ?? 0} 件物品。请选择删除方式：</p>
            <label className="storage-delete-dialog__option">
              <input type="radio" checked={deletePageMode === 'move'} onChange={() => setDeletePageMode('move')} />
              <span>仅删除收藏页，并把物品迁移到“默认收藏”</span>
            </label>
            <label className="storage-delete-dialog__option">
              <input type="radio" checked={deletePageMode === 'delete'} onChange={() => setDeletePageMode('delete')} />
              <span>删除收藏页，并永久删除其中物品</span>
            </label>
          </div>
        </D2ConfirmModal>
      )}
    </div>
  )
}
