import CubeRecipePanel from '../components/CubeRecipePanel'

export default function Cube() {
  return (
    <div className="font-d2emu-ui" style={{
      display: 'flex', flexDirection: 'column', gap: 12,
      flex: 1, minHeight: 0, overflow: 'hidden',
    }}>
      <CubeRecipePanel />
    </div>
  )
}
