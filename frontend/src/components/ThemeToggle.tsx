/**
 * Theme Toggle Component
 *
 * Provides UI for switching between dark, light, and system themes
 * with full accessibility and keyboard navigation
 */

import React from 'react'
import { useThemeStore, ThemeMode } from '../services/themeStore'
import { useKeyboardDropdown } from '../hooks/a11y'

interface ThemeOption {
  mode: ThemeMode
  icon: string
  labelKey: string
  description: string
}

const THEMES: ThemeOption[] = [
  {
    mode: 'light',
    icon: '☀️',
    labelKey: 'theme.light',
    description: 'Светлая тема',
  },
  {
    mode: 'dark',
    icon: '🌙',
    labelKey: 'theme.dark',
    description: 'Тёмная тема',
  },
  {
    mode: 'system',
    icon: '💻',
    labelKey: 'theme.system',
    description: 'Следовать настройкам системы',
  },
]

const ThemeToggle: React.FC = () => {
  const { mode, setTheme } = useThemeStore()
  const [showMenu, setShowMenu] = React.useState(false)
  const triggerRef = React.useRef<HTMLButtonElement>(null)
  const menuRef = React.useRef<HTMLDivElement>(null)
  const listboxId = React.useId()

  const currentOption = THEMES.find((t) => t.mode === mode) || THEMES[1]

  // Close menu on outside click
  React.useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setShowMenu(false)
        triggerRef.current?.focus()
      }
    }

    if (showMenu) {
      document.addEventListener('mousedown', handleClickOutside)
      return () => document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [showMenu])

  const toggleMenu = () => {
    if (!showMenu) {
      setShowMenu(true)
    } else {
      setShowMenu(false)
      triggerRef.current?.focus()
    }
  }

  const handleSelect = (index: number) => {
    const theme = THEMES[index]
    if (theme) {
      setTheme(theme.mode)
      setShowMenu(false)
      triggerRef.current?.focus()
    }
  }

  // Accessible keyboard dropdown
  const { triggerProps, menuProps, getItemProps } = useKeyboardDropdown({
    isOpen: showMenu,
    onClose: () => {
      setShowMenu(false)
    },
    itemCount: THEMES.length,
    onSelect: handleSelect,
  })

  return (
    <div className="relative" ref={menuRef}>
      <button
        ref={triggerRef}
        onClick={toggleMenu}
        {...triggerProps}
        aria-haspopup="listbox"
        className="theme-toggle flex items-center gap-2"
        aria-label={currentOption.description}
        aria-expanded={showMenu}
        aria-controls={listboxId}
        title={`${currentOption.description}`}
      >
        <span className="text-lg" role="img" aria-hidden="true">
          {currentOption.icon}
        </span>
        <span className="text-sm hidden sm:inline">
          {currentOption.labelKey}
        </span>
        <span className="sr-only">{currentOption.description}</span>
      </button>

      {showMenu && (
        <div
          ref={menuRef}
          id={listboxId}
          className="absolute right-0 top-full mt-2 w-56 rounded-lg shadow-lg z-50 overflow-hidden"
          style={{
            backgroundColor: 'var(--color-bg-secondary)',
            border: '1px solid var(--color-border)',
          }}
          {...menuProps}
          role="listbox"
          aria-label="Выбор темы"
        >
          {THEMES.map((theme, index) => {
            const isActive = mode === theme.mode
            const itemProps = getItemProps(index)
            const itemId = `theme-option-${theme.mode}`

            return (
              <button
                key={theme.mode}
                id={itemId}
                {...itemProps}
                className="w-full flex items-center gap-3 px-4 py-3 text-sm transition-colors duration-150"
                style={{
                  backgroundColor: isActive ? 'var(--color-bg-tertiary)' : 'transparent',
                  color: 'var(--color-text-primary)',
                  fontWeight: isActive ? 600 : 400,
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = 'var(--color-bg-tertiary)'
                }}
                onMouseLeave={(e) => {
                  if (!isActive) {
                    e.currentTarget.style.backgroundColor = 'transparent'
                  }
                }}
                aria-label={`${theme.description}${isActive ? ' (выбрано)' : ''}`}
              >
                <span className="text-lg" role="img" aria-hidden="true">
                  {theme.icon}
                </span>
                <span>{theme.labelKey}</span>
                {isActive && (
                  <span className="ml-auto text-blue-500" aria-hidden="true">
                    ✓
                  </span>
                )}
                <span className="sr-only">
                  {isActive ? ' (выбрано)' : ''}
                </span>
              </button>
            )
          })}
          {/* Accessibility help */}
          <div className="sr-only" aria-live="polite">
            Используйте стрелки для навигации, Enter для выбора, Escape для закрытия
          </div>
        </div>
      )}
    </div>
  )
}

export default ThemeToggle
