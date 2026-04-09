/**
 * StoryViewer — Full-screen story viewer with navigation
 */

import React, { useEffect, useRef, useState } from 'react';
import { useStoryStore } from '../services/storyStore';
import { Story } from '../types/stickers';
import { FiX, FiChevronLeft, FiChevronRight } from 'react-icons/fi';

export const StoryViewer: React.FC = () => {
  const {
    storyFeed,
    currentStoryIndex,
    currentStoryViewerOpen,
    closeStoryViewer,
    nextStory,
    prevStory,
    markStoryViewed,
  } = useStoryStore();

  const [progress, setProgress] = useState(0);
  const [isPaused, setIsPaused] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const progressInterval = useRef<ReturnType<typeof setInterval> | null>(null);

  const story: Story | undefined = storyFeed[currentStoryIndex];

  // Auto-advance progress
  useEffect(() => {
    if (!story || !currentStoryViewerOpen || isPaused) return;

    setProgress(0);
    const duration = story.duration_seconds
      ? story.duration_seconds * 1000
      : 5000; // Default 5s for photos
    const interval = 50;
    const step = (interval / duration) * 100;

    progressInterval.current = setInterval(() => {
      setProgress((prev) => {
        if (prev >= 100) {
          nextStory();
          return 0;
        }
        return prev + step;
      });
    }, interval);

    return () => {
      if (progressInterval.current) {
        clearInterval(progressInterval.current);
      }
    };
  }, [story, currentStoryViewerOpen, isPaused, nextStory]);

  // Mark as viewed
  useEffect(() => {
    if (story) {
      markStoryViewed(story.id);
    }
  }, [story?.id, markStoryViewed]);

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!currentStoryViewerOpen) return;

      switch (e.key) {
        case 'ArrowRight':
          nextStory();
          break;
        case 'ArrowLeft':
          prevStory();
          break;
        case 'Escape':
          closeStoryViewer();
          break;
        case ' ':
          setIsPaused((p) => !p);
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [currentStoryViewerOpen, closeStoryViewer, nextStory, prevStory]);

  if (!currentStoryViewerOpen || !story) return null;

  const isVideo = story.type === 'video';

  return (
    <div
      className="fixed inset-0 z-50 bg-black flex items-center justify-center"
      onMouseDown={() => setIsPaused(true)}
      onMouseUp={() => setIsPaused(false)}
    >
      {/* Progress Bar */}
      <div className="absolute top-0 left-0 right-0 z-10 p-2">
        <div className="h-1 bg-gray-700 rounded-full overflow-hidden">
          <div
            className="h-full bg-white transition-all duration-100"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>

      {/* Story Header */}
      <div className="absolute top-4 left-0 right-0 z-10 flex items-center justify-between px-4">
        <div className="flex items-center gap-3">
          {story.avatar_url ? (
            <img
              src={story.avatar_url}
              alt={story.username}
              className="w-10 h-10 rounded-full"
            />
          ) : (
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white font-bold">
              {story.username[0].toUpperCase()}
            </div>
          )}
          <div>
            <p className="text-white font-semibold">{story.username}</p>
            <p className="text-gray-400 text-xs">
              {new Date(story.created_at).toLocaleTimeString([], {
                hour: '2-digit',
                minute: '2-digit',
              })}
            </p>
          </div>
        </div>

        <button
          onClick={closeStoryViewer}
          className="p-2 rounded-full bg-black/30 hover:bg-black/50 text-white"
          aria-label="Close story"
        >
          <FiX className="w-5 h-5" />
        </button>
      </div>

      {/* Story Content */}
      <div className="w-full h-full flex items-center justify-center">
        {isVideo ? (
          <video
            ref={videoRef}
            src={story.content_url}
            autoPlay
            muted
            className="max-w-full max-h-full object-contain"
          />
        ) : (
          <img
            src={story.content_url}
            alt={story.caption || 'Story'}
            className="max-w-full max-h-full object-contain"
          />
        )}

        {/* Caption Overlay */}
        {story.caption && (
          <div className="absolute bottom-20 left-0 right-0 text-center">
            <p className="text-white text-lg font-medium bg-black/40 inline-block px-6 py-2 rounded-full">
              {story.caption}
            </p>
          </div>
        )}
      </div>

      {/* Navigation Areas */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          prevStory();
        }}
        className="absolute left-4 top-1/2 -translate-y-1/2 p-3 rounded-full bg-black/30 hover:bg-black/50 text-white opacity-50 hover:opacity-100 transition-opacity"
        aria-label="Previous story"
      >
        <FiChevronLeft className="w-6 h-6" />
      </button>

      <button
        onClick={(e) => {
          e.stopPropagation();
          nextStory();
        }}
        className="absolute right-4 top-1/2 -translate-y-1/2 p-3 rounded-full bg-black/30 hover:bg-black/50 text-white opacity-50 hover:opacity-100 transition-opacity"
        aria-label="Next story"
      >
        <FiChevronRight className="w-6 h-6" />
      </button>

      {/* View Count */}
      {import.meta.env.VITE_STORY_VIEWER_COUNT_ENABLED !== 'false' && (
        <div className="absolute bottom-4 right-4 text-gray-400 text-xs">
          {story.view_count} {story.view_count === 1 ? 'view' : 'views'}
        </div>
      )}
    </div>
  );
};
