import { createContext, useContext, useState, useCallback, useMemo, type ReactNode } from 'react'
import zhCN from './zhCN.json'
import enUS from './enUS.json'

export type Locale = 'zhCN' | 'enUS' | 'zhTW'

const LOCALE_DATA: Record<Locale, Record<string, string>> = {
  zhCN,
  enUS,
  zhTW: zhCN, // zhTW fallback to zhCN until translated
}

interface LocaleContextValue {
  locale: Locale
  setLocale: (locale: Locale) => void
  t: (key: string, params?: Record<string, string | number>) => string
}

const LocaleContext = createContext<LocaleContextValue | null>(null)

export function LocaleProvider({ children, initialLocale = 'zhCN' }: { children: ReactNode; initialLocale?: Locale }) {
  const [locale, setLocaleState] = useState<Locale>(() => {
    try {
      const saved = localStorage.getItem('d2r-locale')
      if (saved === 'zhCN' || saved === 'enUS' || saved === 'zhTW') return saved
    } catch { /* ignore */ }
    return initialLocale
  })

  const setLocale = useCallback((l: Locale) => {
    setLocaleState(l)
    try { localStorage.setItem('d2r-locale', l) } catch { /* ignore */ }
  }, [])

  const t = useCallback((key: string, params?: Record<string, string | number>): string => {
    const data = LOCALE_DATA[locale] ?? LOCALE_DATA.zhCN
    let text = data[key]
    if (text === undefined) {
      // fallback to zhCN
      text = LOCALE_DATA.zhCN[key] ?? key
    }
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        text = text.replace(`{${k}}`, String(v))
      }
    }
    return text
  }, [locale])

  const value = useMemo(() => ({ locale, setLocale, t }), [locale, setLocale, t])

  return (
    <LocaleContext.Provider value={value}>
      {children}
    </LocaleContext.Provider>
  )
}

export function useLocale(): LocaleContextValue {
  const ctx = useContext(LocaleContext)
  if (!ctx) throw new Error('useLocale must be used within a LocaleProvider')
  return ctx
}
