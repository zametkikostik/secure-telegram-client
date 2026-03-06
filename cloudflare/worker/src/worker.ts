// cloudflare/worker/src/worker.ts
/**
 * Liberty Reach Cloudflare Worker
 * Push notifications, Matrix API, WebRTC signaling
 */

import { MatrixClient } from './matrix';
import { WebRTCHandler } from './webrtc';

export interface Env {
  PUSH_STORE: KVNamespace;
  CALL_STORE: KVNamespace;
  MATRIX_URL: string;
  MATRIX_TOKEN: string;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    
    // CORS headers
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    };

    // Handle preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      // Routes
      if (url.pathname === '/') {
        return new Response(JSON.stringify({
          name: 'Liberty Reach Worker',
          version: '1.0.0',
          status: 'ok'
        }), {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
      }

      // Push notifications
      if (url.pathname.startsWith('/push/')) {
        return handlePush(request, env, url);
      }

      // WebRTC signaling
      if (url.pathname.startsWith('/webrtc/')) {
        return handleWebRTC(request, env, url);
      }

      // Matrix proxy
      if (url.pathname.startsWith('/matrix/')) {
        return handleMatrix(request, env, url);
      }

      return new Response('Not Found', { status: 404 });
    } catch (error) {
      return new Response(JSON.stringify({ error: error.message }), {
        status: 500,
        headers: { ...corsHeaders, 'Content-Type': 'application/json' }
      });
    }
  },
};

async function handlePush(request: Request, env: Env, url: URL): Promise<Response> {
  const userId = url.pathname.split('/')[2];
  
  if (request.method === 'POST') {
    const data = await request.json();
    await env.PUSH_STORE.put(`user:${userId}:notifications`, JSON.stringify(data));
    return new Response(JSON.stringify({ success: true }));
  }

  if (request.method === 'GET') {
    const notifications = await env.PUSH_STORE.get(`user:${userId}:notifications`);
    return new Response(notifications || '[]', {
      headers: { 'Content-Type': 'application/json' }
    });
  }

  return new Response('Method not allowed', { status: 405 });
}

async function handleWebRTC(request: Request, env: Env, url: URL): Promise<Response> {
  const handler = new WebRTCHandler(env.CALL_STORE);
  const path = url.pathname.split('/');
  
  if (request.method === 'POST') {
    if (path[2] === 'offer') {
      const { caller, callee, sdp } = await request.json();
      const callId = await handler.createOffer(caller, callee, sdp);
      return new Response(JSON.stringify({ callId }));
    }
    
    if (path[2] === 'answer') {
      const { callId, sdp } = await request.json();
      await handler.createAnswer(callId, sdp);
      return new Response(JSON.stringify({ success: true }));
    }

    if (path[2] === 'ice') {
      const { callId, candidate, from } = await request.json();
      await handler.addIceCandidate(callId, candidate, from);
      return new Response(JSON.stringify({ success: true }));
    }
  }

  if (request.method === 'GET') {
    if (path[2] === 'offer') {
      const callId = url.searchParams.get('callId');
      const offer = await handler.getOffer(callId!);
      return new Response(JSON.stringify(offer));
    }

    if (path[2] === 'answer') {
      const callId = url.searchParams.get('callId');
      const answer = await handler.getAnswer(callId!);
      return new Response(JSON.stringify(answer));
    }

    if (path[2] === 'ice') {
      const callId = url.searchParams.get('callId');
      const forUser = url.searchParams.get('forUser');
      const candidates = await handler.getIceCandidates(callId!, forUser!);
      return new Response(JSON.stringify(candidates));
    }
  }

  return new Response('Not Found', { status: 404 });
}

async function handleMatrix(request: Request, env: Env, url: URL): Promise<Response> {
  const client = new MatrixClient(env.MATRIX_URL, env.MATRIX_TOKEN);
  const path = url.pathname.split('/');
  
  // Proxy to Matrix API
  const matrixPath = url.pathname.replace('/matrix', '');
  const matrixUrl = `${env.MATRIX_URL}${matrixPath}${url.search}`;
  
  const response = await fetch(matrixUrl, {
    method: request.method,
    headers: {
      'Authorization': `Bearer ${env.MATRIX_TOKEN}`,
      'Content-Type': 'application/json',
    },
    body: request.method !== 'GET' && request.method !== 'HEAD' ? request.body : undefined,
  });

  return response;
}
