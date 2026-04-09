/**
 * Bot Platform Types
 */

// ============================================================================
// Bot Core Types
// ============================================================================

export interface Bot {
  id: string;
  name: string;
  username: string;  // @bot_username format
  description?: string;
  avatar_url?: string;
  owner_id: string;
  is_active: boolean;
  handler_type: BotHandlerType;
  webhook_url?: string;
  command_count: number;
  created_at: string;
  /** Bot token (only shown once after creation) */
  token?: string;
}

export enum BotHandlerType {
  Internal = 'internal',
  Webhook = 'webhook',
  AI = 'ai',
}

export interface BotCommand {
  id: string;
  bot_id: string;
  command: string;       // e.g. "/start"
  description?: string;
  handler_type: BotHandlerType;
  handler_url?: string;
  response_template?: string;  // For simple internal bots
}

export interface BotWebhook {
  id: string;
  bot_id: string;
  url: string;
  events: BotEventType[];
  secret: string;
  active: boolean;
  last_triggered_at?: string;
  last_status?: number;
}

export type BotEventType =
  | 'message'
  | 'command'
  | 'join'
  | 'leave'
  | 'edit'
  | 'delete';

// ============================================================================
// Bot Session / FSM
// ============================================================================

export interface BotSession {
  bot_id: string;
  user_id: string;
  chat_id: string;
  state: string;          // FSM state identifier
  context: Record<string, unknown>;  // Arbitrary context data
  updated_at: string;
}

// ============================================================================
// Bot Event (what gets dispatched to bots)
// ============================================================================

export interface BotEvent {
  event_type: BotEventType;
  bot_id: string;
  user_id: string;
  chat_id: string;
  message?: {
    id: string;
    content: string;
    is_command: boolean;
    command?: string;     // e.g. "/start"
    args?: string;        // e.g. "arg1 arg2"
  };
  timestamp: string;
}

// ============================================================================
// Bot API Response (for external webhooks)
// ============================================================================

export interface BotApiResponse {
  /** Message to send back */
  text?: string;
  /** Sticker ID to send */
  sticker_id?: string;
  /** Keyboard buttons (inline) */
  inline_keyboard?: InlineKeyboardButton[][];
  /** Delete the triggering message */
  delete_message?: boolean;
  /** Edit the triggering message */
  edit_message?: string;
}

export interface InlineKeyboardButton {
  text: string;
  callback_data?: string;
  url?: string;
  switch_inline_query?: string;
}

// ============================================================================
// Bot Store (Marketplace)
// ============================================================================

export interface BotStoreListing {
  id: string;
  name: string;
  username: string;
  description: string;
  avatar_url?: string;
  category: BotCategory;
  rating: number;
  install_count: number;
  is_verified: boolean;
  is_premium: boolean;
  author: string;
  commands: string[];  // Sample commands
}

export enum BotCategory {
  Utility = 'utility',
  Entertainment = 'entertainment',
  AI = 'ai',
  Moderation = 'moderation',
  Integration = 'integration',
  Games = 'games',
}

// ============================================================================
// CRUD Payloads
// ============================================================================

export interface CreateBotPayload {
  name: string;
  username: string;
  description?: string;
  handler_type: BotHandlerType;
  webhook_url?: string;
  ai_prompt?: string;  // For AI bots
}

export interface UpdateBotPayload {
  name?: string;
  description?: string;
  avatar_url?: string;
  webhook_url?: string;
  is_active?: boolean;
}

export interface CreateCommandPayload {
  command: string;
  description?: string;
  handler_type: BotHandlerType;
  handler_url?: string;
  response_template?: string;
}

export interface CreateWebhookPayload {
  url: string;
  events: BotEventType[];
  secret?: string;
}
