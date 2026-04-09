/**
 * Theme Store - Dark/Light theme management
 */

import { create } from 'zustand'

export type ThemeMode = 'dark' | 'light' | 'system'

export interface ThemeState {
  mode: ThemeMode
  isDark: boolean

  /** Set theme mode */
  setTheme: (mode: ThemeMode) => void

  /** Toggle between dark and light */
  toggleTheme: () => void

  /** Apply theme to document */
  applyTheme: () => void
}

/**
 * Get system preference
 */
function getSystemTheme(): 'dark' | 'light' {
  if (typeof window === 'undefined') return 'dark'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

/**
 * Load saved theme from localStorage
 */
function loadSavedTheme(): ThemeMode {
  if (typeof window === 'undefined') return 'system'
  return (localStorage.getItem('theme') as ThemeMode) || 'system'
}

/**
 * Save theme to localStorage
 */
function saveTheme(mode: ThemeMode): void {
  if (typeof window === 'undefined') return
  localStorage.setItem('theme', mode)
}

/**
 * Apply theme class to document root
 */
function applyThemeToDocument(mode: ThemeMode): void {
  const root = document.documentElement
  const isDark = mode === 'system' ? getSystemTheme() === 'dark' : mode === 'dark'

  root.classList.remove('dark', 'light')
  root.classList.add(isDark ? 'dark' : 'light')

  // Update meta theme-color for mobile browsers
  const metaThemeColor = document.querySelector('meta[name="theme-color"]')
  if (metaThemeColor) {
    metaThemeColor.setAttribute('content', isDark ? '#0f172a' : '#ffffff')
  }
}

export const useThemeStore = create<ThemeState>((set, get) => {
  const initialMode = loadSavedTheme()
  const isDark = initialMode === 'system' ? getSystemTheme() === 'dark' : initialMode === 'dark'

  // Apply theme on initialization
  if (typeof window !== 'undefined') {
    applyThemeToDocument(initialMode)
  }

  // Listen for system theme changes
  if (typeof window !== 'undefined') {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (get().mode === 'system') {
        applyThemeToDocument('system')
        set({ isDark: getSystemTheme() === 'dark' })
      }
    })
  }

  return {
    mode: initialMode,
    isDark,

    setTheme: (mode) => {
      saveTheme(mode)
      applyThemeToDocument(mode)
      set({
        mode,
        isDark: mode === 'system' ? getSystemTheme() === 'dark' : mode === 'dark',
      })
    },

    toggleTheme: () => {
      const current = get().isDark ? 'light' : 'dark'
      saveTheme(current)
      applyThemeToDocument(current)
      set({ mode: current, isDark: !get().isDark })
    },

    applyTheme: () => {
      applyThemeToDocument(get().mode)
    },
  }
})
