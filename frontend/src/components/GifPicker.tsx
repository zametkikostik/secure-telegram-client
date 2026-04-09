/**
 * GifPicker — Search and send GIFs via Tenor API
 */
import { useState, useEffect, useRef } from 'react'

const TENOR_API_KEY = import.meta.env.VITE_TENOR_API_KEY || 'LIVDSRZULELA' // Demo key
const TENOR_BASE = 'https://tenor.googleapis.com/v2'

interface GifResult {
  id: string
  url: string
  media: { tinygif: { url: string; dims: [number, number] }[] }
  content_description: string
}

interface GifPickerProps {
  chatId: string
  onSend: (gifUrl: string) => void
  onClose: () => void
}

export const GifPicker: React.FC<GifPickerProps> = ({ chatId, onSend, onClose }) => {
  const [query, setQuery] = useState('')
  const [gifs, setGifs] = useState<GifResult[]>([])
  const [loading, setLoading] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => { inputRef.current?.focus() }, [])

  useEffect(() => {
    const timer = setTimeout(() => {
      if (query.length >= 2) searchGifs(query)
    }, 300)
    return () => clearTimeout(timer)
  }, [query])

  const searchGifs = async (q: string) => {
    setLoading(true)
    try {
      const res = await fetch(
        `${TENOR_BASE}/search?q=${encodeURIComponent(q)}&key=${TENOR_API_KEY}&limit=20&media_filter=tinygif`
      )
      const data = await res.json()
      setGifs(data.results || [])
    } catch (e) {
      console.error('GIF search failed:', e)
    } finally {
      setLoading(false)
    }
  }

  const handleSelect = (gif: GifResult) => {
    onSend(gif.media.tinygif[0].url)
    onClose()
  }

  return (
    <div className="absolute bottom-full right-0 mb-2 w-80 bg-gray-800 border border-gray-700 rounded-lg shadow-xl overflow-hidden z-40">
      <div className="p-2 border-b border-gray-700">
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={e => setQuery(e.target.value)}
          placeholder="Поиск GIF..."
          className="w-full px-3 py-1.5 bg-gray-700 text-white rounded text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
      </div>

      <div className="max-h-80 overflow-y-auto p-2">
        {loading && <p className="text-center text-gray-400 py-4">Загрузка...</p>}
        {!loading && gifs.length === 0 && (
          <p className="text-center text-gray-400 py-4 text-sm">Введите запрос для поиска GIF</p>
        )}
        <div className="grid grid-cols-2 gap-2">
          {gifs.map(gif => (
            <button
              key={gif.id}
              onClick={() => handleSelect(gif)}
              className="rounded overflow-hidden hover:opacity-80 transition-opacity bg-gray-700"
              title={gif.content_description}
            >
              <img src={gif.media.tinygif[0].url} alt={gif.content_description} className="w-full h-auto" loading="lazy" />
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
