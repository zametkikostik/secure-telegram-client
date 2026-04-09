# Migration Tool

Инструменты для миграции данных из Telegram, WhatsApp и других мессенджеров во внутренний формат Secure Messenger.

## Telegram Importer

Парсинг Telegram Desktop JSON export во внутренний формат.

### Использование

```bash
# Базовое использование
python3 telegram_importer.py <path_to_result.json>

# С указанием выходного файла
python3 telegram_importer.py ~/Telegram\ Desktop/tdata/exports/chat_2024-01-15/result.json -o migrated.json

# Форматированный вывод
python3 telegram_importer.py result.json --pretty

# Только статистика
python3 telegram_importer.py result.json --stats-only
```

## WhatsApp Importer

Парсинг WhatsApp TXT export (_chat.txt) во внутренний формат.

### Использование

```bash
# Базовое использование
python3 whatsapp_importer.py <path_to_chat.txt>

# С именем чата и форматом даты
python3 whatsapp_importer.py WhatsApp\ Chat.txt --chat-name "Проект Alpha" --date-format "%m/%d/%Y"

# Только статистика
python3 whatsapp_importer.py _chat.txt --stats-only
```

### Формат WhatsApp TXT

Экспорт из WhatsApp: `Chat → ⋮ → More → Export chat → Without media` (или `With media` для файлов).

Поддерживаемые типы сообщений:
- **Текст** — включая многострочные сообщения
- **Медиа** — voice (.opus), video (.mp4, .mov), photo (.jpg, .png), document (.pdf, .doc)
- **Стикеры** — `<sticker omitted>`
- **Системные** — "was added", "changed the subject", "Security code changed"
- **Многострочные** — автоматически объединяются

## AI Translator

Авто-перевод импортированных сообщений на целевой язык.

### Использование

```bash
# Google Translate (бесплатно, без ключа)
python3 ai_translator.py internal_export.json --lang en

# OpenAI (требуется OPENAI_API_KEY)
python3 ai_translator.py internal_export.json --lang en --api openai --model gpt-4o-mini

# Ollama (локальная модель)
python3 ai_translator.py internal_export.json --lang en --api ollama --model llama3

# Только детекция языка без перевода
python3 ai_translator.py internal_export.json --detect-only

# Перевод с сохранением оригинала (не перезаписывает)
python3 ai_translator.py internal_export.json --lang de -o translated_de.json

# Переводить даже сообщения на целевом языке
python3 ai_translator.py internal_export.json --lang en --translate-all
```

### Поддерживаемые API

| API | Ключ | Качество | Скорость |
|-----|------|----------|----------|
| `google` | Нет | Хорошее | Быстро |
| `openai` | OPENAI_API_KEY | Отличное | Средне |
| `ollama` | Нет (локально) | Зависит от модели | Медленно |

### Детекция языка

Встроенная детекция без зависимостей:
- По n-граммам (3-4 символа)
- По стоп-словам
- Поддерживаемые языки: en, ru, de, es, fr, uk, zh, ja

### Входной формат

Инструмент поддерживает два формата экспорта Telegram Desktop:

1. **Один чат** — `result.json` с полями `name`, `type`, `id`, `messages[]`
2. **Несколько чатов** — `result.json` с полем `chats.list[]`

Экспорт создаётся через Telegram Desktop:
`Settings → Advanced → Export Telegram Data → выберите формат JSON`

### Выходной формат

