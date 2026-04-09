#!/usr/bin/env python3
"""
Migration CLI — единый интерфейс для миграции данных.

Пайплайн:
    0. Валидация входных данных (опционально, по умолчанию: включена)
    1. Импорт (Telegram JSON / WhatsApp TXT)
    2. AI-перевод (опционально)
    3. Запись в SQLite (совместимо с Rust LocalStorage)
    4. Валидация SQLite БД (опционально)

Использование:
    # Telegram → SQLite (с валидацией по умолчанию)
    python3 migrate.py --input telegram_export/result.json --output migrated.db

    # С авто-исправлением
    python3 migrate.py --input export.json --output migrated.db --fix

    # Строгий режим (предупреждения = ошибки)
    python3 migrate.py --input export.json --output migrated.db --strict

    # Без валидации
    python3 migrate.py --input export.json --output migrated.db --no-validate

    # С валидацией БД после записи
    python3 migrate.py --input export.json --output migrated.db --validate-db

    # WhatsApp → SQLite
    python3 migrate.py --input whatsapp_chat.txt --output migrated.db

    # С авто-переводом
    python3 migrate.py --input export.json --output migrated.db --translate en

    # Режим dry-run (только статистика)
    python3 migrate.py --input export.json --dry-run
"""

import argparse
import base64
import json
import sqlite3
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

# Импорты из существующих модулей
sys.path.insert(0, str(Path(__file__).parent))
from telegram_importer import parse_telegram_export
from whatsapp_importer import parse_whatsapp_chat
from ai_translator import AITranslator
from validator import (
    validate_telegram_json,
    validate_whatsapp_txt,
    validate_sqlite_db,
    auto_fix,
    Severity,
)


# ============================================================================
# Автоопределение формата входного файла
# ============================================================================

def detect_input_format(file_path: Path) -> str:
    """
    Определение формата: 'telegram' или 'whatsapp'.
    
    Telegram: result.json с JSON-данными
    WhatsApp: .txt файл с форматом DD/MM/YYYY, HH:MM
    """
    if file_path.suffix == ".json":
        return "telegram"
    if file_path.suffix == ".txt":
        return "whatsapp"
    
    # Попробуем прочитать первые строки
    try:
        with open(file_path, "r", encoding="utf-8", errors="replace") as f:
            first_bytes = f.read(500)
            if first_bytes.strip().startswith("{"):
                return "telegram"
            return "whatsapp"
    except Exception:
        raise ValueError(f"Не удалось определить формат файла: {file_path}")


