/**
 * BotManager — Main panel for managing user's bots
 */

import React, { useEffect, useState } from 'react';
import { FiPlus, FiSettings, FiTrash2, FiCopy, FiRefreshCw } from 'react-icons/fi';
import { useBotPlatformStore } from '../services/botPlatformStore';
import { Bot, BotHandlerType, CreateBotPayload } from '../types/bot';
import { useTranslation } from 'react-i18next';

export const BotManager: React.FC = () => {
  const { t } = useTranslation();
  const { myBots, loading, error, loadMyBots, createBot, deleteBot, selectBot } =
    useBotPlatformStore();

  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newBot, setNewBot] = useState<Partial<CreateBotPayload>>({
    handler_type: BotHandlerType.Internal,
  });
  const [copiedToken, setCopiedToken] = useState<string | null>(null);
  const [createdBotToken, setCreatedBotToken] = useState<string | null>(null);

  useEffect(() => {
    loadMyBots();
  }, [loadMyBots]);

  const handleCreate = async () => {
    if (!newBot.name || !newBot.username) return;

    try {
      const username = newBot.username.startsWith('@')
        ? newBot.username.slice(1)
        : newBot.username;

      const bot = await createBot({
        name: newBot.name,
        username,
        description: newBot.description,
        handler_type: newBot.handler_type || BotHandlerType.Internal,
        webhook_url: newBot.webhook_url,
        ai_prompt: newBot.ai_prompt,
      });

      if (bot.token) {
        setCreatedBotToken(bot.token);
      }

      setShowCreateForm(false);
      setNewBot({ handler_type: BotHandlerType.Internal });
    } catch {
      // Error handled by store
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedToken(text);
    setTimeout(() => setCopiedToken(null), 2000);
  };

  return (
    <div className="w-full max-w-2xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">{t('bot.manager', 'My Bots')}</h2>
        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
        >
          <FiPlus className="w-4 h-4" />
          {t('bot.create', 'Create Bot')}
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-4 p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-300 text-sm">
          {error}
        </div>
      )}

      {/* Created Bot Token Display */}
      {createdBotToken && (
        <div className="mb-4 p-4 bg-yellow-500/20 border border-yellow-500/50 rounded-lg">
          <p className="text-yellow-300 text-sm font-semibold mb-2">
            ⚠️ {t('bot.token_warning', 'Save this token! It won\'t be shown again.')}
          </p>
          <div className="flex items-center gap-2">
            <code className="flex-1 bg-gray-900 px-3 py-2 rounded text-green-400 text-sm font-mono">
              {createdBotToken}
            </code>
            <button
              onClick={() => copyToClipboard(createdBotToken)}
              className="p-2 bg-gray-700 hover:bg-gray-600 rounded transition"
            >
              <FiCopy className="w-4 h-4 text-white" />
            </button>
            {copiedToken === createdBotToken && (
              <span className="text-green-400 text-sm">Copied!</span>
            )}
          </div>
        </div>
      )}

      {/* Create Form */}
      {showCreateForm && (
        <div className="mb-6 p-4 bg-gray-800 rounded-xl border border-gray-700">
          <h3 className="text-lg font-semibold text-white mb-4">
            {t('bot.new_bot', 'New Bot')}
          </h3>

          <div className="space-y-3">
            <input
              value={newBot.name || ''}
              onChange={(e) => setNewBot({ ...newBot, name: e.target.value })}
              placeholder={t('bot.name_placeholder', 'Bot name')}
              className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
            />

            <input
              value={newBot.username || ''}
              onChange={(e) => setNewBot({ ...newBot, username: e.target.value })}
              placeholder={t('bot.username_placeholder', '@bot_username')}
              className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
            />

            <textarea
              value={newBot.description || ''}
              onChange={(e) => setNewBot({ ...newBot, description: e.target.value })}
              placeholder={t('bot.description_placeholder', 'What does your bot do?')}
              rows={2}
              className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none resize-none"
            />

            {/* Handler Type */}
            <select
              value={newBot.handler_type}
              onChange={(e) =>
                setNewBot({ ...newBot, handler_type: e.target.value as BotHandlerType })
              }
              className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
            >
              <option value={BotHandlerType.Internal}>
                {t('bot.type_internal', 'Internal (built-in commands)')}
              </option>
              <option value={BotHandlerType.Webhook}>
                {t('bot.type_webhook', 'Webhook (external server)')}
              </option>
              <option value={BotHandlerType.AI}>
                {t('bot.type_ai', 'AI Assistant (OpenRouter)')}
              </option>
            </select>

            {/* Webhook URL (if webhook type) */}
            {newBot.handler_type === BotHandlerType.Webhook && (
              <input
                value={newBot.webhook_url || ''}
                onChange={(e) => setNewBot({ ...newBot, webhook_url: e.target.value })}
                placeholder="https://your-server.com/webhook"
                className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
              />
            )}

            {/* AI Prompt (if AI type) */}
            {newBot.handler_type === BotHandlerType.AI && (
              <textarea
                value={newBot.ai_prompt || ''}
                onChange={(e) => setNewBot({ ...newBot, ai_prompt: e.target.value })}
                placeholder={t('bot.ai_prompt_placeholder', 'System prompt for the AI...')}
                rows={3}
                className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none resize-none"
              />
            )}

            <div className="flex gap-2">
              <button
                onClick={handleCreate}
                disabled={!newBot.name || !newBot.username}
                className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white rounded-lg transition"
              >
                {t('bot.create', 'Create Bot')}
              </button>
              <button
                onClick={() => setShowCreateForm(false)}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition"
              >
                {t('common.cancel', 'Cancel')}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Bot List */}
      {loading ? (
        <div className="text-center text-gray-500 py-8">{t('common.loading', 'Loading...')}</div>
      ) : myBots.length === 0 ? (
        <div className="text-center text-gray-500 py-12">
          <p className="text-lg mb-2">🤖</p>
          <p>{t('bot.no_bots', 'No bots yet. Create your first bot!')}</p>
        </div>
      ) : (
        <div className="space-y-3">
          {myBots.map((bot) => (
            <BotCard key={bot.id} bot={bot} onDelete={() => deleteBot(bot.id)} />
          ))}
        </div>
      )}
    </div>
  );
};

