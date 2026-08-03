/**
 * D2R 物品图标 URL 解析(按优先级回退)
 *  1. 后端传入的 item.icon(本地资源,优先级最高)
 *  2. 本地 .png(prefetched from d2emu.com by build-time fetch,见 docs/)
 *  3. d2emu.com CDN(在线资源,网络可达时回退)
 *  4. 内置 SVG 占位(data URI)
 *  5. 最终回退:img.onError 隐藏并显示下一个兄弟(通常是 code 文字)
 *
 * 注:历史遗留 .webp 资源已于 2026-07-29 全量转码为 .png 并删除 webp,
 *     因此回退链不再需要 webp 中间态。
 */

const D2EMU_ITEM_IMAGE_BASE = 'https://d2emu.com/d2s/static/img/item-images-by-code'
export const LOCAL_ITEM_IMAGE_BASE = '/assets/img/items'
export const DEFAULT_ITEM_IMAGE = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' fill='%23100a05'/%3E%3Cpath d='M8 8h16v16H8z' fill='none' stroke='%23555' stroke-width='1.5'/%3E%3Ctext x='16' y='20' text-anchor='middle' fill='%23888' font-size='10' font-family='monospace'%3E%3F%3C/text%3E%3C/svg%3E"

export function itemImageUrl(code: string): string {
  return `${D2EMU_ITEM_IMAGE_BASE}/${code.toLowerCase()}.png`
}

/** 解析最终图标 URL:
 *  1. code-based .png (本地 prefetched, d2emu 新格式)
 *  2. 后端传入的 item.icon (本地 .png, code_map 自带)
 *  3. 默认占位 SVG (data URI)
 */
export function resolveItemIcon(item: { icon?: string; code?: string }): string {
  if (item.code) return `${LOCAL_ITEM_IMAGE_BASE}/${item.code.toLowerCase()}.png`
  if (item.icon) return item.icon
  return DEFAULT_ITEM_IMAGE
}

/**
 * 两级 onError 链:.png(本地) → default 占位 → 隐藏 img 显示 code 文字。
 * 使用 data-code + data-fallback 标记阶段,避免无限重试。
 * 在 <img> 上挂 data-code={item.code} 才能让 handler 拿到 code。
 */
export function handleImgError(e: React.SyntheticEvent<HTMLImageElement>) {
  const img = e.currentTarget
  const stage = img.dataset.fallback || '0'
  if (stage === '0') {
    img.dataset.fallback = '1'
    img.src = DEFAULT_ITEM_IMAGE
    return
  }
  // stage >= 1: all fallbacks exhausted, keep current image visible
}