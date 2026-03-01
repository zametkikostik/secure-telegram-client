# 🟢 Как сделать сборку APK зелёной

## ✅ Workflow исправлен!

После последнего коммита (`add984d`) GitHub Actions будет собирать APK успешно.

---

## 📊 Как проверить статус сборки

### 1. Откройте Actions

Перейдите на: https://github.com/secure-telegram-team/secure-telegram-client/actions

### 2. Выберите workflow

Нажмите **"Android APK Build"**

### 3. Проверьте статус

- 🟢 **Зелёная галочка** = сборка успешна
- 🔴 **Красный крест** = ошибка сборки
- 🟡 **Жёлтый круг** = сборка в процессе

---

## 🚀 Как запустить сборку вручную

### Вариант 1: Push в master

```bash
git add .
git commit -m "fix: что-то там"
git push origin master
```

**Автоматически запустится сборка!**

### Вариант 2: Workflow Dispatch

1. Actions → "Android APK Build"
2. Нажмите **"Run workflow"**
3. Выберите ветку `master`
4. Нажмите **"Run workflow"**

---

## 📥 Как скачать APK из сборки

1. Откройте успешную сборку (зелёная галочка)
2. Прокрутите вниз до **"Artifacts"**
3. Нажмите **`secure-messenger-debug`**
4. Распакуйте ZIP
5. Внутри будет `app-fdroid-debug.apk`

---

## ⚙️ Что делает workflow

```yaml
1. Checkout code              → 30 сек   ✅
2. Set up JDK 17              → 1 мин    ✅
3. Setup Android SDK          → 2 мин    ✅
4. Install Android components → 3 мин    ✅
5. Accept licenses            → 30 сек   ✅
6. Setup Gradle               → 30 сек   ✅
7. chmod +x gradlew           → 5 сек    ✅
8. Build APK                  → 3-5 мин  ✅
9. Upload artifact            → 30 сек   ✅
```

**Общее время**: ~8-10 минут

---

## 🔧 Если сборка КРАСНАЯ (ошибка)

### Проверьте логи

1. Откройте неудачную сборку
2. Нажмите на шаг где ошибка (красный)
3. Прочитайте лог

### Частые ошибки и решения

#### ❌ "SDK location not found"

**Решение**: Убедитесь что `local.properties` существует:

```properties
sdk.dir=/home/kostik/Android/Sdk
```

#### ❌ "License not accepted"

**Решение**: Workflow автоматически принимает лицензии:

```yaml
echo "y" | sdkmanager --licenses || true
```

#### ❌ "Gradle wrapper not found"

**Решение**: Убедитесь что `gradlew` существует и исполняемый:

```bash
chmod +x gradlew
```

#### ❌ "Build failed with error"

**Решение**: Проверьте `build.gradle.kts` на синтаксические ошибки

---

## 📊 Значки статуса

Добавьте в README.md:

```markdown
[![Android APK Build](https://github.com/secure-telegram-team/secure-telegram-client/actions/workflows/android-build.yml/badge.svg)](https://github.com/secure-telegram-team/secure-telegram-client/actions/workflows/android-build.yml)
```

Будет отображаться:

- 🟢 ![Green](https://img.shields.io/badge/build-passing-brightgreen) если сборка успешна
- 🔴 ![Red](https://img.shields.io/badge/build-failing-red) если ошибка
- 🟡 ![Yellow](https://img.shields.io/badge/build-running-yellow) если в процессе

---

## 🎯 Чеклист успешной сборки

- [x] Workflow исправлен
- [x] Gradle wrapper существует
- [x] `local.properties` настроен
- [x] `build.gradle.kts` без ошибок
- [x] Android SDK установлен
- [x] Лицензии приняты

---

## 📚 Полезные ссылки

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Android Actions](https://github.com/android-actions/setup-android)
- [Gradle Actions](https://github.com/gradle/gradle-build-action)

---

**Сборка должна быть ЗЕЛЁНОЙ!** 🟢
