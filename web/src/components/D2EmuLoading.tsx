interface D2EmuLoadingProps {
  text?: string
  className?: string
}

/**
 * d2emu 风格的加载面板:暗红渐变 + 旋转环 + 脉动符文字。
 * 不依赖任何图标库,纯 SVG。
 */
export default function D2EmuLoading({ text = '读取中', className = '' }: D2EmuLoadingProps) {
  return (
    <div className={`d2emu-loading ${className}`}>
      <div className="flex flex-col items-center">
        <svg className="d2emu-loading-mark" viewBox="0 0 100 100" fill="none" aria-hidden="true">
          {/* 外圈 */}
          <circle cx="50" cy="50" r="42" stroke="currentColor" strokeWidth="2" opacity="0.18" />
          {/* 旋转环 */}
          <g className="d2emu-loading-ring">
            <circle cx="50" cy="50" r="42" stroke="currentColor" strokeWidth="3" strokeLinecap="round"
              strokeDasharray="60 200" />
          </g>
          {/* 中心符文 */}
          <g className="d2emu-loading-rune" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" fill="none">
            <path d="M50 22 L62 50 L50 78 L38 50 Z" />
            <path d="M22 50 L50 38 L78 50 L50 62 Z" />
            <circle cx="50" cy="50" r="6" />
          </g>
        </svg>
        <p className="d2emu-loading-text">{text}</p>
      </div>
    </div>
  )
}
