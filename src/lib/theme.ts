export type ThemeMode = 'system' | 'light' | 'dark'

const THEME_STORAGE_KEY = 'altersendme-theme'
const VALID_THEMES: ThemeMode[] = ['system', 'light', 'dark']

export function getStoredTheme(): ThemeMode {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY)
    if (stored && VALID_THEMES.includes(stored as ThemeMode)) {
      return stored as ThemeMode
    }
  } catch {
    // Ignore localStorage access errors.
  }

  return 'system'
}

export function saveTheme(theme: ThemeMode): void {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme)
  } catch {
    // Ignore localStorage access errors.
  }
}

function getResolvedTheme(mode: ThemeMode): 'light' | 'dark' {
  if (mode !== 'system') {
    return mode
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function applyTheme(mode: ThemeMode): void {
  const root = document.documentElement
  const resolved = getResolvedTheme(mode)

  root.dataset.theme = resolved
  root.dataset.themeMode = mode
}

export function applyStoredTheme(): ThemeMode {
  const mode = getStoredTheme()
  applyTheme(mode)
  return mode
}
