/**
 * ReactionPicker — Popup for selecting emoji reactions on messages
 */

import React, { useRef, useEffect } from 'react';
import { useReactionStore } from '../services/stickerStore';
import { DEFAULT_REACTIONS } from '../types/stickers';

interface ReactionPickerProps {
  messageId: string;
  onClose: () => void;
  position?: { top: number; left: number };
}

export const ReactionPicker: React.FC<ReactionPickerProps> = ({
  messageId,
  onClose,
  position,
}) => {
  const pickerRef = useRef<HTMLDivElement>(null);
  const { addReaction, removeReaction, getReactions } = useReactionStore();

  const currentReactions = getReactions(messageId);
  const userId = localStorage.getItem('user_id') || 'unknown';

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (pickerRef.current && !pickerRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [onClose]);

  const handleReaction = (emoji: string) => {
    const hasReacted = currentReactions.find(
      (r) => r.emoji === emoji && r.is_selected,
    );

    if (hasReacted) {
      removeReaction(messageId, emoji, userId);
    } else {
      addReaction(messageId, emoji, userId);
    }
    onClose();
  };

  return (
    <div
      ref={pickerRef}
      className="absolute z-50 bg-gray-800 rounded-xl shadow-2xl border border-gray-700 p-2 animate-slide-up"
      style={
        position
          ? { top: position.top, left: position.left }
          : { bottom: '100%', right: 0 }
      }
    >
      <div className="flex items-center gap-1">
        {DEFAULT_REACTIONS.map((emoji) => {
          const isActive = currentReactions.some(
            (r) => r.emoji === emoji && r.is_selected,
          );

          return (
            <button
              key={emoji}
              onClick={() => handleReaction(emoji)}
              className={`p-2 rounded-lg text-xl transition-all hover:scale-125 ${
                isActive
                  ? 'bg-blue-500/30 ring-2 ring-blue-500'
                  : 'hover:bg-gray-700'
              }`}
              aria-label={`React with ${emoji}`}
            >
              {emoji}
            </button>
          );
        })}
      </div>
    </div>
  );
};
