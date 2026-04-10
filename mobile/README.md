# 📱 Secure Messenger Mobile

> React Native приложение для Secure Messenger

## 🚀 Быстрый старт

### 1. Установка зависимостей

```bash
cd mobile
npm install
```

### 2. Инициализация Android проекта

```bash
npx react-native init . --skip-install --template react-native-template-typescript
```

Или создай Android проект вручную:

```bash
cd android
# Следуй официального руководству:
# https://reactnative.dev/docs/environment-setup
```

### 3. Запуск Metro bundler

```bash
npm start
```

### 4. Запуск на Android

```bash
npm run android
```

### 5. Сборка APK

**Debug APK:**
```bash
cd android
./gradlew assembleDebug
```

APK будет в: `android/app/build/outputs/apk/debug/app-debug.apk`

**Release APK:**
```bash
cd android
./gradlew assembleRelease
```

---

## 🔐 Подпись APK для Production

### Генерация Keystore

```bash
cd ../scripts
./generate-keystore.sh
```

Следуй инструкциям скрипта. Он создаст:
- `secure-messenger.keystore` — файл keystore
- `keystore-base64.txt` — base64 для GitHub Secrets
- `GITHUB_SECRETS_SETUP.md` — инструкция по настройке

### Настройка GitHub Secrets

1. Перейди в **GitHub → Repository → Settings → Secrets and variables → Actions**
2. Добавь следующие секреты:

| Secret | Значение |
|--------|----------|
| `ANDROID_KEYSTORE_BASE64` | Содержимое `keystore-base64.txt` |
| `ANDROID_KEYSTORE_PASSWORD` | Пароль от keystore |
| `ANDROID_KEY_ALIAS` | Alias ключа |
| `ANDROID_KEY_PASSWORD` | Пароль от ключа |

### Автоматическая сборка через GitHub Actions

- **Push на main** → Собирается **debug APK**
- **Создание тега (v*)** → Собирается **signed release APK**

```bash
# Создание релиза
git tag v0.1.0
git push origin v0.1.0
```

---

## 📁 Структура

```
mobile/
├── App.tsx                       # Главный файл приложения
├── index.js                      # Entry point
├── package.json                  # Dependencies
├── android/                      # Android проект (генерируется)
├── ios/                          # iOS проект (генерируется)
├── src/
│   ├── navigation/               # React Navigation
│   │   └── index.tsx
│   ├── screens/                  # Экраны приложения
│   │   ├── LoginScreen.tsx
│   │   ├── RegisterScreen.tsx
│   │   ├── ChatsScreen.tsx
│   │   ├── ChatScreen.tsx
│   │   ├── ContactsScreen.tsx
│   │   ├── SettingsScreen.tsx
│   │   └── ProfileScreen.tsx
│   ├── store/                    # Redux Toolkit
│   │   ├── index.ts
│   │   └── slices/
│   │       ├── authSlice.ts
│   │       ├── chatSlice.ts
│   │       └── userSlice.ts
│   ├── services/                 # API и Crypto
│   │   ├── api.ts
│   │   ├── crypto.ts
│   │   └── pushNotifications.ts
│   └── utils/
│       └── constants.ts
└── __tests__/                    # Тесты
```

---

## 🔧 Разработка

### Добавление нового экрана

```bash
# Создай файл в src/screens/
touch src/screens/NewScreen.tsx

# Добавь в навигацию src/navigation/index.tsx
```

### Работа с API

```typescript
import {chatsAPI, usersAPI, authAPI} from './services/api';

// Получение чатов
const chats = await chatsAPI.getChats();

// Отправка сообщения с E2EE
const encrypted = await cryptoService.encrypt(message, sharedSecret);
await chatsAPI.sendMessage(chatId, message, encrypted);
```

### Криптография

```typescript
import {cryptoService} from './services/crypto';

// Генерация ключей
const {publicKey, privateKey} = await cryptoService.generateKeyPair();

// Обмен ключами
const sharedSecret = await cryptoService.deriveSharedSecret(privateKey, theirPublicKey);

// Шифрование
const {ciphertext, iv, tag} = await cryptoService.encrypt(message, sharedSecret);

// Подпись
const signature = await cryptoService.sign(data, privateKey);
```

---

## 🧪 Тестирование

```bash
npm test
```

---

## 🐛 Отладка

### Metro bundler логи
```bash
npm start -- --reset-cache
```

### Android логи
```bash
adb logcat *:E
```

### React DevTools
```bash
npx react-devtools
```

---

## 📦 Деплой

### Google Play Console

1. Собери signed APK через GitHub Actions (создай тег)
2. Скачай APK из GitHub Actions artifacts
3. Загрузи в Google Play Console

### APK Installation

```bash
# Установка на устройство
adb install android/app/build/outputs/apk/release/app-release.apk
```

---

## 🔗 Ссылки

- [React Native Docs](https://reactnative.dev/docs/getting-started)
- [Redux Toolkit](https://redux-toolkit.js.org/)
- [React Navigation](https://reactnavigation.org/)
- [Secure Messenger Backend](https://secure-messenger-push.kostik.workers.dev)

---

## ⚠️  TODO

- [ ] Реализовать полноценное E2EE шифрование (react-native-quick-crypto)
- [ ] Интеграция с Cloudflare Worker для push-уведомлений
- [ ] WebRTC звонки
- [ ] Offline поддержка (Redux Persist)
- [ ] Файлы и изображения
- [ ] Группы и каналы
- [ ] Импорт из Telegram/WhatsApp
