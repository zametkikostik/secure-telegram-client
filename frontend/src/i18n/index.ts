/**
 * i18next configuration
 */

import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'

import ru from './locales/ru.json'
import en from './locales/en.json'
import bg from './locales/bg.json'
import ar from './locales/ar.json'
import zh from './locales/zh.json'
import vi from './locales/vi.json'

const resources = {
  ru: { translation: ru },
  en: { translation: en },
  bg: { translation: bg },
  ar: { translation: ar },
  zh: { translation: zh },
  vi: { translation: vi },
}

const RTL_LANGUAGES = ['ar', 'he', 'fa', 'ur']

// Detect saved language or fallback to browser
const savedLang = typeof window !== 'undefined' ? localStorage.getItem('lang') : null

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'ru',
    supportedLngs: ['ru', 'en', 'bg', 'ar', 'zh', 'vi'],
    lng: savedLang || undefined,
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
      lookupLocalStorage: 'lang',
    },
  })

// Apply RTL direction when language changes
i18n.on('languageChanged', (lng) => {
  if (typeof document !== 'undefined') {
    const dir = RTL_LANGUAGES.includes(lng) ? 'rtl' : 'ltr'
    document.documentElement.setAttribute('dir', dir)
    document.documentElement.setAttribute('lang', lng)
  }
})

// Initial RTL setup
if (typeof document !== 'undefined') {
  const current = i18n.language
  const dir = RTL_LANGUAGES.includes(current) ? 'rtl' : 'ltr'
  document.documentElement.setAttribute('dir', dir)
  document.documentElement.setAttribute('lang', current)
}

export const RTL_LANGUAGES_LIST = RTL_LANGUAGES

export default i18n
