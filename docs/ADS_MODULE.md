# Рекламный Модуль (Ads Module)

## Обзор

Приватная рекламная система с локальным таргетингом и шифрованием.

### Принципы

- **NO tracking pixels, beacons, or analytics** — никаких трекеров
- **NO user data sent to ad servers** — данные пользователей не покидают устройство
- **All ad selection done ON-DEVICE** — выбор рекламы локально на основе категорий
- **Users EARN credits for viewing ads** — пользователи получают кредиты за просмотр
- **Advertiser payments go through Web3** — оплата через Web3 (без fiat-трекинга)

## Архитектура

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (React)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  AdBanner    │  │ AdSettings   │  │  ChatList        │  │
│  │  (баннер)    │  │ (настройки)  │  │  (список чатов)  │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │            │
│         └─────────────────┴────────────────────┘            │
│                           │                                  │
│                  Tauri Commands                              │
│         (select_ad, record_impression, etc)                 │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                    Backend (Rust/Tauri)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  AdEngine    │  │  AdStorage   │  │ AdBundleFetcher  │  │
│  │  (выбор)     │  │  (SQLite)    │  │  (fetch/decrypt) │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │            │
│         └─────────────────┼────────────────────┘            │
│                           │                                  │
│                    Cloudflare Worker                        │
│              (encrypted ad bundles)                         │
└─────────────────────────────────────────────────────────────┘
```

## Компоненты

### Rust Backend (`messenger/src/ads/`)

#### `mod.rs`
- Определение типов: `Ad`, `AdType`, `AdCategory`, `AdPreferences`, `AdImpression`, `AdStats`
- Error types: `AdError`, `AdResult`
- Re-exports для всех публичных типов

#### `engine.rs`
- `AdEngine` — основной движок выбора рекламы
- Локальный таргетинг по категориям (tech, news, gaming, etc)
- Rate limiting (max ads per hour)
- Credit system (watch-to-earn)
- Impression/click tracking
- Priority-based ad selection

#### `fetch.rs`
- `AdBundleFetcher` — загрузка зашифрованных пакетов из Cloudflare Worker
- `AdStorage` — SQLite хранение decrypted ads
- `EncryptedAdBundle` — тип зашифрованного пакета
- Anonymous impression reporting (batch, no PII)

#### `commands.rs`
- Tauri commands для frontend:
  - `cmd_fetch_ads` — загрузить рекламу
  - `cmd_select_ad` — выбрать рекламу
  - `cmd_record_impression` — записать просмотр
  - `cmd_record_click` — записать клик
  - `cmd_get_ad_settings` — получить настройки
  - `cmd_update_adpreferences` — обновить настройки
  - `cmd_get_ad_credits` — получить кредиты
  - `cmd_spend_ad_credits` — потратить кредиты
  - `cmd_list_ads` — список всей рекламы
  - `cmd_report_impressions` — анонимный отчёт
  - `cmd_cleanup_ads` — очистка устаревшей рекламы

### Frontend (`frontend/src/`)

#### `types/ads.ts`
- TypeScript типы для всех ad типов
- Tauri command constants
- Default preferences и category labels

#### `components/AdBanner.tsx`
- Баннер в списке чатов (НЕ внутри чата)
- Кнопка "Скрыть" (FiX icon)
- Auto impression tracking (3 sec delay)
- Click handling с открытием URL
- Credit reward badge (+N кредитов)

#### `components/AdSettings.tsx`
- Модальное окно настроек рекламы
- Privacy guarantees section
- Credits & stats dashboard
- Ad type toggles (banner, reward, native, interstitial)
- Max ads per hour slider
- Category preferences (preferred/blocked)
- Reset to defaults

## Приватность

### Локальный Таргетинг

```
User selects categories locally → NO data sent to server
     ↓
AdEngine filters ads by category → ON-DEVICE only
     ↓
Best ad selected by priority → NO tracking
```

### Шифрование Пакетов

```
Advertiser → encrypts ad bundle → Cloudflare Worker
     ↓
Client fetches encrypted bundle → decrypts locally
     ↓
Ads stored decrypted in SQLite → NO server access
```

### Анонимные Отчёты

```
Impression recorded locally → SHA3-256 hash(ad_id + timestamp)
     ↓
Batch report to server → ONLY hashes (NO PII)
     ↓
