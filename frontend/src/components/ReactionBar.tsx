/**
 * ReactionBar — Displays reactions below a message
 */

import React, { useState, useRef } from 'react';
import { useReactionStore } from '../services/stickerStore';
import { Reaction } from '../types/stickers';
import { ReactionPicker } from './ReactionPicker';

interface ReactionBarProps {
  messageId: string;
  reactions?: Reaction[];
}

export const ReactionBar: React.FC<ReactionBarProps> = ({
  messageId,
  reactions = [],
}) => {
  const { addReaction, removeReaction, getReactions } = useReactionStore();
  const [showPicker, setShowPicker] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);

  const messageReactions = reactions.length > 0 ? reactions : getReactions(messageId);
  const userId = localStorage.getItem('user_id') || 'unknown';

  if (messageReactions.length === 0 && !showPicker) {
    return null;
  }

  const handleTogglePicker = () => {
    setShowPicker(!showPicker);
  };

  return (
    <div className="flex items-center gap-1 flex-wrap mt-1">
      {/* Existing Reactions */}
      {messageReactions.map((reaction) => (
        <button
          key={reaction.emoji}
          onClick={() => {
            if (reaction.is_selected) {
              removeReaction(messageId, reaction.emoji, userId);
            } else {
              addReaction(messageId, reaction.emoji, userId);
            }
          }}
          className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs transition-colors ${
            reaction.is_selected
              ? 'bg-blue-500/30 text-blue-300 ring-1 ring-blue-500/50'
              : 'bg-gray-700/50 text-gray-400 hover:bg-gray-700'
          }`}
        >
          <span className="text-sm">{reaction.emoji}</span>
          <span>{reaction.count}</span>
        </button>
      ))}

      {/* Add Reaction Button */}
      <button
        ref={buttonRef}
        onClick={handleTogglePicker}
        className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-gray-700/30 text-gray-500 hover:bg-gray-700 hover:text-gray-300 transition-colors text-xs"
        aria-label="Add reaction"
      >
        +
      </button>

      {/* Reaction Picker Popup */}
      {showPicker && (
        <ReactionPicker
          messageId={messageId}
          onClose={() => setShowPicker(false)}
        />
      )}
    </div>
  );
};
