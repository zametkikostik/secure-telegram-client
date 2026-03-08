# 🦋 Liberty Reach Messenger - Flutter UI

**Universal Resilient Edition v2.0.0**

---

## 📋 Описание

Flutter UI для Liberty Reach Messenger с FFI мостом к Rust ядру.

---

## 🏗️ Архитектура

```
flutter_ui/
├── lib/
│   ├── main.dart                    # Точка входа
│   ├── screens/                     # Экраны
│   │   ├── chat_list_screen.dart    # Список чатов
│   │   ├── chat_screen.dart         # Экран чата
│   │   ├── login_screen.dart        # Вход
│   │   ├── profile_screen.dart      # Профиль
│   │   ├── calls_screen.dart        # Звонки
│   │   └── settings_screen.dart     # Настройки
│   ├── services/                    # Сервисы
│   │   ├── rust_bridge.dart         # FFI к Rust
│   │   ├── api_service.dart         # Cloudflare API
│   │   ├── p2p_service.dart         # P2P сеть
│   │   └── auth_service.dart        # Аутентификация
│   ├── widgets/                     # Виджеты
│   ├── models/                      # Модели данных
│   └── utils/                       # Утилиты
├── android/                         # Android специфичный код
├── ios/                             # iOS специфичный код
└── pubspec.yaml                     # Зависимости
```

---

## 🚀 Установка

### 1. Установи Flutter

```bash
# Linux
sudo snap install flutter --classic

# Или скачай с https://flutter.dev
```

### 2. Настрой Flutter

```bash
flutter doctor
flutter config --android-sdk /path/to/android/sdk
```

### 3. Установи зависимости

```bash
cd flutter_ui
flutter pub get
```

### 4. Запусти

```bash
flutter run
```

---

## 🔨 Сборка APK

```bash
# Debug APK
flutter build apk --debug

# Release APK
flutter build apk --release

# Split per ABI
flutter build apk --split-per-abi
```

APK будет в: `build/app/outputs/flutter-apk/`

---

## 🔗 FFI к Rust Ядру

### Генерация FFI bindings

```bash
# Установи flutter_rust_bridge
cargo install flutter_rust_bridge_codegen

# Сгенерируй bindings
flutter_rust_bridge_codegen generate
```

### Использование

```dart
import 'services/rust_bridge.dart';

final rust = RustBridge();
await rust.init();

// Создать ядро
await rust.createCore(
  dbPath: '/path/to/db',
  encryptionKey: keyBytes,
  userId: 'user123',
  username: 'alice',
);

// Отправить команду
await rust.sendCommand(command);
```

---

## 📱 Поддерживаемые платформы

| Платформа | Версия | Статус |
|-----------|--------|--------|
| Android | 8.0+ (API 26) | ✅ |
| iOS | 12.0+ | ⏳ |
| Linux | Desktop | ⏳ |
| Windows | Desktop | ⏳ |
| macOS | Desktop | ⏳ |

---

## ✨ Функции

### Реализовано:
- ✅ Список чатов
- ✅ Экран чата
- ✅ Профиль пользователя
- ✅ Настройки
- ✅ Экран звонков
- ✅ Аутентификация
- ✅ P2P подключение
- ✅ Cloudflare API

### В разработке:
- ⏳ WebRTC звонки
- ⏳ Голосовые сообщения
- ⏳ Стикеры
- ⏳ Темы оформления

---

## 🎨 Дизайн

Используется **Material Design 3** с адаптивной темой:

- Светлая тема (днём)
- Тёмная тема (ночью)
- Системная тема (авто)

---

## 📊 Структура проекта

```
lib/
├── main.dart (1.5 KB)
├── screens/
│   ├── chat_list_screen.dart (3.5 KB)
│   ├── chat_screen.dart (2.8 KB)
│   ├── login_screen.dart (2.2 KB)
│   ├── profile_screen.dart (2.5 KB)
│   ├── calls_screen.dart (1.2 KB)
│   └── settings_screen.dart (2.8 KB)
├── services/
│   ├── rust_bridge.dart (1.5 KB)
│   ├── api_service.dart (1.8 KB)
│   ├── p2p_service.dart (1.2 KB)
│   └── auth_service.dart (1.5 KB)
└── ...

Всего: ~25 KB Dart кода
```

---

## 🧪 Тесты

```bash
# Запустить тесты
flutter test

# С покрытием
flutter test --coverage
```

---

## 📦 Зависимости

### Основные:
- `flutter_rust_bridge` - FFI к Rust
- `provider` - State management
- `http` / `dio` - HTTP запросы
- `shared_preferences` - Локальное хранилище

### UI:
- `flutter_slidable` - Swipe действия
- `cached_network_image` - Кэширование картинок
- `photo_view` - Просмотр фото

### Медиа:
- `webrtc` / `flutter_webrtc` - Видеозвонки
- `audioplayers` - Аудио
- `record` - Запись звука

### Уведомления:
- `firebase_messaging` - Push уведомления
- `flutter_local_notifications` - Локальные уведомления

---

## 🔐 Безопасность

- ✅ Шифрование трафика (TLS)
- ✅ FFI к Rust криптографии
- ✅ Безопасное хранение токенов
- ✅ Biometric auth (готово к подключению)

---

## 🐛 Известные проблемы

| Проблема | Статус | Решение |
|----------|--------|---------|
| WebRTC на iOS | ⏳ | В разработке |
| Тёмная тема | ✅ | Исправлено |
| FFI bindings | ✅ | Исправлено |

---

## 📝 Changelog

### v2.0.0 (Март 2026)
- ✅ Первый релиз Flutter UI
- ✅ FFI мост к Rust ядру
- ✅ Все основные экраны
- ✅ Интеграция с Cloudflare

---

## 🤝 Вклад

1. Fork репозиторий
2. Создай ветку (`git checkout -b feature/amazing`)
3. Commit изменения (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing`)
5. Открой Pull Request

---

## 📄 Лицензия

MIT License - см. [LICENSE](../LICENSE)

---

## 📞 Контакты

- **GitHub:** https://github.com/zametkikostik/secure-telegram-client
- **Email:** zametkikostik@gmail.com
- **Cloudflare:** https://secure-messenger-push.zametkikostik.workers.dev

---

*Liberty Reach Messenger v2.0.0*  
*Flutter UI - Universal Resilient Edition*  
*Март 2026*