Advertiser gets aggregate stats → NO individual tracking
```

## Интеграция

### 1. Инициализация AdState (main.rs)

```rust
#[cfg(feature = "tauri-commands")]
let ad_state = secure_messenger_lib::AdState::new(
    secure_messenger_lib::AdPreferences::default()
);

builder = secure_messenger_lib::register_ad_commands(builder, ad_state);
```

### 2. ChatList Integration (React)

```tsx
<ChatList>
  {/* ... */}
  <AdBanner
    enabled={bannerEnabled}
    onHidden={() => setBannerEnabled(false)}
  />
  <button onClick={() => setIsAdSettingsOpen(true)}>
    Настройки рекламы
  </button>
  <AdSettingsModal isOpen={isAdSettingsOpen} onClose={...} />
</ChatList>
```

### 3. Cloudflare Worker Endpoints

Нужно добавить в Worker:

```
POST /api/v1/ads/bundle  → fetch encrypted ad bundle
POST /api/v1/ads/report  → anonymous impression report
```

## Тестирование

### Rust Tests

```bash
cd messenger
cargo test ads::
```

Тесты покрывают:
- Ad creation и validation
- Category filtering
- Impression tracking
- Click handling
- Rate limiting
- Priority selection
- Storage CRUD operations
- Bundle serialization

### Frontend Testing

Нужно проверить:
- [ ] AdBanner renders в ChatList
- [ ] Кнопка "Скрыть" работает
- [ ] AdSettingsModal открывается
- [ ] Category toggles обновляют preferences
- [ ] Impressions записываются через 3 сек
- [ ] Clicks открывают URL в браузере
- [ ] Credits отображаются корректно

## TODO

### Критично
- [ ] Реализовать actual ChaCha20-Poly1305 decryption в `decrypt_ad_bundle()`
- [ ] Добавить Cloudflare Worker endpoints (`/api/v1/ads/bundle`, `/api/v1/ads/report`)
- [ ] AdState initialization с db_path и worker_url (сейчас hardcoded)

### Важно
- [ ] Image caching для ad креативов
- [ ] Offline mode (показывать кэшированные ads)
- [ ] Ad frequency capping (не показывать одну и ту же ad подряд)
- [ ] A/B testing framework (privacy-preserving)

### Nice-to-Have
- [ ] Interstitial ads implementation
- [ ] Reward ad video integration
- [ ] Ad quality controls (advertiser verification)
- [ ] zk-proof для impression verification
- [ ] Ad revenue sharing (Web3 payments)

## Безопасность

### Аудит
- [ ] Audit ad bundle encryption implementation
- [ ] Verify NO PII leaks in impression reporting
- [ ] Check rate limiting effectiveness
- [ ] Review SQLite storage for SQL injection

### Threat Model
- **Threat**: Malicious ad bundle tries to exploit decryption
  - **Mitigation**: ChaCha20-Poly1305 authenticated encryption
- **Threat**: Impression hashes can be correlated
  - **Mitigation**: Nanosecond timestamp adds entropy
- **Threat**: Ad categories reveal user interests to server
  - **Mitigation**: Categories sent for filtering ONLY, selection is local

## API Reference

### Tauri Commands

#### `fetch_ads(request: FetchAdsRequest) → FetchAdsResponse`
Загружает зашифрованный пакет рекламы из Cloudflare Worker.

#### `select_ad(request: SelectAdRequest) → SelectAdResponse`
Выбирает рекламу локально на основе preferences.

#### `record_impression(request: RecordImpressionRequest) → RecordImpressionResponse`
Записывает просмотр рекламы locally.

#### `record_click(request: RecordClickRequest) → RecordClickResponse`
Записывает клик и возвращает URL для открытия.

#### `get_ad_settings() → AdSettings`
Возвращает текущие preferences, credits, и stats.

#### `update_ad_preferences(preferences: AdPreferences) → ()`
Обновляет локальные preferences.

#### `get_ad_credits() → u32`
Возвращает текущий баланс кредитов.

#### `spend_ad_credits(amount: u32) → bool`
Тратит кредиты (возвращает success/failure).

#### `list_ads() → Vec<Ad>`
Возвращает все активные ads (для settings page).

#### `report_impressions() → ()`
Отправляет anonymous batch report на server.

#### `cleanup_ads() → u32`
Удаляет expired ads, возвращает количество удалённых.

## Лицензия

MIT — Secure Telegram Team
