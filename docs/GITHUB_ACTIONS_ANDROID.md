# 🚀 GitHub Actions - Android APK Build Guide

## 📋 Что уже настроено

✅ **GitHub Actions workflow** в `.github/workflows/ci-cd.yml`:
- **Debug APK** собирается при каждом push на main
- **Release APK** собирается при создании тега (v*)
- **Автоматическая подпись** APK через keystore в GitHub Secrets
- **Upload артефактов** на 30 дней (debug) и 90 дней (release)

---

## 🔐 Шаг 1: Генерация Keystore

```bash
cd scripts
./generate-keystore.sh
```

Скрипт:
1. ✅ Создаст `secure-messenger.keystore`
2. ✅ Закодирует в `keystore-base64.txt`
3. ✅ Покажет инструкции по настройке GitHub

⚠️ **ВАЖНО:** После настройки GitHub Secrets **удали** `secure-messenger.keystore`!

---

## 🔧 Шаг 2: Настройка GitHub Secrets

Перейди в: **GitHub → Repository → Settings → Secrets and variables → Actions**

Добавь 4 секрета:

| Secret Name | Значение | Пример |
|-------------|----------|--------|
| `ANDROID_KEYSTORE_BASE64` | Всё из `keystore-base64.txt` | `MIIJqAIBAz...` |
| `ANDROID_KEYSTORE_PASSWORD` | Пароль от keystore | `android123` |
| `ANDROID_KEY_ALIAS` | Alias ключа | `secure-messenger` |
| `ANDROID_KEY_PASSWORD` | Пароль от ключа | `android123` |

---

## 📱 Шаг 3: Инициализация Android проекта

Перед первым запуском нужно создать Android проект:

```bash
cd mobile
npx react-native init SecureMessenger --template react-native-template-typescript
```

Или используй готовый скрипт:

```bash
cd scripts
./mobile-setup.sh
```

---

## 🏗️ Шаг 4: Тест локальной сборки

```bash
cd mobile
npm install
cd android
./gradlew assembleDebug
```

APK будет в: `android/app/build/outputs/apk/debug/app-debug.apk`

---

## 🚀 Шаг 5: Запуск через GitHub Actions

### Debug APK (при push на main)

```bash
git add .
git commit -m "feat: mobile setup"
git push origin main
```

Результат: **Debug APK** в GitHub Actions artifacts

### Release APK (при теге)

```bash
git tag v0.1.0
git push origin v0.1.0
```

Результат: **Signed Release APK** в GitHub Actions artifacts

---

## 📦 Где скачать APK

1. Открой репозиторий на GitHub
2. Перейди в **Actions**
3. Выбери последний запуск
4. Прокрути вниз до **Artifacts**
5. Скачай `secure-messenger-android-debug` или `secure-messenger-android-release`

---

## 🛠️ Что включает workflow

```yaml
build-android:
  ✅ Setup Java 17
  ✅ Setup Node.js 20
  ✅ npm install
  ✅ Setup Android SDK
  ✅ Cache Gradle dependencies
  ✅ Build debug APK
  ✅ Build release APK (только для тегов)
  ✅ Sign APK с keystore из secrets
  ✅ Upload artifacts
```

---

## 🔍 Troubleshooting

### Ошибка: "Keystore not found"
- Проверь, что добавил все 4 секрета в GitHub
- Убедись, что `ANDROID_KEYSTORE_BASE64` не пустой

### Ошибка: "Gradle build failed"
- Проверь логи в GitHub Actions
- Попробуй локально: `cd mobile/android && ./gradlew clean assembleDebug`

### Ошибка: "SDK not found"
- Workflow автоматически устанавливает Android SDK
- Если ошибка, проверь `android-actions/setup-android@v3` в workflow

### APK не устанавливается
- Debug APK подписан debug ключом - работает на эмуляторе и подключенных устройствах
- Release APK подписан твоим keystore - можно устанавливать вручную

---

## 📊 Статус сборки

| Триггер | Debug APK | Release APK | Подпись |
|---------|-----------|-------------|---------|
| Push на main | ✅ | ❌ | Debug key |
| Тег (v*) | ✅ | ✅ | Твой keystore |
| Pull Request | ✅ | ❌ | Debug key |

---

## 🎯 Следующие шаги

1. ✅ Настроить keystore и GitHub Secrets
2. ✅ Протестировать локальную сборку
3. ✅ Push на main → проверить debug APK
4. ✅ Создать тег → проверить release APK
5. ✅ Установить APK на устройство
6. ⬜ Загрузить в Google Play Console

---

## 📞 Поддержка

Если что-то не работает:
1. Проверь логи GitHub Actions
2. Проверь, что все секреты добавлены
3. Протестируй локальную сборку
4. Открой Issue с логами ошибки