// ============================================================================
// Bot Card
// ============================================================================

interface BotCardProps {
  bot: Bot;
  onDelete: () => void;
}

const BotCard: React.FC<BotCardProps> = ({ bot, onDelete }) => {
  const { t } = useTranslation();
  const [showConfirm, setShowConfirm] = useState(false);

  const handlerTypeLabel = (type: BotHandlerType) => {
    switch (type) {
      case BotHandlerType.Internal:
        return t('bot.type_internal', 'Internal');
      case BotHandlerType.Webhook:
        return t('bot.type_webhook', 'Webhook');
      case BotHandlerType.AI:
        return t('bot.type_ai', 'AI');
    }
  };

  return (
    <div className="p-4 bg-gray-800 rounded-xl border border-gray-700 hover:border-gray-600 transition">
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-3">
          {bot.avatar_url ? (
            <img src={bot.avatar_url} alt={bot.name} className="w-12 h-12 rounded-full" />
          ) : (
            <div className="w-12 h-12 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white text-xl font-bold">
              {bot.name[0].toUpperCase()}
            </div>
          )}

          <div>
            <h4 className="font-semibold text-white">{bot.name}</h4>
            <p className="text-sm text-gray-400">@{bot.username}</p>
            {bot.description && (
              <p className="text-xs text-gray-500 mt-1">{bot.description}</p>
            )}
            <div className="flex items-center gap-2 mt-2">
              <span className="text-xs px-2 py-0.5 bg-blue-500/20 text-blue-400 rounded-full">
                {handlerTypeLabel(bot.handler_type)}
              </span>
              <span
                className={`text-xs px-2 py-0.5 rounded-full ${
                  bot.is_active
                    ? 'bg-green-500/20 text-green-400'
                    : 'bg-gray-500/20 text-gray-400'
                }`}
              >
                {bot.is_active ? t('bot.active', 'Active') : t('bot.inactive', 'Inactive')}
              </span>
              <span className="text-xs text-gray-500">
                {bot.command_count} {bot.command_count === 1 ? 'command' : 'commands'}
              </span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-1">
          <button
            className="p-2 rounded hover:bg-gray-700 text-gray-400 transition"
            title={t('bot.settings', 'Settings')}
          >
            <FiSettings className="w-4 h-4" />
          </button>

          {!showConfirm ? (
            <button
              onClick={() => setShowConfirm(true)}
              className="p-2 rounded hover:bg-red-500/20 text-gray-400 hover:text-red-400 transition"
              title={t('bot.delete', 'Delete')}
            >
              <FiTrash2 className="w-4 h-4" />
            </button>
          ) : (
            <div className="flex items-center gap-1">
              <button
                onClick={onDelete}
                className="px-2 py-1 text-xs bg-red-600 hover:bg-red-700 text-white rounded transition"
              >
                {t('bot.confirm_delete', 'Confirm')}
              </button>
              <button
                onClick={() => setShowConfirm(false)}
                className="px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 text-white rounded transition"
              >
                {t('common.cancel', 'Cancel')}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
