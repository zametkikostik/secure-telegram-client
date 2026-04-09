/**
 * AI-модули для обработки импортированных данных из Telegram
 *
 * Включает:
 * - Автоперевод сообщений через AI
 * - Классификация чатов (личные/группы/каналы)
 * - Анализ тональности
 * - Rate limiting для внешних API
 * - Обработка ошибок с retry
 */

// ============================================================
// Types
// ============================================================

export interface AIMessageTranslation {
  originalText: string
  translatedText: string
  sourceLang: string
  targetLang: string
  confidence: number
}

export interface AIChatClassification {
  chatType: 'private' | 'group' | 'channel'
  confidence: number
  tags: string[]
}

export interface AISentimentAnalysis {
  sentiment: 'positive' | 'neutral' | 'negative'
  score: number // -1 to 1
  label: string
}

export interface AIProcessingResult {
  translations: AIMessageTranslation[]
  classifications: AIChatClassification[]
  sentiments: AISentimentAnalysis[]
  stats: {
    totalProcessed: number
    successful: number
    failed: number
    rateLimited: number
  }
}

export interface RateLimitConfig {
  maxRequestsPerMinute: number
  maxRequestsPerHour: number
  retryAfterMs: number
  maxRetries: number
  backoffMultiplier: number
}

export interface RetryConfig {
  maxRetries: number
  initialDelayMs: number
  maxDelayMs: number
  backoffMultiplier: number
  retryableErrors: string[]
}

export const DEFAULT_RATE_LIMIT: RateLimitConfig = {
  maxRequestsPerMinute: 60,
  maxRequestsPerHour: 1000,
  retryAfterMs: 60000,
  maxRetries: 3,
  backoffMultiplier: 2,
}

export const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  initialDelayMs: 1000,
  maxDelayMs: 30000,
  backoffMultiplier: 2,
  retryableErrors: [
    'ECONNRESET',
    'ETIMEDOUT',
    'ECONNREFUSED',
    'NETWORK_ERROR',
    'RATE_LIMITED',
    '503',
    '502',
    '429',
  ],
}

// ============================================================
// Rate Limiter
// ============================================================

export class RateLimiter {
  private minuteBucket: number[] = []
  private hourBucket: number[] = []
  private config: RateLimitConfig

  constructor(config: RateLimitConfig = DEFAULT_RATE_LIMIT) {
    this.config = config
  }

  canMakeRequest(): { allowed: boolean; retryAfterMs?: number } {
    const now = Date.now()

    // Очистка старых записей
    this.minuteBucket = this.minuteBucket.filter(t => now - t < 60000)
    this.hourBucket = this.hourBucket.filter(t => now - t < 3600000)

    // Проверка лимитов
    if (this.minuteBucket.length >= this.config.maxRequestsPerMinute) {
      const oldestInMinute = Math.min(...this.minuteBucket)
      const retryAfter = 60000 - (now - oldestInMinute)
      return { allowed: false, retryAfterMs: Math.max(retryAfter, 1000) }
    }

    if (this.hourBucket.length >= this.config.maxRequestsPerHour) {
      const oldestInHour = Math.min(...this.hourBucket)
      const retryAfter = 3600000 - (now - oldestInHour)
      return { allowed: false, retryAfterMs: Math.max(retryAfter, 1000) }
    }

    return { allowed: true }
  }

  recordRequest(): void {
    const now = Date.now()
    this.minuteBucket.push(now)
    this.hourBucket.push(now)
  }

  getUsage(): { perMinute: number; perHour: number } {
    const now = Date.now()
    const minuteCount = this.minuteBucket.filter(t => now - t < 60000).length
    const hourCount = this.hourBucket.filter(t => now - t < 3600000).length
    return { perMinute: minuteCount, perHour: hourCount }
  }

  reset(): void {
    this.minuteBucket = []
    this.hourBucket = []
  }
}

// ============================================================
// Retry Handler
// ============================================================

export async function withRetry<T>(
  fn: () => Promise<T>,
  config: Partial<RetryConfig> = {},
  onRetry?: (attempt: number, error: Error, delayMs: number) => void
): Promise<T> {
  const mergedConfig = { ...DEFAULT_RETRY_CONFIG, ...config }
  let lastError: Error | null = null

  for (let attempt = 0; attempt <= mergedConfig.maxRetries; attempt++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error))

      const errorCode = lastError.message.toUpperCase()
      const isRetryable = mergedConfig.retryableErrors.some(
        code => errorCode.includes(code)
      )

      if (!isRetryable || attempt === mergedConfig.maxRetries) {
        throw lastError
      }

      // Экспоненциальный backoff с джиттером
      const baseDelay = Math.min(
        mergedConfig.initialDelayMs * Math.pow(mergedConfig.backoffMultiplier, attempt),
        mergedConfig.maxDelayMs
      )
      const jitter = Math.random() * 0.3 * baseDelay // ±15% джиттер
      const delayMs = Math.round(baseDelay + jitter - 0.15 * baseDelay)

      onRetry?.(attempt + 1, lastError, delayMs)

      await new Promise(resolve => setTimeout(resolve, delayMs))
    }
  }

  throw lastError ?? new Error('Unexpected retry error')
}

