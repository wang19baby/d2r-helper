import { useNavigate } from 'react-router-dom'
import D2EmuCard from '../components/D2EmuCard'
import { useLocale } from '../locales/context'

interface NavCard {
  key: string
  icon: string
  titleKey: string
  ledeKey: string
}

export default function Home() {
  const navigate = useNavigate()
  const { t } = useLocale()

  const CORE_CARDS: NavCard[] = [
    { key: 'characters', icon: 'fa-users',               titleKey: 'nav.characters', ledeKey: 'home.lede_characters' },
    { key: 'storage',    icon: 'fa-box-open',            titleKey: 'nav.storage',    ledeKey: 'home.lede_storage' },
    { key: 'runeword',   icon: 'fa-wand-magic-sparkles', titleKey: 'nav.runeword',   ledeKey: 'home.lede_runeword' },
    { key: 'affixes',    icon: 'fa-magic',               titleKey: 'nav.affixes',    ledeKey: 'home.lede_affixes' },
    { key: 'cube',       icon: 'fa-flask',               titleKey: 'nav.cube',       ledeKey: 'home.lede_cube' },
    { key: 'crafted',    icon: 'fa-hammer',              titleKey: 'nav.crafted',    ledeKey: 'home.lede_crafted' },
    { key: 'builds',     icon: 'fa-flask',               titleKey: 'nav.builds',     ledeKey: 'home.lede_builds' },
    { key: 'grail',      icon: 'fa-trophy',              titleKey: 'nav.grail',      ledeKey: 'home.lede_grail' },
  ]

  const MARKET_CARDS: NavCard[] = [
    { key: 'market',     icon: 'fa-store',               titleKey: 'nav.market',     ledeKey: 'home.lede_market' },
    { key: 'listings',   icon: 'fa-tag',                 titleKey: 'nav.listings',   ledeKey: 'home.lede_listings' },
    { key: 'history',    icon: 'fa-clock-rotate-left',   titleKey: 'nav.history',    ledeKey: 'home.lede_history' },
  ]

  return (
    <div className="font-d2emu-ui" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>

      {/* Hero */}
      <D2EmuCard
        kicker={t('home.hero_title')}
        title="D2R 助手"
        lede={t('home.hero_desc')}
        actions={
          <button className="d2emu-btn d2emu-btn-primary d2emu-btn-sm" onClick={() => navigate('/storage')}>
            <i className="fa-solid fa-boxes-stacked" /> {t('home.hero_action')}
          </button>
        }
      />

      {/* 核心功能 */}
      <section>
        <h2 style={{
          font: '600 16px/1 "Roboto", sans-serif',
          letterSpacing: '0.14em', textTransform: 'uppercase',
          color: 'var(--color-d2emu-gold, #FBB13A)',
          margin: '0 0 10px', padding: 0,
        }}>
          <i className="fa-solid fa-cube" style={{ marginRight: 8 }} />
          {t('home.section_core')}
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          {CORE_CARDS.map(c => (
            <article key={c.key}
              className="d2emu-card-quiet d2emu-card-hoverable"
              style={{ padding: 18, cursor: 'pointer' }}
              onClick={() => navigate('/' + c.key)}
            >
              <i className={`fa-solid ${c.icon}`} style={{ fontSize: 28, color: 'var(--color-d2emu-gold)', marginBottom: 8 }} />
              <h3 className="font-d2emu-title" style={{ fontSize: 18, letterSpacing: 2, padding: 0, textAlign: 'left' }}>{t(c.titleKey)}</h3>
              <p className="d2emu-lede" style={{ marginTop: 6 }}>{t(c.ledeKey)}</p>
              <div style={{ marginTop: 12 }}>
                <button className="d2emu-btn d2emu-btn-ghost d2emu-btn-sm" onClick={e => { e.stopPropagation(); navigate('/' + c.key) }}>
                  {t('common.open')} <i className="fa-solid fa-arrow-right" style={{ fontSize: 14 }} />
                </button>
              </div>
            </article>
          ))}
        </div>
      </section>

      {/* 市场功能 */}
      <section>
        <h2 style={{
          font: '600 16px/1 "Roboto", sans-serif',
          letterSpacing: '0.14em', textTransform: 'uppercase',
          color: 'var(--color-d2emu-muted, #888)',
          margin: '0 0 10px', padding: 0,
        }}>
          <i className="fa-solid fa-store" style={{ marginRight: 8 }} />
          {t('home.section_market')}
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          {MARKET_CARDS.map(c => (
            <article key={c.key}
              className="d2emu-card-quiet d2emu-card-hoverable"
              style={{ padding: 18, cursor: 'pointer', opacity: 0.85 }}
              onClick={() => navigate('/' + c.key)}
            >
              <i className={`fa-solid ${c.icon}`} style={{ fontSize: 24, color: 'var(--color-d2emu-muted)', marginBottom: 8 }} />
              <h3 className="font-d2emu-title" style={{ fontSize: 16, letterSpacing: 2, padding: 0, textAlign: 'left' }}>{t(c.titleKey)}</h3>
              <p className="d2emu-lede" style={{ marginTop: 4 }}>{t(c.ledeKey)}</p>
            </article>
          ))}
        </div>
      </section>

    </div>
  )
}