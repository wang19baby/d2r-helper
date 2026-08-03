/**
 * BaseQualityFilter — 底材品质 chip 组 (v2 P0 玩家声音 5)
 *
 * 让玩家切换关注的底材品质：白板 / 优秀 / 无形 / 任意。
 * 默认 "任意",与 backend runewordMeta.json 的可选项 `meta.base_quality` 联动：
 *  - metadata 缺失: 该符文之语不受品质过滤影响 (向后兼容)
 *  - metadata 存在且 qualityFilter !== 'any': 仅显示 qualityFilter ∈ meta.base_quality 的
 *
 * 优势: 不需要立即改 json 全量条目,可渐进式补 metadata;
 *       玩家立即看到 chip,后续 metadata 补齐后过滤自动激活。
 */

import type { Dispatch, JSX, SetStateAction } from 'react'

export type BaseQuality = 'any' | 'normal' | 'superior' | 'ethereal'

export const BASE_QUALITY_LABEL: Record<BaseQuality, string> = {
  any: '任意',
  normal: '白板',
  superior: '优秀',
  ethereal: '无形',
}

export const BASE_QUALITY_TONE: Record<BaseQuality, string> = {
  any: 'transparent',
  normal: 'rgba(184,170,136,0.30)',
  superior: 'rgba(79,131,199,0.30)',
  ethereal: 'rgba(154,124,184,0.30)',
}

export interface BaseQualityFilterProps {
  value: BaseQuality
  onChange: Dispatch<SetStateAction<BaseQuality>>
  /** 横向排列 (默认) */
  layout?: 'row' | 'column'
}

const QUALITY_OPTIONS: BaseQuality[] = ['any', 'normal', 'superior', 'ethereal']

export default function BaseQualityFilter({
  value,
  onChange,
  layout = 'row',
}: BaseQualityFilterProps): JSX.Element {
  return (
    <div>
      <label
        style={{
          fontSize: 12,
          color: 'var(--color-d2emu-muted, #aaa)',
          display: 'block',
          marginBottom: 3,
          fontWeight: 600,
          letterSpacing: '0.04em',
        }}
      >
        <i className="fa-solid fa-gem" style={{ marginRight: 4 }} />
        品质
      </label>
      <div
        style={{
          display: 'flex',
          flexDirection: layout === 'row' ? 'row' : 'column',
          gap: 3,
          flexWrap: 'wrap',
        }}
        role="radiogroup"
        aria-label="底材品质过滤"
      >
        {QUALITY_OPTIONS.map(q => {
          const on = value === q
          return (
            <button
              key={q}
              type="button"
              role="radio"
              aria-checked={on}
              onClick={() => onChange(q)}
              title={q === 'any' ? '不限品质' : `${BASE_QUALITY_LABEL[q]}底材`}
              style={{
                padding: '2px 9px',
                borderRadius: 4,
                cursor: 'pointer',
                fontSize: 12,
                background: on
                  ? q === 'any'
                    ? 'linear-gradient(135deg, #2a1f10, #3d2e18)'
                    : BASE_QUALITY_TONE[q]
                  : 'transparent',
                border: on
                  ? '1px solid var(--color-d2emu-gold, #FBB13A)'
                  : '1px solid var(--color-d2emu-line, #252525)',
                color: on ? 'var(--color-d2emu-gold-bright, #fff)' : 'var(--color-d2emu-muted, #aaa)',
                letterSpacing: '0.04em',
              }}
            >
              {BASE_QUALITY_LABEL[q]}
            </button>
          )
        })}
      </div>
    </div>
  )
}