```json
{
  "version": "1.0",
  "exported_at": "2024-01-15T10:30:00+00:00",
  "source": {
    "type": "telegram_desktop_export",
    "name": "Chat Name",
    "file": "path/to/result.json"
  },
  "users": [
    {
      "id": "tg_123456",
      "display_name": "User Name",
      "telegram_id": "123456",
      "first_seen": "2024-01-01T00:00:00+00:00",
      "last_seen": "2024-01-15T10:30:00+00:00",
      "messages_count": 42
    }
  ],
  "chats": [
    {
      "id": "abc123def456",
      "name": "Chat Name",
      "chat_type": "direct|group|channel",
      "telegram_id": 123456789,
      "telegram_type": "personal_chat|private_group|...",
      "participants": ["tg_111", "tg_222"],
      "messages_count": 100,
      "last_message_at": "2024-01-15T10:30:00+00:00"
    }
  ],
  "messages": [
    {
      "id": "msg_abc123",
      "chat_id": "abc123def456",
      "sender_id": "tg_123456",
      "sender_name": "User Name",
      "content": "Текст сообщения",
      "formatting": [{"type": "bold", "offset": 0, "length": 4}],
      "msg_type": "text|photo|video|document|voice|service|...",
      "created_at": "2024-01-15T10:30:00+00:00",
      "edited_at": null,
      "reply_to_message_id": null,
      "forward_from": null,
      "forward_date": null,
      "attachments": [],
      "service_data": null,
      "reactions": [],
      "raw_telegram_id": 1
    }
  ],
  "stats": {
    "total_users": 2,
    "total_chats": 1,
    "total_messages": 100,
    "total_media_attachments": 15,
    "total_service_messages": 3
  }
}
```

### Поддерживаемые типы сообщений

- **Текст** — с сохранением форматирования (bold, italic, code, link, etc.)
- **Медиа** — photo, video, document, audio, voice, video_note, gif, sticker
- **Локации** — geolocation с координатами
- **Контакты** — shared contacts
- **Опросы** — polls с вариантами ответов
- **Служебные** — service messages (change title, add/remove member, pin message, etc.)
- **Пересланные** — forwarded messages с информацией об источнике
- **Ответы** — reply chains с ссылками на оригинальные сообщения
- **Реакции** — emoji reactions

### ID генерация

- **user_id**: `tg_<telegram_user_id>`
- **chat_id**: SHA256(`tg_chat:<telegram_chat_id>:<chat_type>`)[:16]
- **message_id**: SHA256(`<chat_id>:msg:<telegram_msg_id>`)[:16]

## Валидация данных

Многоуровневая система валидации обеспечивает целостность данных на каждом этапе миграции.

### Уровни валидации

| Уровень | Что проверяет | Модуль |
|---------|--------------|--------|
| **Schema** | Структура JSON, типы полей, required поля | `TelegramSchemaValidator` |
| **WhatsApp Schema** | Формат строк, парсинг дат, пустые файлы | `WhatsAppSchemaValidator` |
| **Business Rules** | Дубликаты ID, orphan ссылки, диапазоны дат, размеры | `BusinessRulesValidator` |
| **SQLite Integrity** | PRAGMA integrity_check, orphan записи, индексы | `SQLiteIntegrityValidator` |

### Правила валидации

**Schema validation:**
- Required поля: `name`, `type`, `id`, `messages`
- Типы: `id` → int/str, `messages` → array, `text` → str/array
- Формат дат: ISO 8601 (`YYYY-MM-DDTHH:MM:SS`)
- Типы чатов: `personal_chat`, `private_group`, `public_channel`, etc.
- Форматирование текста: `bold`, `italic`, `code`, `link`, etc.

**Business rules:**
- ✅ Уникальность `user_id`, `chat_id`, `message_id`
- ✅ Все `reply_to` ссылаются на существующие сообщения
- ✅ Все `chat_id` в сообщениях существуют в `chats`
- ✅ Диапазон дат: 2000–2050
- ✅ Длина текста < 4096 символов
- ✅ attachments < 10 на сообщение
- ✅ Валидный base64 в `encrypted_content`

**SQLite integrity:**
- `PRAGMA integrity_check` = OK
- Все таблицы: `contacts`, `chat_history`, `messages_cache`, `settings`
- Нет orphan сообщений (ссылаются на несуществующие чаты)
- Migration metadata записана

### Использование

```bash
# Standalone валидация файла
python3 validator.py --input result.json

# С авто-исправлением
python3 validator.py --input result.json --fix --fixed-output fixed.json

# Валидация SQLite БД
python3 validator.py --input migrated.db --db-check

# Строгий режим (предупреждения = ошибки)
python3 validator.py --input result.json --strict

# JSON отчёт
python3 validator.py --input result.json --format json --output report.json
```

