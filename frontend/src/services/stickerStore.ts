/**
 * Sticker & Reaction Store
 *
 * Manages installed sticker packs, message reactions, and sticker selection.
 */

import { create } from 'zustand';
import { StickerPack, Sticker, Reaction } from '../types/stickers';

interface ReactionStore {
  /** Map of messageId -> reactions[] */
  reactions: Record<string, Reaction[]>;
  addReaction: (messageId: string, emoji: string, userId: string) => void;
  removeReaction: (messageId: string, emoji: string, userId: string) => void;
  getReactions: (messageId: string) => Reaction[];
  hasUserReacted: (messageId: string, userId: string, emoji: string) => boolean;
}

interface StickerStore {
  /** Installed sticker packs */
  installedPacks: StickerPack[];
  /** Available packs from store (not yet installed) */
  availablePacks: StickerPack[];
  /** Currently selected sticker for sending */
  selectedSticker: Sticker | null;
  /** Whether sticker picker is open */
  isPickerOpen: boolean;

  installPack: (pack: StickerPack) => void;
  uninstallPack: (packId: string) => void;
  selectSticker: (sticker: Sticker | null) => void;
  togglePicker: () => void;
  closePicker: () => void;
  getStickerById: (stickerId: string) => Sticker | undefined;
}

export const useReactionStore = create<ReactionStore>((set, get) => ({
  reactions: {},

  addReaction: (messageId: string, emoji: string, userId: string) => {
    set((state) => {
      const messageReactions = state.reactions[messageId] || [];
      const existingIdx = messageReactions.findIndex((r) => r.emoji === emoji);

      let updated: Reaction[];
      if (existingIdx >= 0) {
        // Update existing reaction
        const existing = messageReactions[existingIdx];
        updated = [...messageReactions];
        updated[existingIdx] = {
          ...existing,
          count: existing.is_selected ? existing.count - 1 : existing.count + 1,
          is_selected: !existing.is_selected,
          user_ids: existing.is_selected
            ? existing.user_ids.filter((id) => id !== userId)
            : [...existing.user_ids, userId],
        };
      } else {
        // Add new reaction
        updated = [
          ...messageReactions,
          { emoji, count: 1, is_selected: true, user_ids: [userId] },
        ];
      }

      return {
        reactions: {
          ...state.reactions,
          [messageId]: updated,
        },
      };
    });
  },

  removeReaction: (messageId: string, emoji: string, userId: string) => {
    set((state) => {
      const messageReactions = state.reactions[messageId] || [];
      const updated = messageReactions
        .map((r) =>
          r.emoji === emoji
            ? {
                ...r,
                count: r.count - 1,
                is_selected: false,
                user_ids: r.user_ids.filter((id) => id !== userId),
              }
            : r,
        )
        .filter((r) => r.count > 0);

      return {
        reactions: {
          ...state.reactions,
          [messageId]: updated,
        },
      };
    });
  },

  getReactions: (messageId: string) => {
    return get().reactions[messageId] || [];
  },

  hasUserReacted: (messageId: string, userId: string, emoji: string) => {
    const reactions = get().reactions[messageId] || [];
    const reaction = reactions.find((r) => r.emoji === emoji);
    return reaction?.is_selected && reaction.user_ids.includes(userId);
  },
}));

export const useStickerStore = create<StickerStore>((set, get) => ({
  installedPacks: [
    // Default built-in pack
    {
      id: 'default-emoji',
      name: 'Emoji',
      emoji: '😀',
      cover_sticker_id: 'emoji-1',
      is_installed: true,
      stickers: [
        { id: 'emoji-1', pack_id: 'default-emoji', emoji: '😀', url: '', width: 64, height: 64, file_size: 0 },
        { id: 'emoji-2', pack_id: 'default-emoji', emoji: '😂', url: '', width: 64, height: 64, file_size: 0 },
        { id: 'emoji-3', pack_id: 'default-emoji', emoji: '❤️', url: '', width: 64, height: 64, file_size: 0 },
        { id: 'emoji-4', pack_id: 'default-emoji', emoji: '🔥', url: '', width: 64, height: 64, file_size: 0 },
        { id: 'emoji-5', pack_id: 'default-emoji', emoji: '👍', url: '', width: 64, height: 64, file_size: 0 },
        { id: 'emoji-6', pack_id: 'default-emoji', emoji: '😮', url: '', width: 64, height: 64, file_size: 0 },
        { id: 'emoji-7', pack_id: 'default-emoji', emoji: '😢', url: '', width: 64, height: 64, file_size: 0 },
        { id: 'emoji-8', pack_id: 'default-emoji', emoji: '🎉', url: '', width: 64, height: 64, file_size: 0 },
      ],
    },
  ],
  availablePacks: [],
  selectedSticker: null,
  isPickerOpen: false,

  installPack: (pack: StickerPack) => {
    set((state) => ({
      installedPacks: [...state.installedPacks, { ...pack, is_installed: true }],
      availablePacks: state.availablePacks.filter((p) => p.id !== pack.id),
    }));
  },

  uninstallPack: (packId: string) => {
    set((state) => {
      const pack = state.installedPacks.find((p) => p.id === packId);
      if (!pack) return state;

      return {
        installedPacks: state.installedPacks.filter((p) => p.id !== packId),
        availablePacks: [...state.availablePacks, { ...pack, is_installed: false }],
      };
    });
  },

  selectSticker: (sticker: Sticker | null) => {
    set({ selectedSticker: sticker });
  },

  togglePicker: () => {
    set((state) => ({ isPickerOpen: !state.isPickerOpen }));
  },

  closePicker: () => {
    set({ isPickerOpen: false, selectedSticker: null });
  },

  getStickerById: (stickerId: string) => {
    const { installedPacks } = get();
    for (const pack of installedPacks) {
      const sticker = pack.stickers.find((s) => s.id === stickerId);
      if (sticker) return sticker;
    }
    return undefined;
  },
}));
