import { Monitor, Moon, Sun } from 'lucide-react'
import { useTheme } from '@/hooks/useTheme'
import { useTranslation } from '@/i18n'
import type { ThemeMode } from '@/lib/theme'
import type { ReactNode } from 'react'

const themeIcon: Record<ThemeMode, ReactNode> = {
  system: <Monitor size={12} />,
  dark: <Moon size={12} />,
  light: <Sun size={12} />,
}

const themeLabelKey: Record<ThemeMode, string> = {
  system: 'common:theme.system',
  dark: 'common:theme.dark',
  light: 'common:theme.light',
}

export function ThemeSwitcher() {
  const { t } = useTranslation()
  const { themeMode, cycleTheme } = useTheme()

  return (
    <button
      onClick={cycleTheme}
      className="flex items-center gap-1 px-2 py-1 text-xs transition-colors hover:opacity-80 text-app-fg underline cursor-pointer"
      title={t('common:theme.switchHint', { theme: t(themeLabelKey[themeMode]) })}
      aria-label={t('common:theme.switchHint', { theme: t(themeLabelKey[themeMode]) })}
    >
      {themeIcon[themeMode]}
      {t(themeLabelKey[themeMode])}
    </button>
  )
}
