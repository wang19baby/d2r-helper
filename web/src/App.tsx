import { useState, useCallback, useRef, useEffect } from 'react'
import { tauriInvoke } from './tauri'
import { Routes, Route, useNavigate, useLocation } from 'react-router-dom'
import { useImportProgress } from './hooks/useImportProgress'
import { LocaleProvider, useLocale } from './locales/context'
import TabBar, { type TabBarItem } from './components/TabBar'
import ToastContainer from './components/Toast'
import Backup from './pages/Backup'
import Home from './pages/Home'
import Characters from './pages/Characters'
import StorageWorkbench from './pages/StorageWorkbench'
import Catalog from './pages/Catalog'
import Listings from './pages/Listings'
import Config from './pages/Config'
import Support from './pages/Support'
import RunewordCalc from './pages/RunewordCalc'
import Affixes from './pages/Affixes'
import Cube from './pages/Cube'
import Crafted from './pages/Crafted'
import Grail from './pages/Grail'
import Builds from './pages/Builds'
import History from './pages/History'
const NAV: { key: string; icon: string }[] = [
  { key: 'home',       icon: 'fa-house' },
  { key: 'characters', icon: 'fa-users' },
  { key: 'storage',    icon: 'fa-box-open' },
  { key: 'runeword',   icon: 'fa-wand-magic-sparkles' },
  { key: 'cube',       icon: 'fa-flask' },
  { key: 'crafted',    icon: 'fa-hammer' },
  { key: 'affixes',    icon: 'fa-magic' },
  { key: 'builds',     icon: 'fa-tools' },
  { key: 'grail',      icon: 'fa-trophy' },
  { key: 'market',     icon: 'fa-store' },
  { key: 'listings',   icon: 'fa-tag' },
  { key: 'backup',     icon: 'fa-floppy-disk' },
  { key: 'config',     icon: 'fa-gear' },
  { key: 'support',    icon: 'fa-mug-hot' },
]

function navLabel(key: string): string {
  // Map nav keys to locale keys
  return 'nav.' + key
}

const AUTO_SAVE_INTERVAL_MS = 10_000

