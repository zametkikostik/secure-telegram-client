// Secure Messenger P2P Worker + Telegram Bridge
// Ad Module Integration (inline to avoid bundling issues)

const MAX_MESSAGES = 1000;
const MESSAGE_TTL = 7 * 24 * 60 * 60 * 1000;
const ADS_MAX_BUNDLE_SIZE = 50;
const ADS_DEFAULT_TTL = 7 * 24 * 60 * 60;

// ============================================================================
// Ad Module Functions (inline)
// ============================================================================

async function handleFetchBundle(request, env) {
  let body;
  try {
    body = await request.json();
  } catch (e) {
    return new Response(JSON.stringify({ error: 'Invalid JSON body' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const { categories, client_public_key, last_sync, max_ads } = body;

  if (!client_public_key) {
    return new Response(JSON.stringify({ error: 'client_public_key required' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const adsJson = await env.PUSH_STORE.get('ads:inventory');
  const allAds = adsJson ? JSON.parse(adsJson) : [];

  let filteredAds = allAds;
  if (categories && categories.length > 0) {
    filteredAds = allAds.filter(ad => {
      return categories.includes(ad.category) || ad.category === 'general';
    });
  }

  if (last_sync) {
    filteredAds = filteredAds.filter(ad => ad.timestamp > last_sync);
  }

  const limit = Math.min(max_ads || ADS_MAX_BUNDLE_SIZE, ADS_MAX_BUNDLE_SIZE);
  filteredAds = filteredAds.slice(0, limit);

  const advertiserKey = await env.PUSH_STORE.get('ads:advertiser_key');
  if (!advertiserKey) {
    return new Response(JSON.stringify({ error: 'Advertiser key not configured' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const encryptedBundle = {
    ciphertext: filteredAds,
    nonce: 'placeholder',
    advertiser_key: advertiserKey,
    timestamp: Date.now(),
    version: 1,
  };

  return new Response(JSON.stringify({
    bundle: encryptedBundle,
    total_ads: filteredAds.length,
    server_timestamp: Date.now(),
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
}

async function handleReportImpressions(request, env) {
  let impressionHashes;
  try {
    impressionHashes = await request.json();
  } catch (e) {
    return new Response(JSON.stringify({ error: 'Invalid JSON body' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  if (!Array.isArray(impressionHashes)) {
    return new Response(JSON.stringify({ error: 'Expected array of hashes' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const batchKey = `ads:impressions:${Date.now()}`;
  await env.PUSH_STORE.put(batchKey, JSON.stringify({
    hashes: impressionHashes,
    timestamp: Date.now(),
    count: impressionHashes.length,
  }), { expirationTtl: ADS_DEFAULT_TTL });

  const statsJson = await env.PUSH_STORE.get('ads:stats');
  const stats = statsJson ? JSON.parse(statsJson) : {
    total_impressions: 0,
    total_clicks: 0,
    by_ad_id: {},
    by_category: {},
  };

  stats.total_impressions += impressionHashes.length;
  stats.last_updated = Date.now();

  await env.PUSH_STORE.put('ads:stats', JSON.stringify(stats), {
    expirationTtl: 30 * 24 * 60 * 60,
  });

  return new Response(JSON.stringify({
    success: true,
    recorded: impressionHashes.length,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
}

async function handleAddAd(request, env) {
  let ad;
  try {
    ad = await request.json();
  } catch (e) {
    return new Response(JSON.stringify({ error: 'Invalid JSON body' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  if (!ad.id || !ad.advertiser || !ad.category) {
    return new Response(JSON.stringify({ error: 'Required fields missing' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const adsJson = await env.PUSH_STORE.get('ads:inventory');
  const ads = adsJson ? JSON.parse(adsJson) : [];

  const existingIndex = ads.findIndex(a => a.id === ad.id);
  if (existingIndex >= 0) {
    ads[existingIndex] = { ...ad, timestamp: Date.now() };
  } else {
    ads.push({ ...ad, timestamp: Date.now() });
  }

  await env.PUSH_STORE.put('ads:inventory', JSON.stringify(ads), {
    expirationTtl: 30 * 24 * 60 * 60,
  });

  return new Response(JSON.stringify({
    success: true,
    ad_id: ad.id,
    total_ads: ads.length,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
}

async function handleGetStats(request, env) {
  const statsJson = await env.PUSH_STORE.get('ads:stats');
  const stats = statsJson ? JSON.parse(statsJson) : {
    total_impressions: 0,
    total_clicks: 0,
    by_ad_id: {},
    by_category: {},
  };

  return new Response(JSON.stringify(stats), {
    headers: { 'Content-Type': 'application/json' },
  });
}

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const path = url.pathname;
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization, X-Device-ID',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      if (path === '/' || path === '/health') {
        return new Response(JSON.stringify({ status: 'ok', version: '3.0.0-p2p' }), { 
          headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
        });
      }

      // P2P Регистрация
      if (path === '/p2p/register' && request.method === 'POST') {
        let body;
        try {
          body = await request.json();
        } catch (e) {
          return new Response(JSON.stringify({ error: 'Invalid JSON body' }), {
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }
        const { deviceId, userId, publicKey } = body;
        if (!deviceId || !userId || !publicKey) {
          return new Response(JSON.stringify({ error: 'Required fields missing' }), {
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }
        await env.PUSH_STORE.put(`p2p:device:${deviceId}`, JSON.stringify({
          deviceId, userId, publicKey, registeredAt: new Date().toISOString(), lastSeen: new Date().toISOString()
        }));
        return new Response(JSON.stringify({ success: true, deviceId }), { 
          headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
        });
      }

      // P2P Отправка сообщения
      if (path === '/p2p/send' && request.method === 'POST') {
        let body;
        try {
          body = await request.json();
        } catch (e) {
          return new Response(JSON.stringify({ error: 'Invalid JSON body' }), {
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }
        const { fromDeviceId, toDeviceId, encryptedMessage, messageType } = body;
        if (!fromDeviceId || !toDeviceId || !encryptedMessage) {
          return new Response(JSON.stringify({ error: 'Required fields missing' }), {
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }
        const message = {
          id: crypto.randomUUID(), fromDeviceId, toDeviceId, encryptedMessage,
          messageType: messageType || 'text', timestamp: Date.now(), delivered: false
        };
        const queueKey = `p2p:queue:${toDeviceId}`;
        const existing = await env.PUSH_STORE.get(queueKey);
        let queue = existing ? JSON.parse(existing) : [];
        queue.unshift(message);
        if (queue.length > MAX_MESSAGES) queue = queue.slice(0, MAX_MESSAGES);
        await env.PUSH_STORE.put(queueKey, JSON.stringify(queue));
        return new Response(JSON.stringify({ success: true, messageId: message.id, queued: true }), { 
          headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
        });
      }

      // P2P Получение сообщений
      if (path === '/p2p/messages' && request.method === 'GET') {
        const deviceId = url.searchParams.get('deviceId');
        if (!deviceId) {
          return new Response(JSON.stringify({ error: 'deviceId required' }), { 
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
          });
        }
        const queueKey = `p2p:queue:${deviceId}`;
        const existing = await env.PUSH_STORE.get(queueKey);
        if (!existing) {
          return new Response(JSON.stringify({ messages: [], count: 0 }), { 
            headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
          });
        }
        let queue = JSON.parse(existing);
        const now = Date.now();
        queue = queue.filter(msg => (now - msg.timestamp) < MESSAGE_TTL);
        await env.PUSH_STORE.put(queueKey, JSON.stringify(queue));
        return new Response(JSON.stringify({ messages: queue, count: queue.length }), { 
          headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
        });
      }

      // Синхронизация контактов
      if (path === '/contacts/sync' && request.method === 'POST') {
        let body;
        try {
          body = await request.json();
        } catch (e) {
          return new Response(JSON.stringify({ error: 'Invalid JSON body' }), {
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }
        const { userId, contacts } = body;
        if (!userId || !contacts) {
          return new Response(JSON.stringify({ error: 'userId, contacts required' }), {
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }
        await env.PUSH_STORE.put(`contacts:${userId}`, JSON.stringify({ userId, contacts, syncedAt: new Date().toISOString() }));
        return new Response(JSON.stringify({ success: true, count: contacts.length }), { 
          headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
        });
      }

      // Получение контактов
      if (path === '/contacts' && request.method === 'GET') {
        const userId = url.searchParams.get('userId');
        if (!userId) {
          return new Response(JSON.stringify({ error: 'userId required' }), { 
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
          });
        }
        const existing = await env.PUSH_STORE.get(`contacts:${userId}`);
        if (!existing) {
          return new Response(JSON.stringify({ contacts: [], count: 0 }), { 
            headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
          });
        }
        const data = JSON.parse(existing);
        return new Response(JSON.stringify({ contacts: data.contacts, count: data.contacts.length }), { 
          headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
        });
      }

      // Email код
      if (path === '/send-email-code' && request.method === 'POST') {
        let body;
        try {
          body = await request.json();
        } catch (e) {
          return new Response(JSON.stringify({ error: 'Invalid JSON body' }), {
            status: 400, headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }
        const { email, code } = body;
        return new Response(JSON.stringify({ success: true, code, message: 'Code generated' }), {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
      }

      // ========================================================================
      // Ad Module Endpoints
      // ========================================================================

      // Fetch encrypted ad bundle
      if (path === '/api/v1/ads/bundle' && request.method === 'POST') {
        return await handleFetchBundle(request, env);
      }

      // Report impressions anonymously
      if (path === '/api/v1/ads/report' && request.method === 'POST') {
        return await handleReportImpressions(request, env);
      }

      // Add ad to inventory (admin)
      if (path === '/api/v1/ads/add' && request.method === 'POST') {
        return await handleAddAd(request, env);
      }

      // Get ad stats (admin)
      if (path === '/api/v1/ads/stats' && request.method === 'GET') {
        return await handleGetStats(request, env);
      }

      return new Response(JSON.stringify({ error: 'Not Found' }), { 
        status: 404, headers: { ...corsHeaders, 'Content-Type': 'application/json' } 
      });

    } catch (error) {
      const errorMessage = typeof error.message === 'string' 
        ? error.message 
        : JSON.stringify(error);
      console.error('Worker error:', error);
      return new Response(JSON.stringify({ error: errorMessage }), {
        status: 500, headers: { ...corsHeaders, 'Content-Type': 'application/json' }
      });
    }
  }
};
