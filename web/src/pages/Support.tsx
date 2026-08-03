import { useLocale } from '../locales/context'
import D2EmuCard from '../components/D2EmuCard'

export default function Support() {
  const { t } = useLocale()
  return (
    <div style={{ margin: '0 auto' }}>
      <D2EmuCard
        kicker={t('support.title')}
        title={<span className="font-d2emu-title" style={{ color: 'var(--color-d2emu-gold)' }}>{t('support.sponsor')}</span>}
        lede={t('support.desc')}
      >
        <div style={{ textAlign: 'center', marginTop: 8 }}>
          <a
            href="https://ko-fi.com/spectr4l"
            target="_blank"
            rel="noreferrer"
            className="d2emu-btn d2emu-btn-action"
            style={{ padding: '14px 28px', fontSize: 16 }}
          >
            <i className="fa-solid fa-mug-hot" style={{ marginRight: 8 }} />
            {t('support.coffee')}
          </a>
        </div>
      </D2EmuCard>
    </div>
  )
}
