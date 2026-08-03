import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { SKILL_TREE_LAYOUTS, type SkillNodeDef, type SkillTreeDef } from '../data/skillTreeLayouts'
import { skillName } from '../data/skills'
import SKILL_TOOLTIPS from '../data/skillTooltips.json'
import type { SkillBonus, SkillEntry } from '../types'
import { tauriInvoke } from '../tauri'
import {
  findDirectionalNode,
  summarizeSkills,
  type SkillDirectionKey,
} from '../utils/skillPresentation'
import SkillDetailsPanel, {
  type SkillDetailsSelection,
  type SkillTooltip,
} from './SkillDetailsPanel'
import TabBar from './TabBar'

/** 各职业 3 系技能面板的中文名称 */
const TAB_NAMES_ZHCN: Record<string, [string, string, string]> = {
  Amazon: ['弓和十字弓', '被动与魔法', '标枪和长矛'],
  Sorceress: ['火焰技能', '闪电技能', '冰霜技能'],
  Necromancer: ['诅咒', '白骨和毒素', '召唤技能'],
  Paladin: ['战斗技能', '攻击光环', '防御光环'],
  Barbarian: ['战斗技能', '战斗专精', '战嗥'],
  Druid: ['召唤技能', '变形技能', '元素技能'],
  Assassin: ['陷阱', '影子训练', '武学技能'],
  Warlock: ['符印/毁灭', '咒术/邪能', '召唤/仆从'],
}

function tabTitleFor(class_en: string, index: number, treeTitle: string, language?: string): string {
  if (language === 'zhCN') {
    const names = TAB_NAMES_ZHCN[class_en]
    if (names) return names[index]
  }
  return treeTitle
}

function tooltipFor(class_en: string, skillId: number): SkillTooltip | undefined {
  const clsData = (SKILL_TOOLTIPS as unknown as Record<string, { skills: Record<string, SkillTooltip> }>)[class_en]
  return clsData?.skills[String(skillId)]
}

const S = {
  nodeSize: 56,
  ptColor: '#fff',
  ptBg: '#050403',
  ptBorder: 'rgba(229,179,76,0.9)',
  hasPointsBg: 'radial-gradient(circle at 50% 42%, rgba(255,230,150,0.16), rgba(255,230,150,0.04) 58%, transparent 76%)',
  hasPointsShadow: 'inset 0 0 16px rgba(246,216,128,0.22), 0 0 14px rgba(229,179,76,0.18)',
  selectedShadow: '0 0 0 2px rgba(5,4,3,0.85), 0 0 18px rgba(251,177,58,0.58)',
  previewShadow: '0 0 0 2px rgba(5,4,3,0.75), 0 0 16px rgba(154,0,0,0.48)',
}

function edgeClass(edges: string[]) {
  let result = ''
  if (edges.includes('left')) result += ' edge-left'
  if (edges.includes('right')) result += ' edge-right'
  if (edges.includes('top')) result += ' edge-top'
  return result
}

function lookupLocalized(
  tooltip: { name?: string } | undefined,
  keySuffix: string,
  texts: Record<string, string> | null,
): string | undefined {
  if (!texts || !tooltip?.name) return undefined
  const lookupKey = `${tooltip.name.replace(/\s+/g, '').toLowerCase()}${keySuffix}`
  return texts[lookupKey]
}

interface SkillNodeProps {
  node: SkillNodeDef
  level: number
  name: string
  ariaLabel: string
  selected: boolean
  previewed: boolean
  onPreview: (id: number | null) => void
  onSelect: (id: number) => void
  onNavigate: (id: number, key: SkillDirectionKey) => void
  onRegister: (id: number, element: HTMLButtonElement | null) => void
}

