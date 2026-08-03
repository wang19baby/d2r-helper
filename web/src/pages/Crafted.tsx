import CraftedItemPanel from '../components/CraftedItemPanel'

export default function Crafted() {
  return (
    <div className="font-d2emu-ui" style={{
      display: 'flex', flexDirection: 'column', gap: 12,
      flex: 1, minHeight: 0, overflow: 'hidden',
    }}>
      <CraftedItemPanel />
    </div>
  )
}