### Интеграция в migrate.py

Валидация **включена по умолчанию** в pipeline миграции:

```bash
# Стандартная миграция (валидация + import + SQLite)
python3 migrate.py --input result.json --output migrated.db

# С валидацией БД после записи
python3 migrate.py --input result.json --output migrated.db --validate-db

# Авто-исправление типовых проблем
python3 migrate.py --input result.json --output migrated.db --fix

# Строгий режим
python3 migrate.py --input result.json --output migrated.db --strict

# Пропустить валидацию (не рекомендуется)
python3 migrate.py --input result.json --output migrated.db --no-validate
```

### Авто-исправления

`--fix` автоматически исправляет:
- Неизвестные `chat_type` → `direct`
- Слишком длинные имена чатов → обрезка до 255 символов
- Пустые имена пользователей → `User_<id[:8]>`
- Future даты → текущее время
- Ancient даты (до 2000) → 2000-01-01
- Дубликаты сообщений → удаление дубликатов

### Тесты

```bash
# Запуск unit tests
python3 -m pytest tests/test_validator.py -v

# Тестовые данные
test_data/result.json              # Валидный экспорт (5 сообщений)
test_data/result_with_errors.json  # Экспорт с 11 типами ошибок
test_data/whatsapp_chat.txt        # Валидный WhatsApp чат
```

### API модуля

```python
from validator import (
    validate_telegram_json,
    validate_whatsapp_txt,
    validate_sqlite_db,
    auto_fix,
    Severity,
    ValidationReport,
)

# Валидация Telegram JSON
with open("result.json") as f:
    data = json.load(f)
report = validate_telegram_json(data)
print(report.summary())

# Авто-исправление
if not report.is_valid:
    data = auto_fix(data, report)

# Валидация WhatsApp
report = validate_whatsapp_txt(Path("chat.txt"))

# Валидация SQLite
report = validate_sqlite_db("migrated.db")
```

## Структура

```
migration-tool/
├── migrate.py                # Единый CLI: валидация + импорт + перевод + SQLite
├── telegram_importer.py      # Парсер Telegram JSON export
├── whatsapp_importer.py      # Парсер WhatsApp TXT export
├── ai_translator.py          # AI авто-перевод сообщений
├── validator.py              # Multi-layer валидация + auto-fix
├── test_data/                # Тестовые данные
│   ├── result.json           # Telegram тестовый экспорт
│   ├── result_with_errors.json  # Экспорт с ошибками
│   └── whatsapp_chat.txt     # WhatsApp тестовый чат
├── tests/                    # Unit тесты
│   ├── __init__.py
│   └── test_validator.py     # 34 теста валидации
├── README.md                 # Этот файл
└── .gitignore
```

## Использование

### Быстрый старт

```bash
# Telegram → SQLite
python3 migrate.py --input telegram_export/result.json --output migrated.db

# WhatsApp → SQLite
python3 migrate.py --input chat.txt --output migrated.db --chat-name "Группа"

# С авто-переводом на английский
python3 migrate.py --input export.json --output migrated.db --translate en

# С OpenAI переводом
python3 migrate.py --input export.json --output migrated.db --translate en --translate-api openai

# Dry-run (только статистика)
python3 migrate.py --input export.json --dry-run
```

### Отдельные модули

```bash
# Только Telegram импорт → JSON
python3 telegram_importer.py result.json -o export.json --pretty

# Только WhatsApp импорт → JSON
python3 whatsapp_importer.py chat.txt -o export.json --pretty

# Только AI-перевод
python3 ai_translator.py export.json --lang en -o export_en.json

# Только детекция языка
python3 ai_translator.py export.json --detect-only
```

## Разработка

```bash
# Проверка синтаксиса
python3 -m py_compile telegram_importer.py
python3 -m py_compile whatsapp_importer.py
python3 -m py_compile ai_translator.py

# Запуск тестов
python3 telegram_importer.py test_data/result.json --stats-only
python3 whatsapp_importer.py test_data/whatsapp_chat.txt --stats-only
python3 ai_translator.py test_data/result.json --detect-only
```
