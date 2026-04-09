/**
 * Language Switcher Component
 *
 * Dropdown with language selection, RTL support, and full accessibility
 */

import React from 'react'
import { useTranslation } from 'react-i18next'
import { useKeyboardDropdown } from '../hooks/a11y'

interface LanguageOption {
  code: string
  flag: string
  label: string
  nativeName: string
}

const LANGUAGES: LanguageOption[] = [
  { code: 'ru', flag: '🇷🇺', label: 'Русский', nativeName: 'Русский' },
  { code: 'en', flag: '🇺🇸', label: 'English', nativeName: 'English' },
  { code: 'bg', flag: '🇧🇬', label: 'Български', nativeName: 'Български' },
  { code: 'ar', flag: '🇸🇦', label: 'العربية', nativeName: 'العربية' },
  { code: 'zh', flag: '🇨🇳', label: '中文', nativeName: '中文' },
  { code: 'vi', flag: '🇻🇳', label: 'Tiếng Việt', nativeName: 'Tiếng Việt' },
]

const RTL_LANGUAGES = ['ar', 'he', 'fa', 'ur']

const LanguageSwitcher: React.FC = () => {
  const { i18n, t } = useTranslation()
  const [showMenu, setShowMenu] = React.useState(false)
  const triggerRef = React.useRef<HTMLButtonElement>(null)
  const menuRef = React.useRef<HTMLDivElement>(null)
  const listboxId = React.useId()

  const currentLang = i18n.language.split('-')[0]
  const currentOption = LANGUAGES.find((l) => l.code === currentLang) || LANGUAGES[0]
  const isRTL = RTL_LANGUAGES.includes(currentLang)

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
  }, [showMenu, triggerRef])

  const changeLanguage = (code: string) => {
    i18n.changeLanguage(code)
    localStorage.setItem('lang', code)

    // Apply RTL if needed
    const dir = RTL_LANGUAGES.includes(code) ? 'rtl' : 'ltr'
    document.documentElement.setAttribute('dir', dir)
    document.documentElement.setAttribute('lang', code)

    setShowMenu(false)
    triggerRef.current?.focus()
  }

  const handleSelect = (index: number) => {
    const lang = LANGUAGES[index]
    if (lang) {
      changeLanguage(lang.code)
    }
  }

  const toggleMenu = () => {
    if (!showMenu) {
      setShowMenu(true)
    } else {
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
    itemCount: LANGUAGES.length,
    onSelect: handleSelect,
  })

  return (
    <div className="relative" ref={menuRef}>
      <button
        ref={triggerRef}
        onClick={toggleMenu}
        {...triggerProps}
        aria-haspopup="listbox"
        className="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors duration-200 text-sm"
        style={{
          backgroundColor: 'var(--color-bg-secondary)',
          color: 'var(--color-text-primary)',
          border: '1px solid var(--color-border)',
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = 'var(--color-bg-tertiary)'
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = 'var(--color-bg-secondary)'
        }}
        aria-label={t('language.title')}
        title={`${t('language.title')}: ${currentOption.nativeName}`}
      >
        <span className="text-lg" role="img" aria-hidden="true">
          {currentOption.flag}
        </span>
        <span className="hidden sm:inline">{currentOption.label}</span>
        {/* Screen-reader only description */}
        <span className="sr-only">
          {t('language.title')}: {currentOption.nativeName}
        </span>
      </button>

      {showMenu && (
        <div
          ref={menuRef}
          id={listboxId}
          className="absolute top-full mt-2 min-w-[200px] rounded-lg shadow-lg z-50 overflow-hidden"
          style={{
            backgroundColor: 'var(--color-bg-secondary)',
            border: '1px solid var(--color-border)',
            right: isRTL ? 'auto' : '0',
            left: isRTL ? '0' : 'auto',
          }}
          role="listbox"
          aria-label={t('language.title')}
          onKeyDown={(e: React.KeyboardEvent) => menuProps.onKeyDown(e)}
        >
          {LANGUAGES.map((lang, index) => {
            const isActive = lang.code === currentLang
            const itemProps = getItemProps(index)
            const itemId = `lang-option-${lang.code}`

            return (
              <button
                key={lang.code}
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
                aria-label={`${lang.nativeName}${isActive ? ` (${t('common.selected') || 'selected'})` : ''}`}
              >
                <span className="text-base" role="img" aria-hidden="true">
                  {lang.flag}
                </span>
                <span>{lang.nativeName}</span>
                {isActive && (
                  <span
                    className="ml-auto text-blue-500"
                    role="img"
                    aria-label={t('common.selected') || 'selected'}
                  >
                    ✓
                  </span>
                )}
              </button>
            )
          })}
          {/* Accessibility help text */}
          <div className="sr-only" aria-live="polite">
            {t('a11y.use_arrows', 'Используйте стрелки для навигации, Enter для выбора, Escape для закрытия')}
          </div>
        </div>
      )}
    </div>
  )
}

export default LanguageSwitcher
