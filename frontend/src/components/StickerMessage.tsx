/**
 * StickerMessage — Renders a sticker in chat
 */

import React from 'react';
import { useStickerStore } from '../services/stickerStore';

interface StickerMessageProps {
  stickerId: string;
  senderId: string;
  isOwn: boolean;
}

export const StickerMessage: React.FC<StickerMessageProps> = ({
  stickerId,
  isOwn,
}) => {
  const { getStickerById } = useStickerStore();
  const sticker = getStickerById(stickerId);

  if (!sticker) {
    return (
      <div className={`flex ${isOwn ? 'justify-end' : 'justify-start'}`}>
        <div className="text-gray-500 text-sm italic">Sticker unavailable</div>
      </div>
    );
  }

  return (
    <div className={`flex ${isOwn ? 'justify-end' : 'justify-start'}`}>
      <div className="w-24 h-24 flex items-center justify-center">
        {sticker.url ? (
          <img
            src={sticker.url}
            alt={sticker.emoji}
            className="w-full h-full object-contain"
          />
        ) : (
          <span className="text-6xl">{sticker.emoji}</span>
        )}
      </div>
    </div>
  );
};