// ============================================================
// AI Translation Module
// ============================================================

export interface TranslationOptions {
  targetLang: string
  rateLimit?: Partial<RateLimitConfig>
  retry?: Partial<RetryConfig>
  onProgress?: (processed: number, total: number) => void
  onError?: (error: Error, message: string) => void
}

export async function translateMessages(
  messages: Array<{ id: string; text: string; sourceLang?: string }>,
  options: TranslationOptions
): Promise<AIMessageTranslation[]> {
  const { targetLang, onProgress, onError } = options
  const rateLimiter = new RateLimiter({
    ...DEFAULT_RATE_LIMIT,
    ...options.rateLimit,
  })

  const results: AIMessageTranslation[] = []
  const retryConfig = { ...DEFAULT_RETRY_CONFIG, ...options.retry }

  for (let i = 0; i < messages.length; i++) {
    const msg = messages[i]

    // Rate limiting
    const rateCheck = rateLimiter.canMakeRequest()
    if (!rateCheck.allowed) {
      onError?.(
        new Error('Rate limited'),
        `Rate limited, waiting ${rateCheck.retryAfterMs}ms`
      )
      await new Promise(resolve => setTimeout(resolve, rateCheck.retryAfterMs))
    }

    rateLimiter.recordRequest()

    try {
      const translation = await withRetry(
        async () => {
          // TODO: Заменить на реальный AI API (OpenAI, Claude, локальная модель)
          return await mockTranslateMessage(
            msg.text,
            msg.sourceLang ?? 'auto',
            targetLang
          )
        },
        retryConfig,
        (attempt, error, delay) => {
          console.warn(
            `[AI Translate] Retry attempt ${attempt} for message ${msg.id}:`,
            error.message,
            `delay: ${delay}ms`
          )
        }
      )

      results.push(translation)
    } catch (error) {
      onError?.(
        error instanceof Error ? error : new Error(String(error)),
        `Failed to translate message ${msg.id}: "${msg.text.slice(0, 50)}..."`
      )
      // Добавляем заглушку с оригинальным текстом
      results.push({
        originalText: msg.text,
        translatedText: msg.text,
        sourceLang: msg.sourceLang ?? 'unknown',
        targetLang,
        confidence: 0,
      })
    }

    onProgress?.(i + 1, messages.length)
  }

  return results
}

async function mockTranslateMessage(
  text: string,
  _sourceLang: string,
  targetLang: string
): Promise<AIMessageTranslation> {
  // Симуляция задержки API
  await new Promise(resolve => setTimeout(resolve, 50 + Math.random() * 100))

  // Простая "трансляция" для демо — в реальности здесь вызов AI API
  const detectedLang = detectLanguage(text)

  return {
    originalText: text,
    translatedText: `[${detectedLang}→${targetLang}] ${text}`,
    sourceLang: detectedLang,
    targetLang,
    confidence: 0.85 + Math.random() * 0.15,
  }
}

// Простая эвристика определения языка
function detectLanguage(text: string): string {
  const cyrillic = /[\u0400-\u04FF]/
  const arabic = /[\u0600-\u06FF]/
  const chinese = /[\u4E00-\u9FFF]/
  const thai = /[\u0E00-\u0E7F]/

  if (cyrillic.test(text)) return 'ru'
  if (arabic.test(text)) return 'ar'
  if (chinese.test(text)) return 'zh'
  if (thai.test(text)) return 'th'
  return 'en'
}

// ============================================================
// AI Chat Classification Module
// ============================================================

export interface ClassificationOptions {
  rateLimit?: Partial<RateLimitConfig>
  retry?: Partial<RetryConfig>
}

