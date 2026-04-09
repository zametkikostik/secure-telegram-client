/**
 * StoryBar — Horizontal story feed at top of chat list
 */

import React from 'react';
import { useStoryStore } from '../services/storyStore';
import { Story } from '../types/stickers';
import { FiPlus } from 'react-icons/fi';

interface StoryBarProps {
  onOpenStory: (index: number) => void;
  onCreateStory: () => void;
}

export const StoryBar: React.FC<StoryBarProps> = ({
  onOpenStory,
  onCreateStory,
}) => {
  const { storyFeed, openStoryViewer } = useStoryStore();

  // Group stories by user, take latest from each
  const uniqueUserStories = storyFeed.reduce<Map<string, Story>>((map, story) => {
    const existing = map.get(story.user_id);
    if (!existing || new Date(story.created_at) > new Date(existing.created_at)) {
      map.set(story.user_id, story);
    }
    return map;
  }, new Map());

  const stories = Array.from(uniqueUserStories.values());

  if (!stories.length) {
    return null;
  }

  return (
    <div className="border-b border-gray-700 p-3 overflow-x-auto">
      <div className="flex gap-3">
        {/* Create Story Button */}
        <button
          onClick={onCreateStory}
          className="flex flex-col items-center gap-1 min-w-[64px]"
        >
          <div className="w-16 h-16 rounded-full border-2 border-dashed border-gray-500 flex items-center justify-center hover:border-blue-500 transition-colors">
            <FiPlus className="w-6 h-6 text-gray-500" />
          </div>
          <span className="text-xs text-gray-500">Your Story</span>
        </button>

        {/* User Stories */}
        {stories.map((story, index) => (
          <button
            key={story.user_id}
            onClick={() => {
              const feedIndex = storyFeed.findIndex((s) => s.user_id === story.user_id);
              onOpenStory(feedIndex >= 0 ? feedIndex : index);
            }}
            className="flex flex-col items-center gap-1 min-w-[64px]"
          >
            <div
              className={`w-16 h-16 rounded-full p-0.5 ${
                story.is_viewed
                  ? 'bg-gray-600'
                  : 'bg-gradient-to-br from-blue-500 to-purple-600'
              }`}
            >
              <div className="w-full h-full rounded-full bg-gray-800 flex items-center justify-center overflow-hidden">
                {story.avatar_url ? (
                  <img
                    src={story.avatar_url}
                    alt={story.username}
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <span className="text-2xl text-white font-bold">
                    {story.username[0].toUpperCase()}
                  </span>
                )}
              </div>
            </div>
            <span className="text-xs text-gray-400 truncate max-w-[64px]">
              {story.username}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
};
