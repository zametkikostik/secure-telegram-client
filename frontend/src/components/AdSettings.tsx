import { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { FiShield, FiEye, FiClock, FiAward, FiX, FiCheck } from 'react-icons/fi'
import type {
  AdPreferences,
  AdSettings,
  AdStats,
  AdCategory,
} from '../types/ads'
import {
  ALL_AD_CATEGORIES,
  CATEGORY_LABELS,
  DEFAULT_AD_PREFERENCES,
} from '../types/ads'
import clsx from 'clsx'

interface AdSettingsModalProps {
  isOpen: boolean
  onClose: () => void
}

/**
 * Ad Settings Modal
 * 
 * Allows users to:
 * - Choose preferred and blocked categories (local targeting)
 * - Enable/disable ad types (banner, reward, interstitial)
 * - Set max ads per hour
 * - View credits earned and stats
 * - View privacy guarantees
 */
export function AdSettingsModal({ isOpen, onClose }: AdSettingsModalProps) {
  const { t } = useTranslation()
  const [settings, setSettings] = useState<AdSettings | null>(null)
  const [preferences, setPreferences] = useState<AdPreferences>(DEFAULT_AD_PREFERENCES)
  const [isLoading, setIsLoading] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [saveSuccess, setSaveSuccess] = useState(false)

  /** Load settings on mount */
  const loadSettings = useCallback(async () => {
    if (!isOpen) return

    setIsLoading(true)
    setError(null)

    try {
      const loaded = await invoke<AdSettings>('get_ad_settings')
      setSettings(loaded)
      setPreferences(loaded.preferences)
    } catch (err) {
      console.error('[AdSettings] Failed to load settings:', err)
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [isOpen])

  useEffect(() => {
    loadSettings()
  }, [loadSettings])

  /** Save preferences */
  const handleSave = async () => {
    setIsSaving(true)
    setSaveSuccess(false)
    setError(null)

    try {
      await invoke('update_ad_preferences', { preferences })
      setSaveSuccess(true)

      // Reload settings
      const loaded = await invoke<AdSettings>('get_ad_settings')
      setSettings(loaded)
    } catch (err) {
      console.error('[AdSettings] Failed to save preferences:', err)
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsSaving(false)

      // Clear success message after 3 seconds
      setTimeout(() => setSaveSuccess(false), 3000)
    }
  }

  /** Toggle category in preferred list */
  const togglePreferredCategory = (category: AdCategory) => {
    setPreferences(prev => {
      const isInPreferred = prev.preferred_categories.includes(category)
      const isInBlocked = prev.blocked_categories.includes(category)

      let newPreferred = [...prev.preferred_categories]
      let newBlocked = [...prev.blocked_categories]

      if (isInPreferred) {
        // Remove from preferred
        newPreferred = newPreferred.filter(c => c !== category)
      } else {
        // Add to preferred, remove from blocked if present
        newPreferred.push(category)
        newBlocked = newBlocked.filter(c => c !== category)
      }

      return {
        ...prev,
        preferred_categories: newPreferred,
        blocked_categories: newBlocked,
      }
    })
  }

  /** Toggle category in blocked list */
  const toggleBlockedCategory = (category: AdCategory) => {
    setPreferences(prev => {
      const isInPreferred = prev.preferred_categories.includes(category)
      const isInBlocked = prev.blocked_categories.includes(category)

      let newPreferred = [...prev.preferred_categories]
      let newBlocked = [...prev.blocked_categories]

      if (isInBlocked) {
        // Remove from blocked
        newBlocked = newBlocked.filter(c => c !== category)
      } else {
        // Add to blocked, remove from preferred if present
        newBlocked.push(category)
        newPreferred = newPreferred.filter(c => c !== category)
      }

      return {
        ...prev,
        preferred_categories: newPreferred,
        blocked_categories: newBlocked,
      }
    })
  }

  /** Reset to defaults */
  const handleReset = () => {
    setPreferences(DEFAULT_AD_PREFERENCES)
  }

  /** Close handler */
  const handleClose = () => {
    setSaveSuccess(false)
    setError(null)
    onClose()
  }

  if (!isOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      role="dialog"
      aria-modal="true"
      aria-labelledby="ad-settings-title"
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black bg-opacity-50"
        onClick={handleClose}
        aria-hidden="true"
      />

      {/* Modal */}
      <div
        className="relative w-full max-w-2xl max-h-[90vh] overflow-y-auto rounded-lg shadow-xl"
        style={{ backgroundColor: 'var(--color-bg-primary)' }}
      >
        {/* Header */}
        <div
          className="sticky top-0 z-10 flex items-center justify-between p-6 border-b"
          style={{
            backgroundColor: 'var(--color-bg-primary)',
            borderColor: 'var(--color-border)',
          }}
        >
          <h2
            id="ad-settings-title"
            className="text-xl font-semibold"
            style={{ color: 'var(--color-text-primary)' }}
          >
            Настройки рекламы
          </h2>
          <button
            onClick={handleClose}
            className="p-2 rounded-lg transition-colors hover:bg-opacity-10"
            style={{ color: 'var(--color-text-muted)' }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--color-text-muted)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
            aria-label="Закрыть"
          >
            <FiX className="w-5 h-5" aria-hidden="true" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-8">
          {/* Privacy Guarantee */}
          <div
            className="p-4 rounded-lg border"
            style={{
              backgroundColor: 'var(--color-bg-secondary)',
              borderColor: 'var(--color-border)',
            }}
          >
            <div className="flex items-start gap-3">
              <FiShield className="w-6 h-6 flex-shrink-0 mt-1" style={{ color: 'var(--color-accent)' }} />
              <div>
                <h3 className="font-semibold mb-2" style={{ color: 'var(--color-text-primary)' }}>
                  Гарантия приватности
                </h3>
                <ul className="text-sm space-y-1" style={{ color: 'var(--color-text-secondary)' }}>
                  <li>✓ Все данные хранятся локально на вашем устройстве</li>
                  <li>✓ Таргетинг происходит БЕЗ отправки данных на сервер</li>
                  <li>✓ Отчёты отправляются анонимно (хеши без PII)</li>
                  <li>✓ Вы получаете кредиты за просмотр рекламы</li>
                </ul>
              </div>
            </div>
          </div>

          {/* Credits & Stats */}
          {settings && (
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {/* Credits */}
              <div
                className="p-4 rounded-lg border"
                style={{
                  backgroundColor: 'var(--color-bg-secondary)',
                  borderColor: 'var(--color-border)',
                }}
              >
                <div className="flex items-center gap-2 mb-2">
                  <FiAward className="w-5 h-5" style={{ color: 'var(--color-accent)' }} />
                  <span className="text-sm font-medium" style={{ color: 'var(--color-text-muted)' }}>
                    Кредиты
                  </span>
                </div>
                <p className="text-2xl font-bold" style={{ color: 'var(--color-text-primary)' }}>
                  {settings.credits}
                </p>
              </div>

              {/* Total Views */}
              <div
                className="p-4 rounded-lg border"
                style={{
                  backgroundColor: 'var(--color-bg-secondary)',
                  borderColor: 'var(--color-border)',
                }}
              >
                <div className="flex items-center gap-2 mb-2">
                  <FiEye className="w-5 h-5" style={{ color: 'var(--color-accent)' }} />
                  <span className="text-sm font-medium" style={{ color: 'var(--color-text-muted)' }}>
                    Всего просмотров
                  </span>
                </div>
                <p className="text-2xl font-bold" style={{ color: 'var(--color-text-primary)' }}>
                  {settings.stats.total_views}
                </p>
              </div>

              {/* Today Views */}
              <div
                className="p-4 rounded-lg border"
                style={{
                  backgroundColor: 'var(--color-bg-secondary)',
                  borderColor: 'var(--color-border)',
                }}
              >
                <div className="flex items-center gap-2 mb-2">
                  <FiClock className="w-5 h-5" style={{ color: 'var(--color-accent)' }} />
                  <span className="text-sm font-medium" style={{ color: 'var(--color-text-muted)' }}>
                    Сегодня
                  </span>
                </div>
                <p className="text-2xl font-bold" style={{ color: 'var(--color-text-primary)' }}>
                  {settings.stats.views_today}
                </p>
              </div>
            </div>
          )}

          {/* Ad Type Toggles */}
          <div>
            <h3 className="font-semibold mb-3" style={{ color: 'var(--color-text-primary)' }}>
              Типы рекламы
            </h3>
            <div className="space-y-3">
              {/* Banner Ads */}
              <label className="flex items-center justify-between cursor-pointer">
                <div>
                  <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
                    Баннерная реклама
                  </span>
                  <p className="text-xs" style={{ color: 'var(--color-text-muted)' }}>
                    Небольшие баннеры в списке чатов
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={preferences.enable_banner_ads}
                  onChange={(e) => setPreferences(prev => ({
                    ...prev,
                    enable_banner_ads: e.target.checked,
                  }))}
                  className="sr-only"
                />
                <div
                  className={clsx(
                    'relative w-12 h-6 rounded-full transition-colors',
                    preferences.enable_banner_ads ? 'bg-green-500' : 'bg-gray-500'
                  )}
                  onClick={() => setPreferences(prev => ({
                    ...prev,
                    enable_banner_ads: !prev.enable_banner_ads,
                  }))}
                  role="switch"
                  aria-checked={preferences.enable_banner_ads}
                >
                  <div
                    className={clsx(
                      'absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform',
                      preferences.enable_banner_ads && 'translate-x-6'
                    )}
                  />
                </div>
              </label>

              {/* Reward Ads */}
              <label className="flex items-center justify-between cursor-pointer">
                <div>
                  <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
                    Реклама за кредиты
                  </span>
                  <p className="text-xs" style={{ color: 'var(--color-text-muted)' }}>
                    Получайте кредиты за просмотр
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={preferences.enable_reward_ads}
                  onChange={(e) => setPreferences(prev => ({
                    ...prev,
                    enable_reward_ads: e.target.checked,
                  }))}
                  className="sr-only"
                />
                <div
                  className={clsx(
                    'relative w-12 h-6 rounded-full transition-colors',
                    preferences.enable_reward_ads ? 'bg-green-500' : 'bg-gray-500'
                  )}
                  onClick={() => setPreferences(prev => ({
                    ...prev,
                    enable_reward_ads: !prev.enable_reward_ads,
                  }))}
                  role="switch"
                  aria-checked={preferences.enable_reward_ads}
                >
                  <div
                    className={clsx(
                      'absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform',
                      preferences.enable_reward_ads && 'translate-x-6'
                    )}
                  />
                </div>
              </label>

              {/* Native Ads */}
              <label className="flex items-center justify-between cursor-pointer">
                <div>
                  <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
                    Нативная реклама
                  </span>
                  <p className="text-xs" style={{ color: 'var(--color-text-muted)' }}>
                    Реклама в стиле интерфейса
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={preferences.enable_native_ads}
                  onChange={(e) => setPreferences(prev => ({
                    ...prev,
                    enable_native_ads: e.target.checked,
                  }))}
                  className="sr-only"
                />
                <div
                  className={clsx(
                    'relative w-12 h-6 rounded-full transition-colors',
                    preferences.enable_native_ads ? 'bg-green-500' : 'bg-gray-500'
                  )}
                  onClick={() => setPreferences(prev => ({
                    ...prev,
                    enable_native_ads: !prev.enable_native_ads,
                  }))}
                  role="switch"
                  aria-checked={preferences.enable_native_ads}
                >
                  <div
                    className={clsx(
                      'absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform',
                      preferences.enable_native_ads && 'translate-x-6'
                    )}
                  />
                </div>
              </label>

              {/* Interstitial Ads */}
              <label className="flex items-center justify-between cursor-pointer">
                <div>
                  <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>
                    Межстраничная реклама
                  </span>
                  <p className="text-xs" style={{ color: 'var(--color-text-muted)' }}>
                    Полноэкранная реклама (опционально)
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={preferences.enable_interstitial_ads}
                  onChange={(e) => setPreferences(prev => ({
                    ...prev,
                    enable_interstitial_ads: e.target.checked,
                  }))}
                  className="sr-only"
                />
                <div
                  className={clsx(
                    'relative w-12 h-6 rounded-full transition-colors',
                    preferences.enable_interstitial_ads ? 'bg-green-500' : 'bg-gray-500'
                  )}
                  onClick={() => setPreferences(prev => ({
                    ...prev,
                    enable_interstitial_ads: !prev.enable_interstitial_ads,
                  }))}
                  role="switch"
                  aria-checked={preferences.enable_interstitial_ads}
                >
                  <div
                    className={clsx(
                      'absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform',
                      preferences.enable_interstitial_ads && 'translate-x-6'
                    )}
                  />
                </div>
              </label>
            </div>
          </div>

          {/* Max Ads Per Hour */}
          <div>
            <h3 className="font-semibold mb-3" style={{ color: 'var(--color-text-primary)' }}>
              Лимит рекламы
            </h3>
            <div className="flex items-center gap-4">
              <input
                type="range"
                min="1"
                max="30"
                value={preferences.max_ads_per_hour}
                onChange={(e) => setPreferences(prev => ({
                  ...prev,
                  max_ads_per_hour: parseInt(e.target.value),
                }))}
                className="flex-1"
                aria-label="Максимум рекламы в час"
              />
              <span
                className="text-sm font-medium px-3 py-1 rounded"
                style={{
                  backgroundColor: 'var(--color-bg-secondary)',
                  color: 'var(--color-text-primary)',
                }}
              >
                {preferences.max_ads_per_hour}/час
              </span>
            </div>
          </div>

          {/* Category Preferences */}
          <div>
            <h3 className="font-semibold mb-3" style={{ color: 'var(--color-text-primary)' }}>
              Категории рекламы
            </h3>
            <p className="text-sm mb-4" style={{ color: 'var(--color-text-muted)' }}>
              Выберите категории, которые вам интересны. Таргетинг происходит локально, без отправки данных на сервер.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {ALL_AD_CATEGORIES.map((category) => {
                const isPreferred = preferences.preferred_categories.includes(category)
                const isBlocked = preferences.blocked_categories.includes(category)

                return (
                  <div
                    key={category}
                    className={clsx(
                      'p-3 rounded-lg border transition-colors',
                      isPreferred && 'border-green-500',
                      isBlocked && 'border-red-500 opacity-50',
                      !isPreferred && !isBlocked && 'border-gray-500'
                    )}
                    style={{
                      backgroundColor: isPreferred
                        ? 'var(--color-bg-tertiary)'
                        : isBlocked
                        ? 'var(--color-bg-secondary)'
                        : 'var(--color-bg-secondary)',
                    }}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <span
                        className="font-medium text-sm"
                        style={{ color: 'var(--color-text-primary)' }}
                      >
                        {CATEGORY_LABELS[category]}
                      </span>
                      <div className="flex gap-2">
                        <button
                          onClick={() => togglePreferredCategory(category)}
                          className={clsx(
                            'p-1 rounded transition-colors',
                            isPreferred
                              ? 'text-green-500'
                              : 'text-gray-500 hover:text-green-500'
                          )}
                          aria-label={`Добавить ${CATEGORY_LABELS[category]} в предпочтения`}
                          title={isPreferred ? 'В предпочтениях' : 'Добавить в предпочтения'}
                        >
                          <FiCheck className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => toggleBlockedCategory(category)}
                          className={clsx(
                            'p-1 rounded transition-colors',
                            isBlocked
                              ? 'text-red-500'
                              : 'text-gray-500 hover:text-red-500'
                          )}
                          aria-label={`Заблокировать ${CATEGORY_LABELS[category]}`}
                          title={isBlocked ? 'Заблокировано' : 'Заблокировать'}
                        >
                          <FiX className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                    <div className="flex gap-2 text-xs">
                      {isPreferred && (
                        <span className="text-green-500">✓ Предпочитаю</span>
                      )}
                      {isBlocked && (
                        <span className="text-red-500">✗ Заблокировано</span>
                      )}
                      {!isPreferred && !isBlocked && (
                        <span style={{ color: 'var(--color-text-muted)' }}>Не выбрано</span>
                      )}
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div
          className="sticky bottom-0 p-6 border-t"
          style={{
            backgroundColor: 'var(--color-bg-primary)',
            borderColor: 'var(--color-border)',
          }}
        >
          <div className="flex items-center justify-between gap-4">
            {/* Error message */}
            {error && (
              <p className="text-sm text-red-500 flex-1">{error}</p>
            )}

            {/* Success message */}
            {saveSuccess && (
              <p className="text-sm text-green-500 flex-1">
                ✓ Настройки сохранены
              </p>
            )}

            {/* Actions */}
            <div className="flex gap-3">
              <button
                onClick={handleReset}
                className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                style={{
                  backgroundColor: 'var(--color-bg-secondary)',
                  color: 'var(--color-text-primary)',
                  border: '1px solid var(--color-border)',
                }}
              >
                Сбросить
              </button>
              <button
                onClick={handleClose}
                className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                style={{
                  backgroundColor: 'var(--color-bg-secondary)',
                  color: 'var(--color-text-primary)',
                  border: '1px solid var(--color-border)',
                }}
              >
                Отмена
              </button>
              <button
                onClick={handleSave}
                disabled={isSaving}
                className="px-6 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
                style={{
                  backgroundColor: 'var(--color-accent)',
                  color: 'white',
                }}
              >
                {isSaving ? 'Сохранение...' : 'Сохранить'}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