export async function classifyChat(
  chatData: {
    name: string
    participantCount?: number
    messageCount: number
    hasAdminRights?: boolean
    isBroadcast?: boolean
  },
  options: ClassificationOptions = {}
): Promise<AIChatClassification> {
  const rateLimiter = new RateLimiter({
    ...DEFAULT_RATE_LIMIT,
    ...options.rateLimit,
  })

  const rateCheck = rateLimiter.canMakeRequest()
  if (!rateCheck.allowed) {
    await new Promise(resolve => setTimeout(resolve, rateCheck.retryAfterMs))
  }

  rateLimiter.recordRequest()

  return withRetry(
    async () => {
      // Эвристическая классификация — заменить на AI API
      let chatType: 'private' | 'group' | 'channel' = 'private'
      let confidence = 0.7
      const tags: string[] = []

      if (chatData.isBroadcast || chatData.hasAdminRights) {
        chatType = 'channel'
        confidence = 0.9
        tags.push('broadcast')
      } else if ((chatData.participantCount ?? 1) > 2) {
        chatType = 'group'
        confidence = 0.85
        tags.push('multi-user')
        if ((chatData.participantCount ?? 0) > 50) {
          tags.push('large-group')
        }
      }

      if (chatData.messageCount > 1000) {
        tags.push('active')
      }
      if (chatData.messageCount < 10) {
        tags.push('inactive')
      }

      return { chatType, confidence, tags }
    },
    { ...DEFAULT_RETRY_CONFIG, ...options.retry }
  )
}

// ============================================================
// AI Sentiment Analysis Module
// ============================================================

export interface SentimentOptions {
  rateLimit?: Partial<RateLimitConfig>
  retry?: Partial<RetryConfig>
}

export async function analyzeSentiment(
  text: string,
  options: SentimentOptions = {}
): Promise<AISentimentAnalysis> {
  const rateLimiter = new RateLimiter({
    ...DEFAULT_RATE_LIMIT,
    ...options.rateLimit,
  })

  const rateCheck = rateLimiter.canMakeRequest()
  if (!rateCheck.allowed) {
    await new Promise(resolve => setTimeout(resolve, rateCheck.retryAfterMs))
  }

  rateLimiter.recordRequest()

  return withRetry(
    async () => {
      // Простая эвристика — заменить на AI API
      const positiveWords = [
        'good', 'great', 'excellent', 'thanks', 'love', 'happy', 'ok', '👍', '❤️',
        'хорошо', 'отлично', 'спасибо', 'люблю', 'счастлив', 'да', '👍', '❤️',
      ]
      const negativeWords = [
        'bad', 'terrible', 'hate', 'angry', 'sad', 'worst', 'no', '👎', '💔',
        'плохо', 'ужасно', 'ненавижу', 'злюсь', 'грустно', 'нет', '👎', '💔',
      ]

      const lowerText = text.toLowerCase()
      let score = 0

      for (const word of positiveWords) {
        if (lowerText.includes(word)) score += 0.3
      }
      for (const word of negativeWords) {
        if (lowerText.includes(word)) score -= 0.3
      }

      // Ограничиваем [-1, 1]
      score = Math.max(-1, Math.min(1, score))

      let sentiment: 'positive' | 'neutral' | 'negative'
      let label: string

      if (score > 0.1) {
        sentiment = 'positive'
        label = 'Positive'
      } else if (score < -0.1) {
        sentiment = 'negative'
        label = 'Negative'
      } else {
        sentiment = 'neutral'
        label = 'Neutral'
      }

      return { sentiment, score, label }
    },
    { ...DEFAULT_RETRY_CONFIG, ...options.retry }
  )
}

// ============================================================
// Batch Processor — orchestration with error tracking
// ============================================================

export interface BatchProcessorOptions {
  targetLang: string
  batchSize?: number
  concurrency?: number
  rateLimit?: Partial<RateLimitConfig>
  retry?: Partial<RetryConfig>
  onProgress?: (current: number, total: number) => void
  onError?: (error: Error, context: string) => void
}