function AppContent() {
  const location = useLocation()
  const navigate = useNavigate()

  const currentPage = location.pathname === '/' ? 'home' : location.pathname.replace(/^\//, '')

  const handleNavigate = (key: string) => {
    navigate(key === 'home' ? '/' : `/${key}`)
  }
  const { t } = useLocale()

  const { running: importRunning } = useImportProgress()
  const [autoSaveRunning, setAutoSaveRunning] = useState(false)
  const autoSaveRef = useRef(false)
  const autoSaveEnabledRef = useRef(false)
  const timerRef = useRef<number | null>(null)

  useEffect(() => {
    const tick = async () => {
      if (!autoSaveEnabledRef.current) return
      if (autoSaveRef.current) return
      autoSaveRef.current = true
      try {
        await tauriInvoke('auto_save_stash')
        setAutoSaveRunning(true)
      } catch {
      } finally {
        autoSaveRef.current = false
      }
    }
    const handleEvent = (e: CustomEvent) => {
      const action = e.detail?.action
      if (action === 'stop') { autoSaveEnabledRef.current = false; setAutoSaveRunning(false) }
      else if (action === 'start') { autoSaveEnabledRef.current = true; tick() }
    }
    window.addEventListener('auto-save', handleEvent as EventListener)
    timerRef.current = setInterval(tick, AUTO_SAVE_INTERVAL_MS)
    return () => {
      window.removeEventListener('auto-save', handleEvent as EventListener)
      if (timerRef.current) clearInterval(timerRef.current)
    }
  }, [])
  const [navOpen, setNavOpen] = useState(false)
  const navRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!navOpen) return
    const handler = (e: MouseEvent) => {
      if (navRef.current && !navRef.current.contains(e.target as Node)) setNavOpen(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [navOpen])

  const handleNavigateMobile = useCallback((key: string) => {
    handleNavigate(key)
    setNavOpen(false)
  }, [handleNavigate])

  return (
    <div className="font-d2emu-ui" style={{ height: '100vh', display: 'flex', flexDirection: 'column', overflowY: 'auto' }}>
      <header style={{
        position: 'sticky', top: 0, zIndex: 100,
        display: 'flex', alignItems: 'center', gap: 16, flexWrap: 'wrap',
        padding: '14px 12px',
        background: 'linear-gradient(180deg, #1a1a1a, #0a0a0a)',
        borderBottom: '1px solid var(--color-d2emu-line)',
        boxShadow: '0 2px 12px rgba(0,0,0,0.5)',
      }}>
        <div className="flex items-center gap-3" style={{ flexShrink: 0 }}>
          <img alt="logo"
            style={{ width: 44, height: 44, objectFit: 'contain', filter: 'drop-shadow(0 0 8px rgba(251,177,58,0.4))' }}
            src="data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'><path d='M32 4l6 22h22l-18 14 7 22-17-13-17 13 7-22-18-14h22z' fill='%23FBB13A' stroke='%23800000' stroke-width='2'/></svg>" />
          <div>
            <h1 style={{ margin: 0, font: '700 22px/1 "Cinzel", serif', letterSpacing: 4, textTransform: 'uppercase', color: 'var(--color-d2emu-gold-bright, #fff)' }}>{t('nav.title')}</h1>
            <p style={{ margin: '2px 0 0', fontSize: 14, fontWeight: 600, letterSpacing: '0.18em', textTransform: 'uppercase' }}>{t('nav.subtitle')}</p>
          </div>
        </div>
        <div ref={navRef} className="app-header-nav">
          <div className="d2emu-nav-desktop">
            <TabBar variant="main" activeId={currentPage} onChange={(id) => handleNavigate(String(id))}
              items={NAV.map<TabBarItem>(n => ({
                id: n.key,
                label: <span><i className={`fa-solid ${n.icon}`} style={{ marginRight: 6 }} />{t(navLabel(n.key))}</span>,
                title: t(navLabel(n.key)),
              }))}
            />
          </div>
          <div className="d2emu-nav-mobile-toggle">
            <button className="d2emu-nav-hamburger" onClick={() => setNavOpen(o => !o)}
              aria-label={navOpen ? `${t('common.close')} ${t('nav.config')}` : `${t('common.open')} ${t('nav.config')}`}
              aria-expanded={navOpen}>
              <span className="d2emu-hamburger-line" /><span className="d2emu-hamburger-line" /><span className="d2emu-hamburger-line" />
            </button>
          </div>
          {navOpen && (
            <div className="d2emu-nav-mobile-dropdown" role="navigation" aria-label={t('nav.config')}>
              {NAV.map(n => {
                const isActive = currentPage === n.key
                return (
                  <button key={n.key} className={`d2emu-nav-mobile-item ${isActive ? 'is-active' : ''}`} onClick={() => handleNavigateMobile(n.key)}>
                    <i className={`fa-solid ${n.icon}`} />{t(navLabel(n.key))}
                  </button>
                )
              })}
            </div>
          )}
        </div>
        {importRunning && (
          <div className="import-indicator" style={{ display: 'flex', alignItems: 'center', gap: 6, flexShrink: 0, marginLeft: 'auto' }}>
            <span className="dot" /><span>{t('common.loading')}</span>
          </div>
        )}
        {autoSaveRunning && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 4, flexShrink: 0, fontSize: 12, color: '#5a9e6f', marginLeft: importRunning ? 8 : 'auto' }}>
            <span className="mini-dot" style={{ background: '#5a9e6f' }} /><span>{t('nav.auto_save')}</span>
          </div>
        )}
      </header>
      {importRunning && (
        <div style={{ position: 'sticky', top: '100%', zIndex: 99, height: 3, background: 'linear-gradient(90deg, #800000, #c7b377, #800000)', backgroundSize: '200% 100%', animation: 'shimmer 1.5s infinite', width: '100%' }} />
      )}
      <style>{`
        @keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
        .import-indicator { display: flex; align-items: center; gap: 6px; font-size: 14px; color: #c7b377; padding: 2px 0; }
        .import-indicator .dot, .mini-dot { width: 6px; height: 6px; border-radius: 50%; background: #c7b377; animation: pulse 1.2s ease-in-out infinite; display: inline-block; }
        @keyframes pulse { 0%, 100% { opacity: 0.3; } 50% { opacity: 1; } }
        @keyframes fade-in { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: translateY(0); } }
        .page-fade-in { animation: fade-in 0.18s ease-out; }
      `}</style>
      <main style={{ padding: '20px 16px 32px', width: '100%', boxSizing: 'border-box', flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
        <div key={location.pathname} className="page-fade-in" style={{ display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0 }}>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/characters" element={<Characters />} />
            <Route path="/storage" element={<StorageWorkbench />} />
            <Route path="/market" element={<Catalog />} />
            <Route path="/listings" element={<Listings />} />
            <Route path="/history" element={<History />} />
            <Route path="/runeword" element={<RunewordCalc />} />
            <Route path="/affixes" element={<Affixes />} />
            <Route path="/cube" element={<Cube />} />
            <Route path="/crafted" element={<Crafted />} />
            <Route path="/grail" element={<Grail />} />
            <Route path="/builds" element={<Builds />} />
            <Route path="/config" element={<Config />} />
            <Route path="/backup" element={<Backup />} />
            <Route path="/support" element={<Support />} />
          </Routes>
        </div>
      </main>
      <ToastContainer />
    </div>
  )
}

export default function App() {
  return (
    <LocaleProvider>
      <AppContent />
    </LocaleProvider>
  )
}
