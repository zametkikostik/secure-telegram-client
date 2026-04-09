// Ad Module Endpoints for Secure Messenger
// Handles encrypted ad bundle delivery and anonymous impression reporting

const ADS_MAX_BUNDLE_SIZE = 50;
const ADS_DEFAULT_TTL = 7 * 24 * 60 * 60; // 7 days

/**
 * Handle ad bundle fetch request
 * POST /api/v1/ads/bundle
 * 
 * Returns encrypted ad bundle filtered by user categories
 * Categories are used for server-side filtering ONLY
 * Actual ad selection happens ON-DEVICE
 */
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

  // Get available ads from KV
  const adsJson = await env.PUSH_STORE.get('ads:inventory');
  const allAds = adsJson ? JSON.parse(adsJson) : [];

  // Filter ads by categories (if provided)
  let filteredAds = allAds;
  if (categories && categories.length > 0) {
    filteredAds = allAds.filter(ad => {
      // Include ads that match user categories OR are general (no category)
      return categories.includes(ad.category) || ad.category === 'general';
    });
  }

  // Filter by last_sync (delta updates)
  if (last_sync) {
    filteredAds = filteredAds.filter(ad => ad.timestamp > last_sync);
  }

  // Limit bundle size
  const limit = Math.min(max_ads || ADS_MAX_BUNDLE_SIZE, ADS_MAX_BUNDLE_SIZE);
  filteredAds = filteredAds.slice(0, limit);

  // Get advertiser encryption key
  const advertiserKey = await env.PUSH_STORE.get('ads:advertiser_key');
  if (!advertiserKey) {
    return new Response(JSON.stringify({ error: 'Advertiser key not configured' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // In production, encrypt the bundle here
  // For now, return as-is (encryption should be done by advertiser before storing)
  const encryptedBundle = {
    ciphertext: filteredAds, // Already encrypted by advertiser
    nonce: 'placeholder', // Should be generated during encryption
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

/**
 * Handle anonymous impression report
 * POST /api/v1/ads/report
 * 
 * Receives batch of impression hashes (NO PII)
 * Aggregates stats for advertisers
 */
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

  // Store impression hashes for aggregation
  const batchKey = `ads:impressions:${Date.now()}`;
  await env.PUSH_STORE.put(batchKey, JSON.stringify({
    hashes: impressionHashes,
    timestamp: Date.now(),
    count: impressionHashes.length,
  }), { expirationTtl: ADS_DEFAULT_TTL });

  // Update aggregate stats
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
    expirationTtl: 30 * 24 * 60 * 60, // 30 days
  });

  return new Response(JSON.stringify({
    success: true,
    recorded: impressionHashes.length,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
}

/**
 * Add ad to inventory (admin endpoint)
 * POST /api/v1/ads/add
 */
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

  // Get current inventory
  const adsJson = await env.PUSH_STORE.get('ads:inventory');
  const ads = adsJson ? JSON.parse(adsJson) : [];

  // Add or update ad
  const existingIndex = ads.findIndex(a => a.id === ad.id);
  if (existingIndex >= 0) {
    ads[existingIndex] = { ...ad, timestamp: Date.now() };
  } else {
    ads.push({ ...ad, timestamp: Date.now() });
  }

  await env.PUSH_STORE.put('ads:inventory', JSON.stringify(ads), {
    expirationTtl: 30 * 24 * 60 * 60, // 30 days
  });

  return new Response(JSON.stringify({
    success: true,
    ad_id: ad.id,
    total_ads: ads.length,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
}

/**
 * Get ad stats (admin endpoint)
 * GET /api/v1/ads/stats
 */
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

// Export functions for use in main worker
export {
  handleFetchBundle,
  handleReportImpressions,
  handleAddAd,
  handleGetStats,
};