export async function processImportedData(
  chats: Array<{
    id: string
    name: string
    messages: Array<{ id: string; text: string; sourceLang?: string }>
    participantCount?: number
    messageCount: number
    hasAdminRights?: boolean
    isBroadcast?: boolean
  }>,
  options: BatchProcessorOptions
): Promise<AIProcessingResult> {
  const {
    targetLang,
    batchSize = 10,
    concurrency = 3,
    onProgress,
    onError,
  } = options

  const translations: AIMessageTranslation[] = []
  const classifications: AIChatClassification[] = []
  const sentiments: AISentimentAnalysis[] = []

  let totalProcessed = 0
  let totalSuccessful = 0
  let totalFailed = 0
  let totalRateLimited = 0

  const totalItems = chats.reduce(
    (sum, chat) => sum + chat.messages.length + 1,
    0
  ) // +1 за классификацию чата

  const rateLimiter = new RateLimiter({
    ...DEFAULT_RATE_LIMIT,
    ...options.rateLimit,
  })

  // Обработка чатов батчами
  for (let batchStart = 0; batchStart < chats.length; batchStart += batchSize) {
    const batch = chats.slice(batchStart, batchStart + batchSize)

    // Параллельная обработка с ограничением concurrency
    const chunks: typeof batch[] = []
    for (let i = 0; i < batch.length; i += concurrency) {
      chunks.push(batch.slice(i, i + concurrency))
    }

    for (const chunk of chunks) {
      const promises = chunk.map(async (chat) => {
        try {
          // Классификация чата
          const classification = await classifyChat(
            {
              name: chat.name,
              participantCount: chat.participantCount,
              messageCount: chat.messageCount,
              hasAdminRights: chat.hasAdminRights,
              isBroadcast: chat.isBroadcast,
            },
            { rateLimit: options.rateLimit, retry: options.retry }
          )
          classifications.push(classification)
          totalSuccessful++

          // Перевод сообщений
          for (const msg of chat.messages) {
            try {
              const rateCheck = rateLimiter.canMakeRequest()
              if (!rateCheck.allowed) {
                totalRateLimited++
                await new Promise(resolve =>
                  setTimeout(resolve, rateCheck.retryAfterMs)
                )
              }

              rateLimiter.recordRequest()

              const translation = await withRetry(
                () => mockTranslateMessage(msg.text, msg.sourceLang ?? 'auto', targetLang),
                { ...DEFAULT_RETRY_CONFIG, ...options.retry },
                (attempt, error, _delay) => {
                  console.warn(
                    `[BatchProcessor] Retry ${attempt} for chat ${chat.id}/${msg.id}:`,
                    error.message
                  )
                }
              )

              translations.push(translation)
              totalSuccessful++

              // Анализ тональности для коротких сообщений
              if (msg.text.length < 500) {
                const sentiment = await analyzeSentiment(msg.text, {
                  rateLimit: options.rateLimit,
                  retry: options.retry,
                })
                sentiments.push(sentiment)
              }
            } catch (error) {
              totalFailed++
              onError?.(
                error instanceof Error ? error : new Error(String(error)),
                `Failed to process message ${msg.id} in chat ${chat.id}`
              )
            }

            totalProcessed++
            onProgress?.(totalProcessed, totalItems)
          }
        } catch (error) {
          totalFailed++
          onError?.(
            error instanceof Error ? error : new Error(String(error)),
            `Failed to classify chat ${chat.id}`
          )
        }

        totalProcessed++
        onProgress?.(totalProcessed, totalItems)
      })

      await Promise.all(promises)
    }
  }

  return {
    translations,
    classifications,
    sentiments,
    stats: {
      totalProcessed,
      successful: totalSuccessful,
      failed: totalFailed,
      rateLimited: totalRateLimited,
    },
  }
}

// ============================================================
// Error Reporter
// ============================================================

export interface AIErrorReport {
  timestamp: string
  module: string
  error: string
  context: Record<string, unknown>
  retryCount: number
  isRecovered: boolean
}

export class AIErrorReporter {
  private reports: AIErrorReport[] = []
  private maxReports = 100

  report(
    module: string,
    error: Error,
    context: Record<string, unknown>,
    retryCount: number,
    isRecovered: boolean
  ): void {
    if (this.reports.length >= this.maxReports) {
      this.reports.shift() // Удаляем старые
    }

    this.reports.push({
      timestamp: new Date().toISOString(),
      module,
      error: error.message,
      context,
      retryCount,
      isRecovered,
    })

    console.error(`[AI Error] [${module}] ${error.message}`, {
      context,
      retries: retryCount,
      recovered: isRecovered,
    })
  }

  getReports(): AIErrorReport[] {
    return [...this.reports]
  }

  getSummary(): {
    total: number
    byModule: Record<string, number>
    recovered: number
    unrecovered: number
  } {
    const byModule: Record<string, number> = {}
    let recovered = 0
    let unrecovered = 0

    for (const report of this.reports) {
      byModule[report.module] = (byModule[report.module] || 0) + 1
      if (report.isRecovered) recovered++
      else unrecovered++
    }

    return {
      total: this.reports.length,
      byModule,
      recovered,
      unrecovered,
    }
  }

  clear(): void {
    this.reports = []
  }
}

// ============================================================
// Export public API
// ============================================================

export {
  RateLimiter as AILimiter,
  withRetry as AIWithRetry,
  translateMessages as AITranslateMessages,
  classifyChat as AIClassifyChat,
  analyzeSentiment as AIAnalyzeSentiment,
  processImportedData as AIProcessImportedData,
}
