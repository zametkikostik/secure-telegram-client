/**
 * ProfileSettings — User profile with family status
 */
import { useState, useEffect } from 'react'
import { api } from '../services/apiClient'

const FAMILY_STATUSES = [
  { value: 'none', label: 'Не указано', emoji: '' },
  { value: 'single', label: 'В активном поиске', emoji: '💔' },
  { value: 'relationship', label: 'В отношениях', emoji: '💑' },
  { value: 'married', label: 'В браке', emoji: '💍' },
  { value: 'divorced', label: 'Разведён(а)', emoji: '🔓' },
  { value: 'widowed', label: 'Вдовец/Вдова', emoji: '🕊️' },
]

export const ProfileSettings: React.FC = () => {
  const [displayName, setDisplayName] = useState('')
  const [familyStatus, setFamilyStatus] = useState('none')
  const [saving, setSaving] = useState(false)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    api.getMe().then(user => {
      setDisplayName(user.display_name || user.username)
      setFamilyStatus((user as any).family_status || 'none')
    }).catch(console.error)
  }, [])

  const handleSave = async () => {
    setSaving(true)
    setSaved(false)
    try {
      await api.updateProfile(displayName, undefined, undefined, undefined, familyStatus)
      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch (e) {
      console.error('Failed to save profile:', e)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="p-4 space-y-4">
      <h3 className="text-lg font-semibold text-white">Профиль</h3>

      {/* Display Name */}
      <div>
        <label className="block text-sm text-gray-400 mb-1">Имя</label>
        <input
          type="text"
          value={displayName}
          onChange={e => setDisplayName(e.target.value)}
          className="w-full px-3 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
        />
      </div>

      {/* Family Status */}
      <div>
        <label className="block text-sm text-gray-400 mb-1">Семейное положение</label>
        <select
          value={familyStatus}
          onChange={e => setFamilyStatus(e.target.value)}
          className="w-full px-3 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
        >
          {FAMILY_STATUSES.map(s => (
            <option key={s.value} value={s.value}>{s.emoji} {s.label}</option>
          ))}
        </select>
      </div>

      {/* Save Button */}
      <button
        onClick={handleSave}
        disabled={saving}
        className="w-full py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg transition"
      >
        {saving ? 'Сохранение...' : saved ? '✓ Сохранено!' : 'Сохранить'}
      </button>
    </div>
  )
}
