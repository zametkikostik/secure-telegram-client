import { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { FiX, FiExternalLink, FiAward } from 'react-icons/fi'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import type { Ad, SelectAdRequest, RecordImpressionRequest, RecordClickRequest } from '../types/ads'

interface AdBannerProps {
  /** Whether banner ads are enabled */
  enabled: boolean
  /** Called when ad is hidden */
  onHidden?: () => void
}

/**
 * Ad Banner Component
 * 
 * Displays a non-intrusive banner ad in the chat list.
 * - Positioned at the bottom of the chat list (not inside chat)
 * - User can hide the ad with "Скрыть" button
 * - Impressions and clicks are tracked locally and reported anonymously
 */
export function AdBanner({ enabled, onHidden }: AdBannerProps) {
  const { t } = useTranslation()
  const [ad, setAd] = useState<Ad | null>(null)
  const [isHidden, setIsHidden] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [viewStartTime, setViewStartTime] = useState<number>(0)
  const [error, setError] = useState<string | null>(null)

  /** Fetch a banner ad from the engine */
  const fetchAd = useCallback(async () => {
    if (!enabled || isHidden) return

    setIsLoading(true)
    setError(null)

    try {
      const request: SelectAdRequest = { ad_type: 'banner' }
      const response = await invoke<{ ad: Ad | null }>('select_ad', { request })

      if (response.ad) {
        setAd(response.ad)
        setViewStartTime(Date.now())
      }
    } catch (err) {
      console.error('[AdBanner] Failed to fetch ad:', err)
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [enabled, isHidden])

  /** Load ad on mount and when dependencies change */
  useEffect(() => {
    fetchAd()
  }, [fetchAd])

  /** Record impression when ad is visible */
  useEffect(() => {
    if (!ad || !viewStartTime) return

    const recordImpression = async () => {
      const durationSecs = Math.floor((Date.now() - viewStartTime) / 1000)
      
      try {
        const request: RecordImpressionRequest = {
          ad_id: ad.id,
          duration_secs: durationSecs,
        }

        await invoke('record_impression', { request })
      } catch (err) {
        // Silently fail - don't break UX on impression tracking errors
        console.debug('[AdBanner] Failed to record impression:', err)
      }
    }

    // Record impression after 3 seconds of visibility
    const timer = setTimeout(recordImpression, 3000)

    return () => clearTimeout(timer)
  }, [ad, viewStartTime])

  /** Handle ad click */
  const handleClick = async () => {
    if (!ad?.url) return

    try {
      const response = await invoke<{ success: boolean; url: string | null }>(
        'record_click',
        { request: { ad_id: ad.id } }
      )

      // Open URL in external browser
      if (response.url) {
        await open(response.url)
      }

      // Update ad viewed state
      setAd(prev => prev ? { ...prev, viewed: true, click_count: prev.click_count + 1 } : null)
    } catch (err) {
      console.error('[AdBanner] Failed to record click:', err)
    }
  }

  /** Handle hide button click */
  const handleHide = () => {
    setIsHidden(true)
    setAd(null)
    onHidden?.()
  }

  /** If disabled or hidden, don't render */
  if (!enabled || isHidden) {
    return null
  }

  /** Loading state */
  if (isLoading) {
    return (
      <div
        className="w-full p-4 flex items-center justify-center border-t"
        style={{
          backgroundColor: 'var(--color-bg-secondary)',
          borderColor: 'var(--color-border)',
        }}
      >
        <div
          className="w-6 h-6 border-2 border-t-transparent rounded-full animate-spin"
          style={{ borderColor: 'var(--color-accent)', borderTopColor: 'transparent' }}
          aria-hidden="true"
        />
      </div>
    )
  }

  /** Error state */
  if (error) {
    return (
      <div
        className="w-full p-3 border-t text-xs text-center"
        style={{
          backgroundColor: 'var(--color-bg-secondary)',
          borderColor: 'var(--color-border)',
          color: 'var(--color-text-muted)',
        }}
      >
        <span>Реклама недоступна</span>
      </div>
    )
  }

  /** No ad available */
  if (!ad) {
    return (
      <div
        className="w-full p-3 border-t text-xs text-center"
        style={{
          backgroundColor: 'var(--color-bg-secondary)',
          borderColor: 'var(--color-border)',
          color: 'var(--color-text-muted)',
        }}
      >
        <span>Реклама недоступна</span>
      </div>
    )
  }

  /** Render ad banner */
  return (
    <div
      className="w-full border-t transition-colors"
      style={{
        backgroundColor: 'var(--color-bg-secondary)',
        borderColor: 'var(--color-border)',
      }}
      role="complementary"
      aria-label="Рекламный баннер"
    >
      <div className="p-3">
        {/* Ad header */}
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <span
              className="text-xs font-medium"
              style={{ color: 'var(--color-text-muted)' }}
            >
              Реклама
            </span>
            {ad.credit_reward > 0 && (
              <span
                className="flex items-center gap-1 text-xs font-semibold px-2 py-0.5 rounded-full"
                style={{
                  backgroundColor: 'var(--color-accent)',
                  color: 'white',
                }}
              >
                <FiAward className="w-3 h-3" aria-hidden="true" />
                +{ad.credit_reward} кредитов
              </span>
            )}
          </div>

          {/* Hide button */}
          <button
            onClick={handleHide}
            className="p-1 rounded transition-colors hover:bg-opacity-20"
            style={{
              color: 'var(--color-text-muted)',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--color-text-muted)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
            aria-label="Скрыть рекламу"
            title="Скрыть"
          >
            <FiX className="w-4 h-4" aria-hidden="true" />
          </button>
        </div>

        {/* Ad content */}
        <div
          className="rounded-lg p-3 cursor-pointer transition-all hover:opacity-90"
          style={{
            backgroundColor: 'var(--color-bg-tertiary)',
          }}
          onClick={handleClick}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault()
              handleClick()
            }
          }}
          aria-label={`${ad.title} от ${ad.advertiser}`}
        >
          {/* Ad image (if available) */}
          {ad.image_url && (
            <div className="mb-2 rounded overflow-hidden">
              <img
                src={ad.image_url}
                alt=""
                className="w-full h-32 object-cover"
                loading="lazy"
              />
            </div>
          )}

          {/* Ad title and body */}
          <h4
            className="font-semibold text-sm mb-1"
            style={{ color: 'var(--color-text-primary)' }}
          >
            {ad.title}
          </h4>
          <p
            className="text-xs line-clamp-2"
            style={{ color: 'var(--color-text-secondary)' }}
          >
            {ad.body}
          </p>

          {/* CTA button */}
          {ad.cta && (
            <div className="mt-2 flex items-center justify-end">
              <span
                className="flex items-center gap-1 text-xs font-medium px-3 py-1.5 rounded-md transition-colors"
                style={{
                  backgroundColor: 'var(--color-accent)',
                  color: 'white',
                }}
              >
                {ad.cta}
                <FiExternalLink className="w-3 h-3" aria-hidden="true" />
              </span>
            </div>
          )}
        </div>

        {/* Advertiser attribution */}
        <div className="mt-2 text-xs" style={{ color: 'var(--color-text-muted)' }}>
          <span>от {ad.advertiser}</span>
        </div>
      </div>
    </div>
  )
}

/**
 * Ad Banner Placeholder
 * 
 * Shows a placeholder when ads are disabled or unavailable.
 */
export function AdBannerPlaceholder() {
  const { t } = useTranslation()

  return (
    <div
      className="w-full p-4 border-t text-center"
      style={{
        backgroundColor: 'var(--color-bg-secondary)',
        borderColor: 'var(--color-border)',
        color: 'var(--color-text-muted)',
      }}
    >
      <p className="text-xs">Реклама отключена в настройках</p>
    </div>
  )
}
