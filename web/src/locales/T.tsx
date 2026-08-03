import { useLocale } from './context'
import type { ReactNode } from 'react'

interface TProps {
  _key: string
  params?: Record<string, string | number>
  children?: never
}

/** Inline translation component. Usage: <T _key="nav.home" /> */
export function T({ _key, params }: TProps): ReactNode {
  const { t } = useLocale()
  return t(_key, params)
}
