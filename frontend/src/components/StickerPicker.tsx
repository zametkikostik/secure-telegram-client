/**
 * StickerPicker — Panel for browsing and selecting stickers
 */

import React, { useState } from 'react';
import { useStickerStore } from '../services/stickerStore';
import { StickerPack } from '../types/stickers';
import { FiX } from 'react-icons/fi';

interface StickerPickerProps {
  onSelectSticker: (stickerId: string) => void;
  onClose: () => void;
}

export const StickerPicker: React.FC<StickerPickerProps> = ({
  onSelectSticker,
  onClose,
}) => {
  const { installedPacks, selectSticker, closePicker } = useStickerStore();
  const [selectedPack, setSelectedPack] = useState<StickerPack | null>(null);

  const handleStickerClick = (stickerId: string) => {
    selectSticker(null);
    onSelectSticker(stickerId);
    closePicker();
  };

  return (
    <div className="absolute bottom-full left-0 right-0 z-40 bg-gray-800 border-t border-gray-700 shadow-2xl">
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b border-gray-700">
        <h3 className="font-semibold text-white">Stickers</h3>
        <button
          onClick={onClose}
          className="p-1 rounded hover:bg-gray-700 text-gray-400"
          aria-label="Close sticker picker"
        >
          <FiX className="w-5 h-5" />
        </button>
      </div>

      <div className="flex h-80">
        {/* Pack List (Left Sidebar) */}
        <div className="w-16 border-r border-gray-700 overflow-y-auto">
          {installedPacks.map((pack) => (
            <button
              key={pack.id}
              onClick={() => setSelectedPack(pack)}
              className={`w-full p-2 text-2xl transition-colors hover:bg-gray-700 ${
                selectedPack?.id === pack.id ? 'bg-gray-700' : ''
              }`}
              title={pack.name}
            >
              {pack.emoji}
            </button>
          ))}
        </div>

        {/* Sticker Grid (Main Area) */}
        <div className="flex-1 overflow-y-auto p-3">
          {selectedPack ? (
            <div className="grid grid-cols-4 gap-2">
              {selectedPack.stickers.map((sticker) => (
                <button
                  key={sticker.id}
                  onClick={() => handleStickerClick(sticker.id)}
                  className="aspect-square flex items-center justify-center text-4xl hover:bg-gray-700 rounded-lg transition-colors"
                  title={sticker.emoji}
                >
                  {sticker.url ? (
                    <img
                      src={sticker.url}
                      alt={sticker.emoji}
                      className="w-full h-full object-contain"
                      loading="lazy"
                    />
                  ) : (
                    <span>{sticker.emoji}</span>
                  )}
                </button>
              ))}
            </div>
          ) : (
            <div className="flex items-center justify-center h-full text-gray-500 text-sm">
              Select a sticker pack
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
