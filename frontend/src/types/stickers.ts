/**
 * Stickers, Reactions & Stories Types
 */

// ============================================================================
// Stickers
// ============================================================================

export interface StickerPack {
  id: string;
  name: string;
  emoji: string;
  author?: string;
  cover_sticker_id: string;
  stickers: Sticker[];
  is_installed: boolean;
  is_premium?: boolean;
}

export interface Sticker {
  id: string;
  pack_id: string;
  emoji: string;
  /** Relative URL path to sticker image (WebP) */
  url: string;
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** File size in bytes */
  file_size: number;
}

// ============================================================================
// Reactions
// ============================================================================

export interface Reaction {
  emoji: string;
  count: number;
  is_selected: boolean;
  /** Users who reacted (limited to first few for display) */
  user_ids: string[];
}

export interface ReactionEvent {
  message_id: string;
  chat_id: string;
  user_id: string;
  emoji: string;
  timestamp: string;
}

export const DEFAULT_REACTIONS = ['👍', '👎', '❤️', '🔥', '😂', '😮', '😢', '🎉'];

// ============================================================================
// Stories
// ============================================================================

export enum StoryType {
  Photo = 'photo',
  Video = 'video',
  Text = 'text',
}

export enum StoryPrivacy {
  Everyone = 'everyone',
  Contacts = 'contacts',
  CloseFriends = 'close_friends',
  SelectedContacts = 'selected_contacts',
}

export interface Story {
  id: string;
  user_id: string;
  username: string;
  avatar_url?: string;
  type: StoryType;
  content_url: string;
  caption?: string;
  created_at: string;
  expires_at: string;
  view_count: number;
  has_audio?: boolean;
  duration_seconds?: number;
  is_viewed: boolean;
}

export interface StoryViewer {
  user_id: string;
  username: string;
  avatar_url?: string;
  viewed_at: string;
}

export interface CreateStoryPayload {
  type: StoryType;
  file?: File;
  caption?: string;
  privacy: StoryPrivacy;
  selected_user_ids?: string[];
  duration_seconds?: number;
}

// ============================================================================
// Combined Media Types
// ============================================================================

export interface MediaAttachment {
  type: 'sticker' | 'photo' | 'video' | 'file';
  id: string;
  url: string;
  width?: number;
  height?: number;
  file_size?: number;
  mime_type?: string;
}