def validate_input(input_path: Path, strict: bool = False, auto_fix_flag: bool = False, verbose: bool = False) -> Optional[dict]:
    """
    Валидация входных данных перед импортом.

    Returns:
        dict с исправленными данными (если auto_fix) или None (если валидация прошла или не нужна)
    """
    fmt = detect_input_format(input_path)

    if fmt == "telegram":
        try:
            with open(input_path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except json.JSONDecodeError as e:
            print(f"❌ Невалидный JSON: {e}")
            return None

        report = validate_telegram_json(data, strict=strict)

        if auto_fix_flag and not report.is_valid:
            print("🔧 Применение авто-исправлений...")
            data = auto_fix(data, report)

        # Вывод отчёта — только если есть проблемы или verbose
        if not report.is_valid or verbose or report.warning_count > 0:
            report.source_file = str(input_path)
            print(f"🔍 Валидация входных данных ({fmt})...")
            print(report.summary())

        if not report.is_valid:
            if strict:
                print(f"\n❌ Валидация не пройдена (strict mode): {report.error_count} ошибок, {report.warning_count} предупреждений")
                return None
            elif report.error_count > 0 or report.critical_count > 0:
                print(f"\n❌ Валидация не пройдена: {report.critical_count} critical, {report.error_count} ошибок")
                return None

        return data if auto_fix_flag else None

    else:
        # WhatsApp
        report = validate_whatsapp_txt(input_path, strict=strict)

        if not report.is_valid or verbose or report.warning_count > 0:
            report.source_file = str(input_path)
            print(f"🔍 Валидация входных данных ({fmt})...")
            print(report.summary())

        if not report.is_valid:
            print(f"\n❌ Валидация не пройдена: {report.error_count} ошибок")
            return None

        return None


def validate_output_db(db_path: str) -> None:
    """Валидация SQLite БД после записи."""
    if Path(db_path).exists():
        print(f"\n🔍 Валидация SQLite БД: {db_path}")
        report = validate_sqlite_db(db_path)
        print(report.summary())
        if not report.is_valid:
            print(f"\n⚠️  Обнаружены проблемы в БД: {report.error_count} ошибок, {report.warning_count} предупреждений")


# ============================================================================
# Импорт данных
# ============================================================================

def import_data(input_path: Path, chat_name: Optional[str] = None, date_format: str = "%d/%m/%Y") -> dict:
    """Импорт данных из Telegram или WhatsApp экспорта."""
    fmt = detect_input_format(input_path)
    
    print(f"📂 Формат: {fmt}")
    
    if fmt == "telegram":
        return parse_telegram_export(str(input_path))
    else:
        return parse_whatsapp_chat(
            str(input_path),
            chat_name=chat_name,
            date_format_hint=date_format,
        )


# ============================================================================
# AI-перевод
# ============================================================================

def translate_data(data: dict, target_lang: str, api: str = "google",
                   model: Optional[str] = None) -> dict:
    """Перевод сообщений на целевой язык."""
    print(f"🌐 Перевод на {target_lang} (API: {api})...")
    
    translator = AITranslator(
        api=api,
        target_lang=target_lang,
        model=model,
        skip_if_target_lang=True,
    )
    
    # Прямой перевод без записи в файл
    messages = data.get("messages", [])
    for i, msg in enumerate(messages):
        original_text = msg.get("meta", {}).get("original_text", "")
        if not original_text:
            try:
                original_text = base64.b64decode(msg["encrypted_content"]).decode('utf-8')
            except Exception:
                original_text = ""
        
        if not original_text or not original_text.strip():
            continue
        
        # Детекция языка
        from ai_translator import detect_language
        detected_lang, confidence = detect_language(original_text)
        if detected_lang == target_lang:
            continue
        
        # Перевод
        try:
            translated = translator._translate_text(original_text)
            msg["encrypted_content"] = base64.b64encode(translated.encode()).decode('ascii')
            msg["meta"]["translation"] = {
                "target_lang": target_lang,
                "translated_text": translated,
                "original_text": original_text,
                "detected_source_lang": detected_lang,
            }
        except Exception as e:
            print(f"  ⚠️ Ошибка: {e}", file=sys.stderr)
        
        if (i + 1) % 50 == 0:
            print(f"  Обработано: {i + 1}/{len(messages)}")
    
    print(f"  ✅ Перевод завершён")
    return data


# ============================================================================
# Запись в SQLite (совместимо с Rust LocalStorage)
# ============================================================================

def write_to_sqlite(data: dict, db_path: str, dry_run: bool = False) -> dict:
    """
    Запись импортированных данных в SQLite.
    
    Схема таблиц полностью совместима с Rust `LocalStorage`:
    - contacts: контакты (пользователи)
    - chat_history: метаданные чатов
    - messages_cache: кэш сообщений
    - settings: настройки миграции
    """
    if dry_run:
        print("🔍 DRY RUN — запись в БД пропущена")
        return data
    
    print(f"💾 Запись в SQLite: {db_path}")
    
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA foreign_keys=ON")
    cursor = conn.cursor()
    
    try:
        # === Создание таблиц (совместимо с Rust миграциями) ===
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL UNIQUE,
                encrypted_display_name BLOB NOT NULL,
                encrypted_avatar TEXT,
                public_key_x25519 BLOB NOT NULL,
                public_key_kyber BLOB NOT NULL,
                public_key_ed25519 BLOB NOT NULL,
                encrypted_notes BLOB,
                added_at INTEGER NOT NULL,
                last_contacted INTEGER,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                is_favorite INTEGER NOT NULL DEFAULT 0
            )
        """)
        
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_contacts_user_id ON contacts(user_id)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_contacts_favorite ON contacts(is_favorite)")
        
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )
        """)
        
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS chat_history (
                id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL,
                chat_type TEXT NOT NULL,
                peer_id TEXT,
                encrypted_name BLOB,
                last_message_id TEXT,
                last_message_at INTEGER,
                unread_count INTEGER NOT NULL DEFAULT 0,
                is_archived INTEGER NOT NULL DEFAULT 0,
                is_muted INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
        """)
        
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_chat_history_updated ON chat_history(updated_at DESC)")
        
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS messages_cache (
                id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                encrypted_content BLOB NOT NULL,
                signature BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                msg_type TEXT NOT NULL,
                delivery_status TEXT NOT NULL,
                reply_to TEXT,
                attachments TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0
            )
        """)
        
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_messages_cache_chat ON messages_cache(chat_id, created_at DESC)")
        
        conn.commit()
        
        now = int(time.time())
        contacts_inserted = 0
        chats_inserted = 0
        messages_inserted = 0
        
        # === Импорт контактов (пользователей) ===
        users = data.get("users", [])
        if users:
            print(f"  Контакты: {len(users)}")
            stub_key = base64.b64decode("c3R1Yg==")  # b"stub" — заглушка для ключей
            
            for user in users:
                user_id = user["id"]
                display_name = user.get("display_name", "Unknown").encode('utf-8')
                
                cursor.execute("""
                    INSERT OR IGNORE INTO contacts
                    (id, user_id, encrypted_display_name, encrypted_avatar,
                     public_key_x25519, public_key_kyber, public_key_ed25519,
                     encrypted_notes, added_at, last_contacted, is_blocked, is_favorite)
                    VALUES (?, ?, ?, NULL, ?, ?, ?, NULL, ?, ?, 0, 0)
                """, (
                    user_id,
                    user_id,
                    display_name,
                    stub_key,    # TODO: реальный X25519 ключ
                    stub_key,    # TODO: реальный Kyber ключ
                    stub_key,    # TODO: реальный Ed25519 ключ
                    int(user.get("first_seen") or now),
                    int(user.get("last_seen") or now),
                ))
                contacts_inserted += 1
        
        # === Импорт чатов ===
        chats = data.get("chats", [])
        # WhatsApp: чат один, но нет списка chats — есть chat_id, chat_name, chat_type
        if not chats and "chat_id" in data:
            chats = [{
                "id": data["chat_id"],
                "name": data.get("chat_name", ""),
                "chat_type": data.get("chat_type", "direct"),
                "participants": [],
                "last_message_at": data.get("stats", {}).get("date_range", {}).get("last"),
            }]
        
        if chats:
            print(f"  Чаты: {len(chats)}")
            for chat in chats:
                chat_id = chat["id"]
                chat_name = chat.get("name", "").encode('utf-8')
                chat_type = chat.get("chat_type", "direct")
                last_message_at = chat.get("last_message_at")
                peer_id = chat.get("participants", [None])[0] if chat.get("participants") else None
                
                cursor.execute("""
                    INSERT OR IGNORE INTO chat_history
                    (id, chat_id, chat_type, peer_id, encrypted_name,
                     last_message_id, last_message_at, unread_count,
                     is_archived, is_muted, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, NULL, ?, 0, 0, 0, ?, ?)
                """, (
                    chat_id,
                    chat_id,
                    chat_type,
                    peer_id,
                    chat_name,
                    last_message_at,
                    int(last_message_at) if last_message_at else now,
                    now,
                ))
                chats_inserted += 1
        
        # === Импорт сообщений ===
        messages = data.get("messages", [])
        if messages:
            print(f"  Сообщения: {len(messages)}")
            
            # Батч-вставка для производительности
            batch = []
            for msg in messages:
                encrypted_content = base64.b64decode(msg["encrypted_content"])
                signature = base64.b64decode(msg["signature"])
                created_at = int(msg["created_at"])
                attachments = json.dumps(msg.get("attachments", []), ensure_ascii=False)
                reply_to = msg.get("reply_to")
                
                batch.append((
                    msg["id"],
                    msg["chat_id"],
                    msg["sender_id"],
                    encrypted_content,
                    signature,
                    created_at,
                    msg["msg_type"],
                    msg["delivery_status"],
                    reply_to,
                    attachments,
                    0,  # is_deleted
                ))
                
                # Вставка батчами по 500
                if len(batch) >= 500:
                    cursor.executemany("""
                        INSERT OR IGNORE INTO messages_cache
                        (id, chat_id, sender_id, encrypted_content, signature,
                         created_at, msg_type, delivery_status, reply_to, attachments,
                         is_deleted)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """, batch)
                    conn.commit()
                    messages_inserted += len(batch)
                    batch = []
            
            # Оставшиеся
            if batch:
                cursor.executemany("""
                    INSERT OR IGNORE INTO messages_cache
                    (id, chat_id, sender_id, encrypted_content, signature,
                     created_at, msg_type, delivery_status, reply_to, attachments,
                     is_deleted)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, batch)
                messages_inserted += len(batch)
        
        # === Сохранение метаданных миграции ===
        migration_meta = {
            "migrated_at": datetime.now(timezone.utc).isoformat(),
            "source": data.get("source", {}),
            "stats": data.get("stats", {}),
        }
        
        cursor.execute("""
            INSERT OR REPLACE INTO settings (key, value, updated_at)
            VALUES (?, ?, ?)
        """, ("migration_meta", json.dumps(migration_meta, ensure_ascii=False), now))
        
        cursor.execute("""
            INSERT OR REPLACE INTO settings (key, value, updated_at)
            VALUES (?, ?, ?)
        """, ("migration_version", "1.0", now))
        
        conn.commit()
        
        print(f"\n✅ Миграция завершена!")
        print(f"  БД: {db_path}")
        db_size = Path(db_path).stat().st_size / 1024
        print(f"  Размер: {db_size:.1f} KB")
        print(f"  Контактов: {contacts_inserted}")
        print(f"  Чатов: {chats_inserted}")
        print(f"  Сообщений: {messages_inserted}")
        
        data["migration"] = {
            "db_path": db_path,
            "db_size_bytes": Path(db_path).stat().st_size,
            "contacts_inserted": contacts_inserted,
            "chats_inserted": chats_inserted,
            "messages_inserted": messages_inserted,
        }
        
    except Exception as e:
        conn.rollback()
        raise
    finally:
        conn.close()
    
    return data


# ============================================================================
# CLI
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Migration CLI: импорт + перевод + запись в SQLite",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Примеры:
  python3 migrate.py --input telegram_export/result.json --output migrated.db
  python3 migrate.py --input whatsapp_chat.txt --output migrated.db --chat-name "Группа"
  python3 migrate.py --input export.json --output migrated.db --translate en
  python3 migrate.py --input export.json --dry-run
        """
    )
    
    parser.add_argument(
        "--input", "-i",
        required=True,
        help="Путь к файлу экспорта (Telegram JSON или WhatsApp TXT)"
    )
    parser.add_argument(
        "--output", "-o",
        default="migrated.db",
        help="Путь к выходной SQLite БД (по умолчанию: migrated.db)"
    )
    parser.add_argument(
        "--chat-name",
        default=None,
        help="Имя чата/группы (для WhatsApp)"
    )
    parser.add_argument(
        "--date-format",
        default="%d/%m/%Y",
        choices=["%d/%m/%Y", "%m/%d/%Y", "%Y/%m/%d", "%d-%m-%Y"],
        help="Формат даты в WhatsApp (по умолчанию: DD/MM/YYYY)"
    )
    parser.add_argument(
        "--translate", "-t",
        default=None,
        help="Код целевого языка для авто-перевода (en, de, fr, ...)"
    )
    parser.add_argument(
        "--translate-api",
        default="google",
        choices=["google", "openai", "ollama"],
        help="API для перевода (по умолчанию: google)"
    )
    parser.add_argument(
        "--translate-model",
        default=None,
        help="Модель для OpenAI/Ollama перевода"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Только статистика без записи в БД"
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Подробный вывод"
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        default=True,
        help="Валидация входных данных (по умолчанию: включена)"
    )
    parser.add_argument(
        "--no-validate",
        action="store_true",
        help="Пропустить валидацию"
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Строгий режим: предупреждения = ошибки"
    )
    parser.add_argument(
        "--fix",
        action="store_true",
        help="Авто-исправление типовых проблем"
    )
    parser.add_argument(
        "--validate-db",
        action="store_true",
        help="Валидация SQLite БД после записи"
    )
    parser.add_argument(
        "--report",
        default=None,
        help="Путь к файлу отчёта валидации (JSON)"
    )
    
    args = parser.parse_args()

    input_path = Path(args.input)
    if not input_path.exists():
        print(f"❌ Файл не найден: {input_path}", file=sys.stderr)
        sys.exit(1)

    skip_validate = args.no_validate

    try:
        # === Шаг 0: Валидация входных данных ===
        fixed_data = None
        if not skip_validate and not args.dry_run:
            fixed_data = validate_input(input_path, strict=args.strict, auto_fix_flag=args.fix, verbose=args.verbose)

            # Если авто-фикс применил данные — используем их напрямую
            if fixed_data is not None and args.fix:
                data = fixed_data
            elif not args.fix:
                # Валидация прошла — читаем данные заново
                pass

        # === Шаг 1: Импорт ===
        print("=" * 50)
        print("🚀 Migration Pipeline")
        print("=" * 50)
        print()

        if not skip_validate and args.fix and fixed_data is not None:
            data = fixed_data
            print("📦 Использованы данные с авто-исправлениями")
        else:
            data = import_data(input_path, args.chat_name, args.date_format)
        
        # === Шаг 2: Перевод (опционально) ===
        if args.translate:
            print()
            data = translate_data(
                data, args.translate,
                api=args.translate_api,
                model=args.translate_model,
            )
        
        # === Шаг 3: Запись в SQLite ===
        print()
        data = write_to_sqlite(data, args.output, dry_run=args.dry_run)
        
        # === Итоговая статистика ===
        print()
        print("=" * 50)
        print("📊 Итоговая статистика")
        print("=" * 50)
        stats = data.get("stats", {})
        print(f"  Источник: {data.get('source', {}).get('type', 'unknown')}")
        print(f"  Пользователей: {stats.get('total_users', 0)}")
        print(f"  Чатов: {stats.get('total_chats', 1)}")
        print(f"  Сообщений: {stats.get('total_messages', 0)}")
        print(f"  Медиа: {stats.get('total_media_attachments', 0)}")
        if stats.get('translation'):
            tr = stats['translation']
            print(f"  Переведено: {tr.get('translated_count', 0)} ({tr.get('target_lang')})")
        
        if args.dry_run:
            print()
            print("⚠️ DRY RUN — данные не записаны в БД")

        # === Шаг 4: Валидация БД (опционально) ===
        if args.validate_db and not args.dry_run:
            print()
            validate_output_db(args.output)

        # === Сохранение отчёта валидации ===
        if args.report:
            report_data = {
                "pipeline": "migration",
                "input_file": str(input_path),
                "output_db": args.output if not args.dry_run else None,
                "validation_performed": not skip_validate,
                "db_validation": args.validate_db and not args.dry_run,
            }
            with open(args.report, "w", encoding="utf-8") as f:
                json.dump(report_data, f, indent=2, ensure_ascii=False)
            print(f"\n📋 Отчёт сохранён: {args.report}")
        
    except Exception as e:
        print(f"❌ Ошибка миграции: {e}", file=sys.stderr)
        if args.verbose:
            import traceback
            traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
