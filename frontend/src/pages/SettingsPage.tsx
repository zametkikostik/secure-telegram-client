import React from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';

export default function SettingsPage() {
  const navigate = useNavigate();
  const { user, logout } = useAuthStore();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="min-h-screen bg-gray-100">
      <div className="max-w-2xl mx-auto py-8 px-4">
        <h1 className="text-3xl font-bold text-gray-900 mb-8">Настройки</h1>

        {/* Профиль */}
        <div className="bg-white rounded-xl shadow p-6 mb-6">
          <h2 className="text-xl font-semibold mb-4">Профиль</h2>
          <div className="space-y-4">
            <div>
              <label className="text-sm text-gray-600">Имя пользователя</label>
              <p className="text-lg font-medium">{user?.username}</p>
            </div>
            <div>
              <label className="text-sm text-gray-600">ID</label>
              <p className="text-lg font-medium font-mono">{user?.id}</p>
            </div>
            <div>
              <label className="text-sm text-gray-600">Публичный ключ</label>
              <p className="text-sm font-mono break-all">{user?.public_key}</p>
            </div>
          </div>
        </div>

        {/* Настройки приложения */}
        <div className="bg-white rounded-xl shadow p-6 mb-6">
          <h2 className="text-xl font-semibold mb-4">Приложение</h2>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">AI Перевод</p>
                <p className="text-sm text-gray-600">Автоматически переводить сообщения</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" defaultChecked />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600"></div>
              </label>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Уведомления</p>
                <p className="text-sm text-gray-600">Показывать уведомления о сообщениях</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" defaultChecked />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600"></div>
              </label>
            </div>
          </div>
        </div>

        {/* Web3 */}
        <div className="bg-white rounded-xl shadow p-6 mb-6">
          <h2 className="text-xl font-semibold mb-4">Web3</h2>
          <div className="space-y-4">
            <div>
              <label className="text-sm text-gray-600">Кошелёк</label>
              <p className="text-lg font-medium font-mono">0x0000...0000</p>
            </div>
            <button className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition">
              Подключить MetaMask
            </button>
          </div>
        </div>

        {/* Выход */}
        <button
          onClick={handleLogout}
          className="w-full py-3 bg-red-600 text-white rounded-xl font-medium hover:bg-red-700 transition"
        >
          Выйти
        </button>
      </div>
    </div>
  );
}
