/**
 * Story Store
 *
 * Manages user stories, story viewing, and story creation.
 */

import { create } from 'zustand';
import {
  Story,
  StoryViewer,
  CreateStoryPayload,
  StoryType,
  StoryPrivacy,
} from '../types/stickers';

interface StoryStore {
  /** Stories from all users (feed) */
  storyFeed: Story[];
  /** Current user's own stories */
  myStories: Story[];
  /** Currently viewing a specific story */
  currentStoryIndex: number;
  currentStoryViewerOpen: boolean;
  storyViewers: StoryViewer[];

  /** Upload/create a new story */
  createStory: (payload: CreateStoryPayload) => Promise<Story>;

  /** Delete own story */
  deleteStory: (storyId: string) => void;

  /** Mark story as viewed */
  markStoryViewed: (storyId: string) => void;

  /** Open story viewer at specific index */
  openStoryViewer: (index: number) => void;

  /** Close story viewer */
  closeStoryViewer: () => void;

  /** Navigate to next/prev story */
  nextStory: () => void;
  prevStory: () => void;

  /** Load story viewers */
  loadStoryViewers: (storyId: string) => Promise<StoryViewer[]>;

  /** Refresh story feed */
  refreshFeed: () => Promise<void>;
}

export const useStoryStore = create<StoryStore>((set, get) => ({
  storyFeed: [],
  myStories: [],
  currentStoryIndex: 0,
  currentStoryViewerOpen: false,
  storyViewers: [],

  createStory: async (payload: CreateStoryPayload): Promise<Story> => {
    // TODO: Upload to server via API
    const now = new Date();
    const durationHours = parseInt(import.meta.env.VITE_STORY_DURATION_HOURS || '24', 10);
    const expiresAt = new Date(now.getTime() + durationHours * 60 * 60 * 1000);

    const story: Story = {
      id: crypto.randomUUID(),
      user_id: localStorage.getItem('user_id') || 'unknown',
      username: localStorage.getItem('username') || 'user',
      type: payload.type,
      content_url: payload.file ? URL.createObjectURL(payload.file) : '',
      caption: payload.caption,
      created_at: now.toISOString(),
      expires_at: expiresAt.toISOString(),
      view_count: 0,
      has_audio: payload.type === StoryType.Video,
      duration_seconds: payload.duration_seconds,
      is_viewed: true,
    };

    set((state) => ({
      myStories: [...state.myStories, story],
      storyFeed: [...state.storyFeed, story],
    }));

    return story;
  },

  deleteStory: (storyId: string) => {
    set((state) => ({
      myStories: state.myStories.filter((s) => s.id !== storyId),
      storyFeed: state.storyFeed.filter((s) => s.id !== storyId),
    }));
  },

  markStoryViewed: (storyId: string) => {
    set((state) => ({
      storyFeed: state.storyFeed.map((s) =>
        s.id === storyId ? { ...s, is_viewed: true, view_count: s.view_count + 1 } : s,
      ),
    }));
  },

  openStoryViewer: (index: number) => {
    set({ currentStoryIndex: index, currentStoryViewerOpen: true });
  },

  closeStoryViewer: () => {
    set({ currentStoryViewerOpen: false, storyViewers: [] });
  },

  nextStory: () => {
    const { currentStoryIndex, storyFeed } = get();
    if (currentStoryIndex < storyFeed.length - 1) {
      set({ currentStoryIndex: currentStoryIndex + 1 });
    } else {
      get().closeStoryViewer();
    }
  },

  prevStory: () => {
    const { currentStoryIndex } = get();
    if (currentStoryIndex > 0) {
      set({ currentStoryIndex: currentStoryIndex - 1 });
    }
  },

  loadStoryViewers: async (storyId: string): Promise<StoryViewer[]> => {
    // TODO: Fetch from API
    const viewers: StoryViewer[] = [
      {
        user_id: 'demo-user-1',
        username: 'Alice',
        viewed_at: new Date().toISOString(),
      },
    ];
    set({ storyViewers: viewers });
    return viewers;
  },

  refreshFeed: async () => {
    // TODO: Fetch stories from API
    // For now, keep existing feed
  },
}));
