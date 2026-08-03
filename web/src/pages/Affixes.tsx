import D2EmuCard from '../components/D2EmuCard'
import MagicAffixPanel from '../components/MagicAffixPanel'

export default function Affixes() {
  return (
    <div className="font-d2emu-ui" style={{
      display: 'flex', flexDirection: 'column', gap: 12,
      flex: 1, minHeight: 0, overflow: 'hidden',
    }}>
      <D2EmuCard
        kicker="D2 词缀数据库"
        title={<span className="font-d2emu-title" style={{ color: 'var(--color-d2emu-gold)' }}>魔法词缀查询</span>}
        lede="按物品类型与等级范围,浏览可附加到魔法物品上的词缀(前缀/后缀)。共 1386 条魔法 + 201 条稀有。"
        actions={<i className="fa-solid fa-magic" style={{ fontSize: 22, color: '#c7b377', opacity: 0.7 }} />}
      />
      <MagicAffixPanel />
    </div>
  )
}
