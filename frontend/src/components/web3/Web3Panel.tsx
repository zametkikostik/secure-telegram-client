/**
 * Web3Panel — Main Web3 dashboard with tabs for different features
 */
import { useState } from 'react'
import { TokenSwapForm } from './TokenSwapForm'
import { FiatPurchaseForm } from './FiatPurchaseForm'
import { TransactionHistory } from './TransactionHistory'

type Web3Tab = 'swap' | 'buy' | 'history'

const TABS: { id: Web3Tab; label: string; icon: string }[] = [
  { id: 'swap', label: 'Обмен токенов', icon: '🔄' },
  { id: 'buy', label: 'Покупка крипто', icon: '💳' },
  { id: 'history', label: 'История', icon: '📜' },
]

export const Web3Panel: React.FC = () => {
  const [activeTab, setActiveTab] = useState<Web3Tab>('swap')
  const [walletAddress, setWalletAddress] = useState<string>('')

  // TODO: Подключить реальный MetaMask/Tauri wallet
  // Пока используем заглушку для демо
  const handleConnectWallet = async () => {
    // Заглушка — в будущем здесь будет Tauri invoke
    setWalletAddress('0x' + '1234567890abcdef'.repeat(3) + 'demo')
  }

  // Если кошелёк не подключен — показываем кнопку подключения
  if (!walletAddress) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8">
        <div className="text-6xl mb-4">🦊</div>
        <h3 className="text-xl font-semibold text-white mb-2">Подключите кошелёк</h3>
        <p className="text-gray-400 text-sm text-center mb-6">
          Для использования Web3 функций необходимо подключить MetaMask или другой Web3 кошелёк
        </p>
        <button
          onClick={handleConnectWallet}
          className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors"
        >
          Подключить кошелёк
        </button>
        <p className="text-gray-500 text-xs mt-4">
          Демо-режим — тестирование без реального подключения
        </p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Tab Bar */}
      <div className="flex border-b border-gray-700">
        {TABS.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? 'bg-blue-600 text-white border-b-2 border-blue-400'
                : 'text-gray-400 hover:text-white hover:bg-gray-800'
            }`}
          >
            <span className="mr-2">{tab.icon}</span>
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === 'swap' && <TokenSwapForm walletAddress={walletAddress} />}
        {activeTab === 'buy' && <FiatPurchaseForm walletAddress={walletAddress} />}
        {activeTab === 'history' && <TransactionHistory />}
      </div>
    </div>
  )
}