function SkillNode({
  node,
  level,
  name,
  ariaLabel,
  selected,
  previewed,
  onPreview,
  onSelect,
  onNavigate,
  onRegister,
}: SkillNodeProps) {
  const hasPoints = level > 0
  const className = [
    'skill-node',
    hasPoints ? 'has-points' : 'unallocated',
    selected ? 'is-selected' : '',
    previewed ? 'is-previewed' : '',
    edgeClass(node.edges),
  ].filter(Boolean).join('')
  const nodeShadow = selected ? S.selectedShadow : previewed ? S.previewShadow : hasPoints ? S.hasPointsShadow : 'none'

  return (
    <button
      ref={element => onRegister(node.skillId, element)}
      type="button"
      className={className}
      aria-label={ariaLabel}
      title={name}
      aria-pressed={selected}
      aria-controls="skill-details-panel"
      style={{
        position: 'absolute',
        left: `${node.x}%`,
        top: `${node.y}%`,
        width: `${S.nodeSize}px`,
        height: `${S.nodeSize}px`,
        transform: 'translate(-50%, -50%)',
        zIndex: selected || previewed ? 3 : 1,
        cursor: 'pointer',
        borderRadius: 4,
        padding: 0,
        appearance: 'none',
        background: hasPoints ? S.hasPointsBg : 'transparent',
        border: hasPoints ? '2px solid transparent' : '2px solid rgba(45,41,35,0.72)',
        boxShadow: nodeShadow,
        transition: 'box-shadow 0.12s ease, border-color 0.12s ease, filter 0.12s ease',
      }}
      onMouseEnter={() => onPreview(node.skillId)}
      onMouseLeave={event => {
        if (document.activeElement !== event.currentTarget) onPreview(null)
      }}
      onFocus={() => onPreview(node.skillId)}
      onBlur={() => onPreview(null)}
      onClick={() => onSelect(node.skillId)}
      onKeyDown={event => {
        if (!['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) return
        event.preventDefault()
        onNavigate(node.skillId, event.key as SkillDirectionKey)
      }}
    >
      {node.edges.map(direction => {
        const color = hasPoints ? 'rgba(180,155,80,0.55)' : '#2a2a2a'
        if (direction === 'top') return (
          <span key={direction} aria-hidden="true" style={{
            position: 'absolute', left: '50%', top: -14, width: 2, height: 14,
            background: color, transform: 'translateX(-50%)', zIndex: 0,
          }} />
        )
        if (direction === 'left') return (
          <span key={direction} aria-hidden="true" style={{
            position: 'absolute', right: 'calc(50% + 20px)', top: '50%', height: 2,
            width: 16, background: color, transform: 'translateY(-50%)', zIndex: 0,
          }} />
        )
        if (direction === 'right') return (
          <span key={direction} aria-hidden="true" style={{
            position: 'absolute', left: 'calc(50% + 20px)', top: '50%', height: 2,
            width: 16, background: color, transform: 'translateY(-50%)', zIndex: 0,
          }} />
        )
        return null
      })}

      <span className="skill-node-icon" aria-hidden="true" style={{
        position: 'relative', zIndex: 2,
        display: 'grid', placeItems: 'center', width: '100%', height: '100%',
        font: '700 16px/1 "Source Sans 3", sans-serif',
        color: hasPoints ? '#e0d4c0' : '#6a6258',
        textShadow: hasPoints ? '0 1px 3px rgba(0,0,0,0.6)' : 'none',
      }}>
        {node.icon}
      </span>

      <span className="skill-points" aria-hidden="true" style={{
        position: 'absolute', right: -8, bottom: -8, zIndex: 2,
        minWidth: 32, height: 24, padding: '0 6px', borderRadius: 999,
        border: `1px solid ${S.ptBorder}`,
        background: S.ptBg,
        color: S.ptColor,
        font: '700 14px/24px "Source Sans 3", sans-serif',
        textAlign: 'center', pointerEvents: 'none',
      }}>
        {level}
      </span>
    </button>
  )
}

interface SkillTreePanelProps {
  tree: SkillTreeDef
  treeIndex: number
  levels: Map<number, number>
  class_en: string
  language?: string
  localizedTexts: Record<string, string> | null
  localizedTitle: string
  active: boolean
  selectedSkillId: number | null
  previewedSkillId: number | null
  onPreview: (id: number | null) => void
  onSelect: (id: number) => void
  onNavigate: (tree: SkillTreeDef, id: number, key: SkillDirectionKey) => void
  onRegister: (id: number, element: HTMLButtonElement | null) => void
}

function SkillTreePanel({
  tree,
  treeIndex,
  levels,
  class_en,
  language,
  localizedTexts,
  localizedTitle,
  active,
  selectedSkillId,
  previewedSkillId,
  onPreview,
  onSelect,
  onNavigate,
  onRegister,
}: SkillTreePanelProps) {
  const titleId = `skill-tree-title-${treeIndex}`

  return (
    <section className={`skill-tree-panel${active ? ' is-active' : ''}`} aria-labelledby={titleId}>
      <h3 id={titleId} className="skill-tree-title">{localizedTitle}</h3>
      <div
        className="skill-tree"
        style={{ backgroundImage: tree.tabBg ? `url(${tree.tabBg})` : undefined }}
      >
        <div className="skill-tree-vignette" aria-hidden="true" />
        {tree.nodes.map(node => {
          const tooltip = tooltipFor(class_en, node.skillId)
          const localizedName = lookupLocalized(tooltip, 'name', localizedTexts)
          const name = localizedName ?? skillName(class_en, node.skillId)
          const level = levels.get(node.skillId) ?? 0
          const ariaLabel = language === 'enUS'
            ? `${name}, ${level} hard points${node.passive ? ', passive' : ''}`
            : `${name}，硬点 ${level}${node.passive ? '，被动技能' : ''}`
          return (
            <SkillNode
              key={node.skillId}
              node={node}
              level={level}
              name={name}
              ariaLabel={ariaLabel}
              selected={selectedSkillId === node.skillId}
              previewed={previewedSkillId === node.skillId}
              onPreview={onPreview}
              onSelect={onSelect}
              onNavigate={(id, key) => onNavigate(tree, id, key)}
              onRegister={onRegister}
            />
          )
        })}
      </div>
    </section>
  )
}

export function hasSkillTreeLayout(class_en: string): boolean {
  return !!SKILL_TREE_LAYOUTS[class_en]
}

interface SkillTreeDisplayProps {
  skills: SkillEntry[]
  class_en: string
  language?: string
  remainingSkillPoints?: number
  equipmentBonuses?: SkillBonus[]
}

export default function SkillTreeDisplay({
  skills,
  class_en,
  language,
  remainingSkillPoints = 0,
  equipmentBonuses = [],
}: SkillTreeDisplayProps) {
  const [previewedSkillId, setPreviewedSkillId] = useState<number | null>(null)
  const [selectedSkillId, setSelectedSkillId] = useState<number | null>(null)
  const [activeTreeIndex, setActiveTreeIndex] = useState(0)
  const [localizedTexts, setLocalizedTexts] = useState<Record<string, string> | null>(null)
  const nodeRefs = useRef(new Map<number, HTMLButtonElement>())
  const summary = useMemo(() => {
    const s = summarizeSkills(skills)
    // Warlock 职业的布局索引 0↔2 交换过，点数分组需同步交换
    if (class_en === 'Warlock') {
      const tmp = s.pointsByTree[0]
      s.pointsByTree[0] = s.pointsByTree[2]
      s.pointsByTree[2] = tmp
    }
    return s
  }, [skills, class_en])
  const layouts = SKILL_TREE_LAYOUTS[class_en]

  useEffect(() => {
    if (!language || language === 'enUS') {
      setLocalizedTexts(null)
      return
    }
    tauriInvoke('get_localized_skill_texts', { language })
      .then((map: Record<string, string>) => setLocalizedTexts(map))
      .catch(() => setLocalizedTexts(null))
  }, [language])

  const defaultSkillId = useMemo(() => {
    let best: [number, number] | null = null
    for (const [id, level] of summary.levels) {
      if (!best || level > best[1] || (level === best[1] && id < best[0])) best = [id, level]
    }
    return best?.[0] ?? null
  }, [summary])

  useEffect(() => {
    setPreviewedSkillId(null)
    setSelectedSkillId(defaultSkillId)
    setActiveTreeIndex(defaultSkillId == null ? 0 : Math.min(2, Math.floor(defaultSkillId / 10)))
  }, [class_en, defaultSkillId])

  const registerNode = useCallback((id: number, element: HTMLButtonElement | null) => {
    if (element) nodeRefs.current.set(id, element)
    else nodeRefs.current.delete(id)
  }, [])

  const selectNode = useCallback((id: number) => {
    setActiveTreeIndex(Math.min(2, Math.floor(id / 10)))
    setSelectedSkillId(previous => previous === id ? null : id)
  }, [])

  const navigateNode = useCallback((tree: SkillTreeDef, id: number, key: SkillDirectionKey) => {
    const nextId = findDirectionalNode(tree.nodes, id, key)
    if (nextId == null) return
    nodeRefs.current.get(nextId)?.focus()
  }, [])

  if (!layouts) return null

  const currentSkillId = previewedSkillId ?? selectedSkillId
  const currentNode = currentSkillId == null
    ? undefined
    : layouts.flatMap(tree => tree.nodes).find(node => node.skillId === currentSkillId)
  const currentTooltip = currentSkillId == null ? undefined : tooltipFor(class_en, currentSkillId)
  const localizedName = lookupLocalized(currentTooltip, 'name', localizedTexts)
  const localizedDescription = lookupLocalized(currentTooltip, 'description', localizedTexts)
  const selection: SkillDetailsSelection | null = currentSkillId == null || !currentNode
    ? null
    : {
        id: currentSkillId,
        name: localizedName ?? skillName(class_en, currentSkillId),
        description: localizedDescription ?? currentTooltip?.desc,
        level: summary.levels.get(currentSkillId) ?? 0,
        passive: !!currentNode.passive,
        tooltip: currentTooltip,
      }
  const treeTitles = layouts.map((tree, index) => tabTitleFor(class_en, index, tree.title, language))

  return (
    <div
      className="skill-workspace-shell"
      onKeyDown={event => {
        if (event.key !== 'Escape') return
        setPreviewedSkillId(null)
        setSelectedSkillId(null)
      }}
    >
      <div className="skill-workspace-summary" aria-label={language === 'enUS' ? 'Skill summary' : '技能摘要'}>
        <span><small>{language === 'enUS' ? 'Unspent' : '剩余'}</small><strong>{remainingSkillPoints}</strong></span>
        <span><small>{language === 'enUS' ? 'Hard points' : '已投入'}</small><strong>{summary.totalHardPoints}</strong></span>
        <span><small>{language === 'enUS' ? 'Skills' : '已学技能'}</small><strong>{summary.investedSkillCount}</strong></span>
        <span><small>{language === 'enUS' ? 'Equipment effects' : '装备词缀'}</small><strong>{equipmentBonuses.length}</strong></span>
      </div>

      <div className="skill-workspace">
        <div className="skill-tree-area">
          <div className="skill-tree-switcher-wrap">
            <TabBar
              items={treeTitles.map((title, index) => ({
                id: index,
                label: title,
                count: summary.pointsByTree[index],
              }))}
              activeId={activeTreeIndex}
              onChange={id => setActiveTreeIndex(Number(id))}
              variant="sub"
              className="skill-tree-switcher"
            />
          </div>

          <div className="skill-tree-grid">
            {layouts.map((tree, index) => (
              <SkillTreePanel
                key={tree.title}
                tree={tree}
                treeIndex={index}
                levels={summary.levels}
                class_en={class_en}
                language={language}
                localizedTexts={localizedTexts}
                localizedTitle={treeTitles[index]}
                active={activeTreeIndex === index}
                selectedSkillId={selectedSkillId}
                previewedSkillId={previewedSkillId}
                onPreview={setPreviewedSkillId}
                onSelect={selectNode}
                onNavigate={navigateNode}
                onRegister={registerNode}
              />
            ))}
          </div>
        </div>

        <SkillDetailsPanel
          selection={selection}
          pinned={selectedSkillId != null && currentSkillId === selectedSkillId}
          equipmentBonuses={equipmentBonuses}
          language={language}
        />
      </div>
    </div>
  )
}
