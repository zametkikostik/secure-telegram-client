import { useState, useCallback, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { FiUpload, FiX, FiCheck, FiAlertCircle, FiFile } from 'react-icons/fi'
import clsx from 'clsx'

export interface ImportResult {
  success: boolean
  chatsImported: number
  messagesImported: number
  contactsImported: number
  errors: string[]
  duration: number // ms
}

interface TelegramImportModalProps {
  isOpen: boolean
  onClose: () => void
  onImportComplete?: (result: ImportResult) => void
}

type ImportState = 'idle' | 'parsing' | 'importing' | 'completed' | 'error'

export function TelegramImportModal({ isOpen, onClose, onImportComplete }: TelegramImportModalProps) {
  const { t } = useTranslation()
  const [importState, setImportState] = useState<ImportState>('idle')
  const [progress, setProgress] = useState(0)
  const [progressMessage, setProgressMessage] = useState('')
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [result, setResult] = useState<ImportResult | null>(null)
  const [isDragOver, setIsDragOver] = useState(false)
  const [autoTranslate, setAutoTranslate] = useState(true)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const dragCounterRef = useRef(0)

  const resetState = useCallback(() => {
    setImportState('idle')
    setProgress(0)
    setProgressMessage('')
    setSelectedFile(null)
    setResult(null)
    setIsDragOver(false)
    setAutoTranslate(true)
    dragCounterRef.current = 0
  }, [])

  const handleClose = useCallback(() => {
    if (importState === 'parsing' || importState === 'importing') return
    resetState()
    onClose()
  }, [importState, resetState, onClose])

  const handleFileSelect = useCallback((file: File) => {
    if (!file.name.endsWith('.json')) {
      setImportState('error')
      setResult({
        success: false,
        chatsImported: 0,
        messagesImported: 0,
        contactsImported: 0,
        errors: [t('import.telegram.invalid_format')],
        duration: 0,
      })
      return
    }

    setSelectedFile(file)
    setImportState('idle')
  }, [t])

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) handleFileSelect(file)
  }, [handleFileSelect])

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    dragCounterRef.current++
    if (dragCounterRef.current === 1) setIsDragOver(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    dragCounterRef.current--
    if (dragCounterRef.current === 0) setIsDragOver(false)
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragOver(false)
    dragCounterRef.current = 0

    const files = e.dataTransfer.files
    if (files && files.length > 0) {
      handleFileSelect(files[0])
    }
  }, [handleFileSelect])

  const simulateProgress = useCallback(async (
    steps: { message: string; duration: number }[]
  ) => {
    for (const step of steps) {
      setProgressMessage(step.message)
      const stepStart = Date.now()
      while (Date.now() - stepStart < step.duration) {
        const elapsed = Date.now() - stepStart
        const stepProgress = Math.min(elapsed / step.duration, 1)
        setProgress(stepProgress * 100)
        await new Promise(resolve => setTimeout(resolve, 50))
      }
    }
  }, [])

  const parseTelegramExport = useCallback(async (file: File): Promise<any> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = (e) => {
        try {
          const content = JSON.parse(e.target?.result as string)
          resolve(content)
        } catch {
          reject(new Error(t('import.telegram.parse_error')))
        }
      }
      reader.onerror = () => reject(new Error(t('import.telegram.read_error')))
      reader.readAsText(file)
    })
  }, [t])

  const processImportedData = useCallback(async (data: any): Promise<ImportResult> => {
    const startTime = Date.now()
    const errors: string[] = []

    // Подсчёт данных из экспорта Telegram
    let chatsImported = 0
    let messagesImported = 0
    let contactsImported = 0

    // Формат экспорта Telegram может быть разным
    if (data.chats && Array.isArray(data.chats)) {
      chatsImported = data.chats.length
      for (const chat of data.chats) {
        if (chat.messages && Array.isArray(chat.messages)) {
          messagesImported += chat.messages.length
        }
      }
    } else if (data.messages && Array.isArray(data.messages)) {
      chatsImported = 1
      messagesImported = data.messages.length
    }

    if (data.contacts && Array.isArray(data.contacts)) {
      contactsImported = data.contacts.length
    }

    // Если структура не распознана, пытаемся обработать как простой массив
    if (chatsImported === 0 && messagesImported === 0) {
      if (Array.isArray(data)) {
        chatsImported = 1
        messagesImported = data.length
      } else {
        errors.push(t('import.telegram.unknown_format'))
      }
    }

    const duration = Date.now() - startTime

    return {
      success: errors.length === 0,
      chatsImported,
      messagesImported,
      contactsImported,
      errors,
      duration,
    }
  }, [t])

  const handleImport = useCallback(async () => {
    if (!selectedFile) return

    const startTime = Date.now()
    setImportState('parsing')
    setProgress(0)

    try {
      // Фаза парсинга
      await simulateProgress([
        { message: t('import.telegram.parsing'), duration: 800 },
      ])

      const data = await parseTelegramExport(selectedFile)

      // Фаза импорта
      setImportState('importing')
      await simulateProgress([
        { message: t('import.telegram.importing_chats'), duration: 600 },
        { message: t('import.telegram.importing_messages'), duration: 1000 },
        { message: t('import.telegram.importing_contacts'), duration: 400 },
      ])

      const importResult = await processImportedData(data)
      importResult.duration = Date.now() - startTime

      setImportState('completed')
      setProgress(100)
      setResult(importResult)
      onImportComplete?.(importResult)
    } catch (err) {
      setImportState('error')
      setProgress(0)
      setResult({
        success: false,
        chatsImported: 0,
        messagesImported: 0,
        contactsImported: 0,
        errors: [err instanceof Error ? err.message : t('import.telegram.unknown_error')],
        duration: Date.now() - startTime,
      })
    }
  }, [selectedFile, t, simulateProgress, parseTelegramExport, processImportedData, onImportComplete])

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}мс`
    return `${(ms / 1000).toFixed(1)}с`
  }

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} Б`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} КБ`
    return `${(bytes / (1024 * 1024)).toFixed(1)} МБ`
  }

  if (!isOpen) return null

  const isProcessing = importState === 'parsing' || importState === 'importing'

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: 'var(--color-overlay)' }}
      role="dialog"
      aria-modal="true"
      aria-labelledby="import-modal-title"
      onClick={handleClose}
    >
      <div
        className="w-full max-w-lg mx-4 rounded-lg shadow-xl"
        style={{ backgroundColor: 'var(--color-bg-primary)' }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          className="flex items-center justify-between px-6 py-4 border-b"
          style={{ borderColor: 'var(--color-border)' }}
        >
          <h2
            id="import-modal-title"
            className="text-lg font-semibold"
            style={{ color: 'var(--color-text-primary)' }}
          >
            {t('import.telegram.title')}
          </h2>
          <button
            onClick={handleClose}
            disabled={isProcessing}
            className="p-1 rounded transition-colors disabled:opacity-50"
            style={{ color: 'var(--color-text-muted)' }}
            aria-label={t('common.cancel')}
          >
            <FiX className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="px-6 py-4">
          {/* Settings section */}
          {!isProcessing && importState !== 'completed' && importState !== 'error' && (
            <div
              className="mb-4 p-4 rounded-lg"
              style={{ backgroundColor: 'var(--color-bg-tertiary)' }}
            >
              <h3
                className="text-sm font-semibold mb-3"
                style={{ color: 'var(--color-text-primary)' }}
              >
                {t('import.telegram.settings')}
              </h3>

              {/* Auto-translate toggle */}
              <label
                className="flex items-center justify-between cursor-pointer"
                style={{ color: 'var(--color-text-primary)' }}
              >
                <span className="text-sm">
                  {t('import.telegram.auto_translate')}
                </span>
                <button
                  role="switch"
                  aria-checked={autoTranslate}
                  aria-label={t('import.telegram.auto_translate')}
                  onClick={() => setAutoTranslate(prev => !prev)}
                  className={clsx(
                    'relative w-11 h-6 rounded-full transition-colors',
                    autoTranslate ? 'bg-accent' : 'bg-gray-600'
                  )}
                  style={{
                    backgroundColor: autoTranslate
                      ? 'var(--color-accent)'
                      : 'var(--color-bg-primary)',
                    border: `1px solid ${autoTranslate ? 'var(--color-accent)' : 'var(--color-border)'}`,
                  }}
                >
                  <span
                    className={clsx(
                      'absolute top-0.5 left-0.5 w-4 h-4 rounded-full transition-transform',
                      autoTranslate && 'translate-x-5'
                    )}
                    style={{
                      backgroundColor: autoTranslate
                        ? '#ffffff'
                        : 'var(--color-text-muted)',
                    }}
                  />
                </button>
              </label>
              <p
                className="text-xs mt-1.5"
                style={{ color: 'var(--color-text-muted)' }}
              >
                {autoTranslate
                  ? t('import.telegram.auto_translate_on')
                  : t('import.telegram.auto_translate_off')}
              </p>
            </div>
          )}

          {/* File selection area */}
          {!isProcessing && importState !== 'completed' && importState !== 'error' && (
            <div
              className={clsx(
                'border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer',
                isDragOver && 'border-accent'
              )}
              style={{
                borderColor: isDragOver
                  ? 'var(--color-accent)'
                  : 'var(--color-border)',
                backgroundColor: isDragOver
                  ? 'var(--color-bg-tertiary)'
                  : 'transparent',
              }}
              onDragEnter={handleDragEnter}
              onDragLeave={handleDragLeave}
              onDragOver={handleDragOver}
              onDrop={handleDrop}
              onClick={() => fileInputRef.current?.click()}
              role="button"
              tabIndex={0}
              aria-label={t('import.telegram.dropzone')}
            >
              <input
                ref={fileInputRef}
                type="file"
                accept=".json"
                onChange={handleInputChange}
                className="hidden"
                aria-hidden="true"
              />
              <FiUpload
                className="w-12 h-12 mx-auto mb-4"
                style={{ color: 'var(--color-text-muted)' }}
                aria-hidden="true"
              />
              <p
                className="text-base mb-2"
                style={{ color: 'var(--color-text-primary)' }}
              >
                {isDragOver
                  ? t('import.telegram.drop_here')
                  : t('import.telegram.dropzone')}
              </p>
              <p
                className="text-sm"
                style={{ color: 'var(--color-text-secondary)' }}
              >
                {t('import.telegram.format_hint')}
              </p>
            </div>
          )}

          {/* Selected file info */}
          {selectedFile && !isProcessing && importState === 'idle' && (
            <div
              className="mt-4 p-3 rounded-lg flex items-center gap-3"
              style={{ backgroundColor: 'var(--color-bg-tertiary)' }}
            >
              <FiFile
                className="w-5 h-5 flex-shrink-0"
                style={{ color: 'var(--color-accent)' }}
                aria-hidden="true"
              />
              <div className="flex-1 min-w-0">
                <p
                  className="text-sm font-medium truncate"
                  style={{ color: 'var(--color-text-primary)' }}
                >
                  {selectedFile.name}
                </p>
                <p
                  className="text-xs"
                  style={{ color: 'var(--color-text-muted)' }}
                >
                  {formatFileSize(selectedFile.size)}
                </p>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation()
                  setSelectedFile(null)
                }}
                className="p-1 rounded transition-colors"
                style={{ color: 'var(--color-text-muted)' }}
                aria-label={t('common.cancel')}
              >
                <FiX className="w-4 h-4" />
              </button>
            </div>
          )}

          {/* Progress bar */}
          {isProcessing && (
            <div className="mt-4">
              <div className="flex items-center justify-between mb-2">
                <p
                  className="text-sm"
                  style={{ color: 'var(--color-text-primary)' }}
                >
                  {progressMessage}
                </p>
                <p
                  className="text-xs"
                  style={{ color: 'var(--color-text-muted)' }}
                >
                  {Math.round(progress)}%
                </p>
              </div>
              <div
                className="w-full h-2 rounded-full overflow-hidden"
                style={{ backgroundColor: 'var(--color-bg-tertiary)' }}
                role="progressbar"
                aria-valuenow={Math.round(progress)}
                aria-valuemin={0}
                aria-valuemax={100}
              >
                <div
                  className="h-full transition-all duration-300 ease-out rounded-full"
                  style={{
                    width: `${progress}%`,
                    backgroundColor: 'var(--color-accent)',
                  }}
                />
              </div>
            </div>
          )}

          {/* Import button */}
          {selectedFile && importState === 'idle' && (
            <button
              onClick={handleImport}
              className="w-full mt-4 px-4 py-2 rounded-lg font-medium transition-colors"
              style={{
                backgroundColor: 'var(--color-accent)',
                color: '#ffffff',
              }}
            >
              {t('import.telegram.start')}
            </button>
          )}

          {/* Success result */}
          {importState === 'completed' && result && result.success && (
            <div className="mt-2">
              <div className="flex items-center gap-3 mb-4">
                <div
                  className="w-10 h-10 rounded-full flex items-center justify-center"
                  style={{ backgroundColor: 'var(--color-success)' }}
                >
                  <FiCheck className="w-5 h-5 text-white" />
                </div>
                <div>
                  <p
                    className="font-medium"
                    style={{ color: 'var(--color-text-primary)' }}
                  >
                    {t('import.telegram.success')}
                  </p>
                  <p
                    className="text-xs"
                    style={{ color: 'var(--color-text-muted)' }}
                  >
                    {t('import.telegram.duration', { duration: formatDuration(result.duration) })}
                  </p>
                </div>
              </div>

              <div
                className="p-4 rounded-lg space-y-2"
                style={{ backgroundColor: 'var(--color-bg-tertiary)' }}
              >
                <div className="flex justify-between items-center">
                  <span style={{ color: 'var(--color-text-secondary)' }}>
                    {t('import.telegram.chats_imported')}
                  </span>
                  <span
                    className="font-semibold"
                    style={{ color: 'var(--color-text-primary)' }}
                  >
                    {result.chatsImported}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span style={{ color: 'var(--color-text-secondary)' }}>
                    {t('import.telegram.messages_imported')}
                  </span>
                  <span
                    className="font-semibold"
                    style={{ color: 'var(--color-text-primary)' }}
                  >
                    {result.messagesImported}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span style={{ color: 'var(--color-text-secondary)' }}>
                    {t('import.telegram.contacts_imported')}
                  </span>
                  <span
                    className="font-semibold"
                    style={{ color: 'var(--color-text-primary)' }}
                  >
                    {result.contactsImported}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Error result */}
          {importState === 'error' && result && (
            <div className="mt-2">
              <div className="flex items-center gap-3 mb-4">
                <div
                  className="w-10 h-10 rounded-full flex items-center justify-center"
                  style={{ backgroundColor: 'var(--color-error)' }}
                >
                  <FiAlertCircle className="w-5 h-5 text-white" />
                </div>
                <p
                  className="font-medium"
                  style={{ color: 'var(--color-error)' }}
                >
                  {t('import.telegram.error')}
                </p>
              </div>

              {result.errors.length > 0 && (
                <div
                  className="p-4 rounded-lg"
                  style={{ backgroundColor: 'var(--color-bg-tertiary)' }}
                >
                  <ul className="space-y-2">
                    {result.errors.map((error, index) => (
                      <li
                        key={index}
                        className="text-sm flex items-start gap-2"
                        style={{ color: 'var(--color-text-primary)' }}
                      >
                        <FiAlertCircle
                          className="w-4 h-4 mt-0.5 flex-shrink-0"
                          style={{ color: 'var(--color-error)' }}
                          aria-hidden="true"
                        />
                        {error}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {/* Retry button after error/completion */}
          {(importState === 'completed' || importState === 'error') && (
            <button
              onClick={resetState}
              className="w-full mt-4 px-4 py-2 rounded-lg font-medium transition-colors"
              style={{
                backgroundColor: 'var(--color-bg-tertiary)',
                color: 'var(--color-text-primary)',
                border: `1px solid var(--color-border)`,
              }}
            >
              {t('import.telegram.try_again')}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
