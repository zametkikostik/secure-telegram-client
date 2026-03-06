// cloudflare/worker/src/matrix.ts
/**
 * Matrix API для мессенджера
 */

export interface MatrixEvent {
  type: string;
  content: Record<string, any>;
  room_id: string;
  sender: string;
  origin_server_ts: number;
}

export interface MatrixRoom {
  room_id: string;
  name?: string;
  members: string[];
}

export class MatrixClient {
  private baseUrl: string;
  private accessToken: string;

  constructor(baseUrl: string, accessToken: string) {
    this.baseUrl = baseUrl;
    this.accessToken = accessToken;
  }

  async sendMessage(roomId: string, message: string): Promise<string> {
    const response = await fetch(
      `${this.baseUrl}/_matrix/client/v3/rooms/${encodeURIComponent(roomId)}/send/m.room.message`,
      {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          msgtype: 'm.text',
          body: message,
        }),
      }
    );

    const result = await response.json();
    return result.event_id;
  }

  async getRoomMessages(roomId: string, limit: number = 50): Promise<MatrixEvent[]> {
    const response = await fetch(
      `${this.baseUrl}/_matrix/client/v3/rooms/${encodeURIComponent(roomId)}/messages?dir=b&limit=${limit}`,
      {
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
        },
      }
    );

    const result = await response.json();
    return result.chunk;
  }

  async createRoom(name?: string, isPublic: boolean = false): Promise<string> {
    const response = await fetch(
      `${this.baseUrl}/_matrix/client/v3/createRoom`,
      {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name,
          visibility: isPublic ? 'public' : 'private',
        }),
      }
    );

    const result = await response.json();
    return result.room_id;
  }

  async joinRoom(roomId: string): Promise<void> {
    await fetch(
      `${this.baseUrl}/_matrix/client/v3/rooms/${encodeURIComponent(roomId)}/join`,
      {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
        },
      }
    );
  }

  async leaveRoom(roomId: string): Promise<void> {
    await fetch(
      `${this.baseUrl}/_matrix/client/v3/rooms/${encodeURIComponent(roomId)}/leave`,
      {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
        },
      }
    );
  }
}
