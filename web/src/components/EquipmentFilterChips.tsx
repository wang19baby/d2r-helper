/**
 * EquipmentFilterChips · 装备二级筛选 (spec §3.7 P0)
 *
 * 装备类目激活时,在类目行下方展示 3 组 sub-tab chip,受控父级 state。
 * 每组单选("全部" 清除该组过滤),3 组叠加(AND)。
 *
 * 数据来源(全部从 Rust read_stash JSON 透传):
 *   - quality:       StashItem.quality       ('unique' | 'set' | 'rare' | 'magic' | 'superior' | 'normal')
 *   - equipment_slot: StashItem.equipment_slot (0=None, 1=Head, ..., 12=Trinket2)
 *   - base_type:     StashItem.base_type     ('sword' | 'axe' | 'shield' | ...)
 *
 * 过滤逻辑委托给 `utils/equipmentFilters.ts::applyEquipmentFilters` (纯函数,易测试)。
 */

import type { EquipmentFilters } from '../utils/equipmentFilters'

export { EMPTY_EQUIPMENT_FILTERS } from '../utils/equipmentFilters'
export type { EquipmentFilters } from '../utils/equipmentFilters'

interface Props {
  filters: EquipmentFilters
  onChange: (next: EquipmentFilters) => void
}

// ── 选项定义 ──────────────────────────────────────────────────────────────

const QUALITY_OPTIONS: ReadonlyArray<{ value: string | null; label: string }> = [
  { value: null,       label: '全部' },
  { value: 'unique',   label: '暗金' },
  { value: 'set',      label: '绿色套装' },
  { value: 'rare',     label: '黄色稀有' },
  { value: 'magic',    label: '蓝色魔法' },
  { value: 'superior', label: '优秀' },
  { value: 'normal',   label: '白板' },
]

// D2SLib 12 装备位 (0=None, 1..=10 vanilla, 11/12 D2R Trinket1/2)
const SLOT_OPTIONS: ReadonlyArray<{ value: number | null; label: string }> = [
  { value: null, label: '全部' },
  { value: 1,    label: '头盔' },
  { value: 2,    label: '项链' },
  { value: 3,    label: '护甲' },
  { value: 4,    label: '主手' },
  { value: 5,    label: '副手' },
  { value: 6,    label: '右戒指' },
  { value: 7,    label: '左戒指' },
  { value: 8,    label: '腰带' },
  { value: 9,    label: '鞋子' },
  { value: 10,   label: '手套' },
  { value: 11,   label: 'Trinket1' },
  { value: 12,   label: 'Trinket2' },
]

// D2 weapons.txt/armor.txt type 列常见 19 底材(中文标签)
const BASE_TYPE_OPTIONS: ReadonlyArray<{ value: string | null; label: string }> = [
  { value: null,     label: '全部' },
  { value: 'sword',  label: '剑' },
  { value: 'axe',    label: '斧' },
  { value: 'mace',   label: '钉锤' },
  { value: 'hammer', label: '锤' },
  { value: 'staff',  label: '法杖' },
  { value: 'scept',  label: '权杖' },
  { value: 'wand',   label: '棒' },
  { value: 'bow',    label: '弓' },
  { value: 'xbow',   label: '弩' },
  { value: 'dagger', label: '匕首' },
  { value: 'h2h',    label: '近战' },
  { value: 'spear',  label: '矛' },
  { value: 'polearm',label: '长柄' },
  { value: 'javelin',label: '标枪' },
  { value: 'helm',   label: '头盔' },
  { value: 'circ',   label: '头环' },
  { value: 'armor',  label: '护甲' },
  { value: 'shield', label: '盾' },
]

// ── 主组件 ────────────────────────────────────────────────────────────────

export default function EquipmentFilterChips({ filters, onChange }: Props) {
  const update = (patch: Partial<EquipmentFilters>) => {
    onChange({ ...filters, ...patch })
  }

  return (
    <div
      role="region"
      aria-label="装备二级筛选"
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
        padding: '8px 10px',
        background: 'rgba(11, 11, 11, 0.4)',
        border: '1px solid var(--color-d2emu-line-soft)',
        borderRadius: 6,
        marginBottom: 12,
      }}
    >
      <ChipRow
        label="品质"
        options={QUALITY_OPTIONS}
        active={filters.quality}
        onPick={v => update({ quality: v })}
      />
      <ChipRow
        label="部位"
        options={SLOT_OPTIONS}
        active={filters.equipment_slot}
        onPick={v => update({ equipment_slot: v })}
      />
      <ChipRow
        label="底材"
        options={BASE_TYPE_OPTIONS}
        active={filters.base_type}
        onPick={v => update({ base_type: v })}
      />
    </div>
  )
}

interface ChipRowProps<V extends string | number | null> {
  label: string
  options: ReadonlyArray<{ value: V; label: string }>
  active: V
  onPick: (v: V) => void
}

function ChipRow<V extends string | number | null>({ label, options, active, onPick }: ChipRowProps<V>) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
      <span style={{
        fontSize: 12,
        color: 'var(--color-d2emu-label)',
        letterSpacing: '0.08em',
        textTransform: 'uppercase',
        minWidth: 36,
        fontWeight: 600,
      }}>
        {label}
      </span>
      {options.map(o => {
        const isActive = active === o.value
        return (
          <button
            key={String(o.value)}
            onClick={() => onPick(o.value)}
            className={`d2emu-tag ${isActive ? 'd2emu-tag-active' : ''}`}
            style={{ cursor: 'pointer', borderStyle: 'solid', fontSize: 12 }}
            aria-pressed={isActive}
            type="button"
          >
            {o.label}
          </button>
        )
      })}
    </div>
  )
}
