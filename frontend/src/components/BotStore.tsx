/**
 * BotStore — Marketplace for discovering and installing bots
 */

import React, { useEffect, useState } from 'react';
import { FiDownload, FiStar, FiSearch, FiUsers } from 'react-icons/fi';
import { useBotPlatformStore } from '../services/botPlatformStore';
import { BotStoreListing, BotCategory } from '../types/bot';
import { useTranslation } from 'react-i18next';

const CATEGORIES: { value: BotCategory | 'all'; label: string; icon: string }[] = [
  { value: 'all', label: 'All', icon: '🌐' },
  { value: BotCategory.Utility, label: 'Utility', icon: '🔧' },
  { value: BotCategory.Entertainment, label: 'Entertainment', icon: '🎭' },
  { value: BotCategory.AI, label: 'AI', icon: '🤖' },
  { value: BotCategory.Moderation, label: 'Moderation', icon: '🛡️' },
  { value: BotCategory.Integration, label: 'Integration', icon: '🔗' },
  { value: BotCategory.Games, label: 'Games', icon: '🎮' },
];

export const BotStore: React.FC = () => {
  const { t } = useTranslation();
  const { storeBots, loading, error, loadStore, installFromStore } = useBotPlatformStore();
  const [selectedCategory, setSelectedCategory] = useState<BotCategory | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<BotStoreListing[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  useEffect(() => {
    loadStore(selectedCategory === 'all' ? undefined : selectedCategory);
  }, [loadStore, selectedCategory]);

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;
    setIsSearching(true);
    try {
      const results = await useBotPlatformStore.getState().searchBots(searchQuery);
      setSearchResults(results);
    } catch {
      // Error handled by store
    }
    setIsSearching(false);
  };

  const displayBots = searchQuery.trim() ? searchResults : storeBots;

  return (
    <div className="w-full max-w-4xl mx-auto">
      {/* Header */}
      <h2 className="text-2xl font-bold text-white mb-6">{t('bot.store', 'Bot Store')}</h2>

      {/* Search */}
      <div className="flex gap-2 mb-4">
        <div className="relative flex-1">
          <FiSearch className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
          <input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder={t('bot.search_placeholder', 'Search bots...')}
            className="w-full pl-10 pr-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
          />
        </div>
        <button
          onClick={handleSearch}
          disabled={isSearching || !searchQuery.trim()}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg transition"
        >
          {t('common.search', 'Search')}
        </button>
      </div>

      {/* Categories */}
      <div className="flex gap-2 mb-6 overflow-x-auto pb-2">
        {CATEGORIES.map((cat) => (
          <button
            key={cat.value}
            onClick={() => {
              setSelectedCategory(cat.value);
              setSearchQuery('');
              setSearchResults([]);
            }}
            className={`flex items-center gap-1 px-3 py-1.5 rounded-full text-sm whitespace-nowrap transition ${
              selectedCategory === cat.value
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
          >
            <span>{cat.icon}</span>
            <span>{cat.label}</span>
          </button>
        ))}
      </div>

      {/* Error */}
      {error && (
        <div className="mb-4 p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-300 text-sm">
          {error}
        </div>
      )}

      {/* Bot Grid */}
      {loading || isSearching ? (
        <div className="text-center text-gray-500 py-8">{t('common.loading', 'Loading...')}</div>
      ) : displayBots.length === 0 ? (
        <div className="text-center text-gray-500 py-12">
          <p className="text-lg mb-2">🔍</p>
          <p>{t('bot.no_results', 'No bots found')}</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {displayBots.map((bot) => (
            <BotStoreCard key={bot.id} bot={bot} />
          ))}
        </div>
      )}
    </div>
  );
};

// ============================================================================
// Bot Store Card
// ============================================================================

interface BotStoreCardProps {
  bot: BotStoreListing;
}

const BotStoreCard: React.FC<BotStoreCardProps> = ({ bot }) => {
  const { t } = useTranslation();
  const { installFromStore } = useBotPlatformStore();
  const [installing, setInstalling] = useState(false);

  const handleInstall = async () => {
    setInstalling(true);
    try {
      await installFromStore(bot.id);
    } catch {
      // Error handled by store
    }
    setInstalling(false);
  };

  return (
    <div className="p-4 bg-gray-800 rounded-xl border border-gray-700 hover:border-gray-600 transition flex flex-col">
      {/* Header */}
      <div className="flex items-start gap-3 mb-3">
        {bot.avatar_url ? (
          <img src={bot.avatar_url} alt={bot.name} className="w-12 h-12 rounded-full" />
        ) : (
          <div className="w-12 h-12 rounded-full bg-gradient-to-br from-green-400 to-blue-500 flex items-center justify-center text-white text-xl font-bold">
            {bot.name[0].toUpperCase()}
          </div>
        )}

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h4 className="font-semibold text-white truncate">{bot.name}</h4>
            {bot.is_verified && (
              <span className="text-blue-400 text-xs" title="Verified">✓</span>
            )}
            {bot.is_premium && (
              <span className="text-yellow-400 text-xs" title="Premium">⭐</span>
            )}
          </div>
          <p className="text-xs text-gray-500">@{bot.username}</p>
        </div>
      </div>

      {/* Description */}
      <p className="text-sm text-gray-400 mb-3 flex-1">{bot.description}</p>

      {/* Category Badge */}
      <span className="text-xs px-2 py-0.5 bg-purple-500/20 text-purple-400 rounded-full self-start mb-3">
        {bot.category}
      </span>

      {/* Commands Preview */}
      {bot.commands.length > 0 && (
        <div className="mb-3">
          <p className="text-xs text-gray-500 mb-1">{t('bot.commands', 'Commands')}:</p>
          <div className="flex flex-wrap gap-1">
            {bot.commands.slice(0, 3).map((cmd) => (
              <code key={cmd} className="text-xs bg-gray-700 px-1.5 py-0.5 rounded text-blue-300">
                {cmd}
              </code>
            ))}
            {bot.commands.length > 3 && (
              <span className="text-xs text-gray-500">+{bot.commands.length - 3}</span>
            )}
          </div>
        </div>
      )}

      {/* Stats & Install */}
      <div className="flex items-center justify-between pt-3 border-t border-gray-700">
        <div className="flex items-center gap-3 text-xs text-gray-500">
          <span className="flex items-center gap-1">
            <FiUsers className="w-3 h-3" />
            {bot.install_count.toLocaleString()}
          </span>
          <span className="flex items-center gap-1">
            <FiStar className="w-3 h-3 text-yellow-400" />
            {bot.rating.toFixed(1)}
          </span>
        </div>

        <button
          onClick={handleInstall}
          disabled={installing}
          className="flex items-center gap-1 px-3 py-1.5 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white text-sm rounded-lg transition"
        >
          <FiDownload className="w-3 h-3" />
          {installing ? t('bot.installing', 'Installing...') : t('bot.install', 'Install')}
        </button>
      </div>
    </div>
  );
};
