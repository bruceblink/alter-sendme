import { useEffect, useMemo, useState } from 'react'
import { applyTheme, getStoredTheme, saveTheme, type ThemeMode } from '@/lib/theme'

const THEME_ORDER: ThemeMode[] = ['system', 'dark', 'light']

export function useTheme() {
  const [themeMode, setThemeMode] = useState<ThemeMode>(() => getStoredTheme())

  useEffect(() => {
    applyTheme(themeMode)
    saveTheme(themeMode)
  }, [themeMode])

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

    const handleSystemThemeChange = () => {
      if (themeMode === 'system') {
        applyTheme('system')
      }
    }

    mediaQuery.addEventListener('change', handleSystemThemeChange)

    return () => {
      mediaQuery.removeEventListener('change', handleSystemThemeChange)
    }
  }, [themeMode])

  const resolvedTheme = useMemo<'light' | 'dark'>(() => {
    if (themeMode !== 'system') {
      return themeMode
    }

    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }, [themeMode])

  const cycleTheme = () => {
    const currentIndex = THEME_ORDER.indexOf(themeMode)
    const nextIndex = (currentIndex + 1) % THEME_ORDER.length
    setThemeMode(THEME_ORDER[nextIndex])
  }

  return {
    themeMode,
    resolvedTheme,
    setThemeMode,
    cycleTheme,
  }
}
