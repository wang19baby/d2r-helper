export type QualityKey =
  | 'unique' | 'set' | 'rare' | 'magic'
  | 'superior' | 'normal' | 'socketed' | 'runeword'

export interface QualityInfo {
  key: QualityKey
  /** 英文 / 中文 双语 label */
  label: string
  /** 简短说明(给 tooltip) */
  desc?: string
}

export const QUALITY_ORDER: QualityKey[] = [
  'runeword', 'unique', 'set', 'rare', 'magic', 'superior', 'normal',
]

export const QUALITY_COLOR: Record<string, string> = {
  unique:     '#a08030', // 暗金
  set:        '#45b84a', // 亮绿
  rare:       '#c4a847', // 亮金
  magic:      '#5d6cff', // 蓝
  superior:   '#e8e8e8', // 白
  normal:     '#e8e8e8', // 白
  socketed:   '#6a4a8a', // 暗紫
  runeword:   '#f09020', // 橙
}

export const QUALITY_LABEL: Record<QualityKey, string> = {
  unique:    '暗金',
  set:       '套装',
  rare:      '稀有',
  magic:     '魔法',
  superior:  '超强',
  normal:    '普通',
  socketed:  '镶孔',
  runeword:  '神符之语',
}

export const QUALITY_DESC: Record<QualityKey, string> = {
  unique:    '暗金装备 · 单件名',
  set:       '套装装备 · 成套加成',
  rare:      '黄金词缀 · 随机黄装',
  magic:     '蓝色词缀 · 1-2 属性',
  superior:  '加强白装 · 基础升级',
  normal:    '基础白装',
  socketed:  '有孔 · 可镶符文/宝石/珠宝',
  runeword:  '符文之语 · 多符文组合',
}

export interface QualityLegendProps {
  /** 显示哪些 quality(默认 5 个核心 unique/set/rare/magic/socketed) */
  items?: QualityKey[]
  /** 显示模式:full=色块+名+desc / compact=色块+名 */
  mode?: 'full' | 'compact'
  className?: string
}

/**
 * QualityLegend — Quality 颜色图例
 *
 * 沿用 d2emu 设计语言:
 *  - 5 色色块 + 名称 + 描述
 *  - hover 显示 tooltip
 *  - 可嵌入任意 panel 标题下方
 */
export default function QualityLegend({
  items = ['unique', 'set', 'rare', 'magic', 'normal', 'socketed'],
  mode = 'full',
  className,
}: QualityLegendProps) {
  return (
    <div
      className={`d2emu-quality-legend ${className || ''}`}
      role="list"
      style={{
        display: 'grid',
        gridTemplateColumns: mode === 'compact'
          ? `repeat(${items.length}, auto)`
          : `repeat(${items.length}, 1fr)`,
        gap: mode === 'compact' ? 8 : 10,
        font: '400 14px/1.4 Roboto, sans-serif',
        marginTop: 12,
      }}>
      {items.map(key => {
        const c = QUALITY_COLOR[key]
        return (
          <div key={key} role="listitem"
            title={`${QUALITY_LABEL[key]} · ${QUALITY_DESC[key]}`}
            style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: mode === 'compact' ? '4px 8px' : '6px 10px',
              background: 'rgba(0,0,0,0.4)',
              borderRadius: 4,
              cursor: 'help',
            }}>
            <span style={{
              width: 18, height: 18,
              borderRadius: 2,
              border: `2px solid ${c}`,
              flexShrink: 0,
              background: `${c}10`,
            }} />
            <div style={{ minWidth: 0 }}>
              <div style={{
                color: 'var(--color-d2emu-text, #e5e2de)',
                fontWeight: 600,
                whiteSpace: 'nowrap',
              }}>
                {QUALITY_LABEL[key]}
              </div>
              {mode === 'full' && (
                <div style={{
                  color: 'var(--color-d2emu-text-muted, #8c8a85)',
                  fontSize: 14,
                  marginTop: 2,
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                }}>
                  {QUALITY_DESC[key]}
                </div>
              )}
            </div>
          </div>
        )
      })}
    </div>
  )
}