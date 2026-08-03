import type { TooltipData } from '../types'
import { resolveItemIcon, handleImgError } from '../utils/itemImages'

/** 根据孔数返回 CSS grid 模板 (孔间固定 20px gap) */
function socketGrid(count: number): { cols: number; rows: number; skip?: number } {
  switch (count) {
    case 1:  return { cols: 1, rows: 1 }
    case 2:  return { cols: 1, rows: 2 }
    case 3:  return { cols: 1, rows: 3 }
    case 4:  return { cols: 2, rows: 2 }
    case 5:  return { cols: 2, rows: 3, skip: 2 } // 中心空
    default: return { cols: 2, rows: 3 } // 6
  }
}

export interface SocketsOverlayProps {
  sockets: NonNullable<TooltipData['sockets']>
}

export default function SocketsOverlay({ sockets }: SocketsOverlayProps) {
  const { cols, rows, skip } = socketGrid(sockets.count)
  const cells = rows * cols
  return (
    <div style={{
      position: 'absolute', inset: 0, pointerEvents: 'none',
      display: 'grid',
      gridTemplateColumns: `repeat(${cols}, auto)`,
      gridTemplateRows: `repeat(${rows}, auto)`,
      gap: '5px',
      placeItems: 'center',
      alignContent: 'center',
      justifyContent: 'center',
    }}>
      {Array.from({ length: cells }, (_, i) => {
        if (skip === i) { return <div key={i} style={{ width: 36, height: 36 }} /> }
        const itemIdx = skip !== undefined && i > skip ? i - 1 : i
        const item = sockets.items[itemIdx] ?? null
        const filled = !!item
        return (
          <div key={i} style={{
            width: 36, height: 36,
            borderRadius: '50%',
            background: filled ? 'rgba(0,0,0,0.7)' : 'rgba(0,0,0,0.6)',
            border: filled ? '1px solid rgba(200,180,120,0.6)' : '1px solid rgba(150,130,100,0.4)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            overflow: 'hidden',
          }}>
            {filled && (
              <img src={resolveItemIcon({ code: item.code })}
                alt={item.code}
                style={{ width: '80%', height: '80%', objectFit: 'contain', imageRendering: 'pixelated' }}
                onError={handleImgError}
              />
            )}
          </div>
        )
      })}
    </div>
  )
}
