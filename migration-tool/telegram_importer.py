#!/usr/bin/env python3
"""
Telegram Importer - парсинг Telegram JSON export во внутренний формат.

Конвертирует экспорт Telegram Desktop (result.json) во внутренний формат
мессенджера для последующей миграции данных.

Использование:
    python telegram_importer.py <path_to_result.json> [--output <output.json>]

Формат экспорта Telegram:
    - result.json с полями: name, type, id, messages[]
    - messages[] содержит: id, type, date, from, from_id, text, media и др.

Выходной формат:
    - users: список пользователей с ключами
    - chats: список чатов с метаданными
    - messages: список сообщений с привязкой к чатам
"""

import argparse
import base64
import hashlib
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional


# ============================================================================
# Внутренний формат данных
# ============================================================================

def generate_user_id(telegram_id: int | str) -> str:
    """Генерация внутреннего user_id из Telegram ID."""
    return f"tg_{telegram_id}"


def generate_message_id(chat_id: str, telegram_msg_id: int) -> str:
    """Генерация внутреннего message_id."""
    raw = f"{chat_id}:msg:{telegram_msg_id}"
    return hashlib.sha256(raw.encode()).hexdigest()[:16]


def generate_chat_id(telegram_chat_id: int | str, chat_type: str) -> str:
    """Генерация внутреннего chat_id."""
    raw = f"tg_chat:{telegram_chat_id}:{chat_type}"
    return hashlib.sha256(raw.encode()).hexdigest()[:16]


def parse_timestamp(date_str: str) -> Optional[int]:
    """Парсинг даты из Telegram формата в epoch seconds (i64 для Rust)."""
    if not date_str:
        return None
    
    try:
        dt = datetime.fromisoformat(date_str)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return int(dt.timestamp())
    except ValueError:
        return int(datetime.now(timezone.utc).timestamp())


def parse_text_field(text: Any) -> dict:
    """
    Парсинг текстового поля Telegram.
    
    Telegram text может быть:
    - строкой: "Hello"
    - массивом с форматированием: [{"type": "plain", "text": "Hello"}, {"type": "bold", "text": "world"}]
    """
    if isinstance(text, str):
        return {
            "content": text,
            "formatting": [],
        }
    
    if isinstance(text, list):
        parts = []
        formatting = []
        offset = 0
        
        for entity in text:
            if isinstance(entity, str):
                parts.append(entity)
                offset += len(entity)
            elif isinstance(entity, dict):
                entity_type = entity.get("type", "plain")
                entity_text = entity.get("text", "")
                entity_length = len(entity_text)
                
                parts.append(entity_text)
                formatting.append({
                    "type": entity_type,
                    "offset": offset,
                    "length": entity_length,
                })
                offset += entity_length
        
        return {
            "content": "".join(parts),
            "formatting": formatting,
        }
    
    return {"content": str(text), "formatting": []}


def map_chat_type(telegram_type: str) -> str:
    """Маппинг типов чатов Telegram во внутренний формат."""
    type_map = {
        "personal_chat": "direct",
        "private_chat": "direct",
        "private_group": "group",
        "private_supergroup": "group",
        "public_group": "group",
        "public_channel": "channel",
        "private_channel": "channel",
    }
    return type_map.get(telegram_type, "direct")


def map_message_type(msg: dict) -> str:
    """Определение типа сообщения."""
    if "photo" in msg:
        return "photo"
    if "video" in msg:
        return "video"
    if "document" in msg:
        return "document"
    if "sticker" in msg:
        return "sticker"
    if "voice_message" in msg:
        return "voice"
    if "video_message" in msg:
        return "video_note"
    if "animation" in msg:
        return "gif"
    if "audio" in msg:
        return "audio"
    if "location" in msg:
        return "location"
    if "contact" in msg:
        return "contact"
    if "poll" in msg:
        return "poll"
    if msg.get("type") == "service":
        return "service"
    return "text"


def extract_media_info(msg: dict) -> tuple[list[str], list[dict]]:
    """Извлечение информации о медиа-вложениях.
    
    Returns:
        (file_paths, media_details) — только пути для Rust, метаданные для meta
    """
    file_paths: list[str] = []
    media_details: list[dict] = []
    
    if "photo" in msg:
        photos = msg["photo"]
        if isinstance(photos, list):
            for photo in photos:
                fp = photo.get("file", "")
                file_paths.append(fp)
                media_details.append({
                    "type": "photo",
                    "file_name": photo.get("file_name", ""),
                    "file_path": fp,
                    "width": photo.get("width"),
                    "height": photo.get("height"),
                })
        elif isinstance(photos, dict):
            fp = photos.get("file", "")
            file_paths.append(fp)
            media_details.append({
                "type": "photo",
                "file_name": photos.get("file_name", ""),
                "file_path": fp,
                "width": photos.get("width"),
                "height": photos.get("height"),
            })
    
    if "video" in msg:
        video = msg["video"]
        fp = video.get("file", "")
        file_paths.append(fp)
        media_details.append({
            "type": "video",
            "file_name": video.get("file_name", ""),
            "file_path": fp,
            "width": video.get("width"),
            "height": video.get("height"),
            "duration_seconds": video.get("duration_seconds"),
        })
    
    if "document" in msg:
        doc = msg["document"]
        fp = doc.get("file", "")
        file_paths.append(fp)
        media_details.append({
            "type": "document",
            "file_name": doc.get("file_name", ""),
            "file_path": fp,
            "mime_type": doc.get("mime_type", ""),
            "duration_seconds": doc.get("duration_seconds"),
        })
    
    if "sticker" in msg:
        sticker = msg["sticker"]
        fp = sticker.get("file", "")
        file_paths.append(fp)
        media_details.append({
            "type": "sticker",
            "file_name": sticker.get("file_name", ""),
            "file_path": fp,
            "emoji": sticker.get("emoji"),
            "is_animated": sticker.get("is_animated", False),
            "is_video": sticker.get("is_video", False),
        })
    
    if "voice_message" in msg:
        voice = msg["voice_message"]
        fp = voice.get("file", "")
        file_paths.append(fp)
        media_details.append({
            "type": "voice",
            "file_name": voice.get("file_name", ""),
            "file_path": fp,
            "duration_seconds": voice.get("duration_seconds"),
        })
    
    if "video_message" in msg:
        video_msg = msg["video_message"]
        fp = video_msg.get("file", "")
        file_paths.append(fp)
        media_details.append({
            "type": "video_note",
            "file_name": video_msg.get("file_name", ""),
            "file_path": fp,
            "width": video_msg.get("width"),
            "height": video_msg.get("height"),
            "duration_seconds": video_msg.get("duration_seconds"),
        })
    
    if "animation" in msg:
        anim = msg["animation"]
        fp = anim.get("file", "")
        file_paths.append(fp)
        media_details.append({
            "type": "gif",
            "file_name": anim.get("file_name", ""),
            "file_path": fp,
            "width": anim.get("width"),
            "height": anim.get("height"),
            "duration_seconds": anim.get("duration_seconds"),
        })
    
    if "audio" in msg:
        audio = msg["audio"]
        fp = audio.get("file", "")
        file_paths.append(fp)
        media_details.append({
            "type": "audio",
            "file_name": audio.get("file_name", ""),
            "file_path": fp,
            "duration_seconds": audio.get("duration_seconds"),
        })
    
    if "location" in msg:
        loc = msg["location"]
        media_details.append({
            "type": "location",
            "latitude": loc.get("latitude"),
            "longitude": loc.get("longitude"),
            "address": loc.get("address"),
            "place": loc.get("place"),
        })
    
    if "contact" in msg:
        contact = msg["contact"]
        media_details.append({
            "type": "contact",
            "name": contact.get("name", ""),
            "phone_number": contact.get("phone_number", ""),
        })
    
    if "poll" in msg:
        poll = msg["poll"]
        media_details.append({
            "type": "poll",
            "question": poll.get("question", ""),
            "total_voters": poll.get("total_voters"),
            "closed": poll.get("closed", False),
            "options": poll.get("options", []),
        })
    
    return file_paths, media_details


def parse_service_message(msg: dict, chat_id: str) -> Optional[dict]:
    """Парсинг служебного сообщения → формат CachedMessage."""
    action = msg.get("action", "")
    sender_id = msg.get("from_id")
    sender_name = msg.get("from", "")
    created_at = parse_timestamp(msg.get("date", ""))

    return {
        "id": generate_message_id(chat_id, msg.get("id", 0)),
        "chat_id": chat_id,
        "sender_id": generate_user_id(sender_id) if sender_id else "system",
        "encrypted_content": base64.b64encode(b"").decode('ascii'),  # TODO: реальное шифрование (ChaCha20-Poly1305)
        "signature": base64.b64encode(b"stub").decode('ascii'),       # TODO: реальная подпись (ed25519-dalek)
        "created_at": created_at,
        "msg_type": "service",
        "delivery_status": "delivered",
        "reply_to": None,
        "attachments": [],
        "is_deleted": False,
        # Мета-данные (не в Rust-структуре, сохраняем отдельно)
        "meta": {
            "sender_name": sender_name,
            "service_data": {
                "action": action,
                "members": msg.get("members", []),
                "title": msg.get("title"),
            },
            "reactions": msg.get("reactions", []),
            "raw_telegram_id": msg.get("id"),
        },
    }


def parse_message(msg: dict, chat_id: str) -> Optional[dict]:
    """Парсинг одного сообщения → формат CachedMessage."""
    if "id" not in msg:
        return None

    msg_type = map_message_type(msg)

    if msg_type == "service":
        return parse_service_message(msg, chat_id)

    # Текст → заглушка шифрования
    text_data = parse_text_field(msg.get("text", ""))
    plaintext = text_data["content"]
    # TODO: реальное шифрование (ChaCha20-Poly1305)
    encrypted_content = base64.b64encode(plaintext.encode()).decode('ascii')

    # Медиа-вложения: только пути + метаданные в meta
    file_paths, media_details = extract_media_info(msg)

    # Reply
    reply_to_id = msg.get("reply_to_message_id")
    reply_to = None
    if reply_to_id:
        reply_to = generate_message_id(chat_id, reply_to_id)

    # Forward
    forward_from = None
    forward_date = None
    if msg.get("forward_from") or msg.get("forward_from_chat"):
        forward_from = msg.get("forward_from_name") or msg.get("forward_from_chat", "")
        if msg.get("forward_date"):
            forward_date = parse_timestamp(msg["forward_date"])

    # Edited
    edited_at = None
    if msg.get("edited"):
        edited_at = parse_timestamp(msg["edited"])

    created_at = parse_timestamp(msg.get("date", ""))

    return {
        "id": generate_message_id(chat_id, msg["id"]),
        "chat_id": chat_id,
        "sender_id": generate_user_id(msg.get("from_id", "unknown")),
        "encrypted_content": encrypted_content,     # TODO: реальное шифрование (ChaCha20-Poly1305)
        "signature": base64.b64encode(b"stub").decode('ascii'),  # TODO: реальная подпись (ed25519-dalek)
        "created_at": created_at,
        "msg_type": msg_type,
        "delivery_status": "delivered",
        "reply_to": reply_to,
        "attachments": file_paths,
        "is_deleted": False,
        # Мета-данные (не в Rust-структуре, сохраняем отдельно)
        "meta": {
            "sender_name": msg.get("from", ""),
            "formatting": text_data["formatting"],
            "edited_at": edited_at,
            "forward_from": forward_from,
            "forward_date": forward_date,
            "media_details": media_details,
            "reactions": msg.get("reactions", []),
            "raw_telegram_id": msg.get("id"),
        },
    }


def parse_chat(chat_data: dict) -> dict:
    """Парсинг одного чата."""
    chat_type = map_chat_type(chat_data.get("type", "personal_chat"))
    chat_id = generate_chat_id(
        chat_data.get("id", "unknown"),
        chat_data.get("type", "personal_chat")
    )
    
    messages = []
    users = set()
    last_message_at = None
    
    for msg in chat_data.get("messages", []):
        parsed = parse_message(msg, chat_id)
        if parsed:
            messages.append(parsed)
            users.add(parsed["sender_id"])
            if parsed["created_at"]:
                if last_message_at is None or parsed["created_at"] > last_message_at:
                    last_message_at = parsed["created_at"]
    
    return {
        "id": chat_id,
        "name": chat_data.get("name", ""),
        "chat_type": chat_type,
        "telegram_id": chat_data.get("id"),
        "telegram_type": chat_data.get("type", "personal_chat"),
        "participants": list(users),
        "messages_count": len(messages),
        "last_message_at": last_message_at,
        "messages": messages,
    }


def parse_telegram_export(input_path: str) -> dict:
    """
    Полный парсинг Telegram JSON export.
    
    Args:
        input_path: путь к result.json или директории с экспортом
    
    Returns:
        dict с полями:
        - version: версия формата
        - exported_at: время конвертации
        - source: информация об источнике
        - users: dict user_id -> user_info
        - chats: список чатов с сообщениями
        - stats: статистика
    """
    input_path = Path(input_path)
    
    # Определяем путь к result.json
    if input_path.is_file() and input_path.name == "result.json":
        result_json_path = input_path
    elif input_path.is_dir():
        result_json_path = input_path / "result.json"
        if not result_json_path.exists():
            raise FileNotFoundError(f"result.json not found in {input_path}")
    else:
        raise FileNotFoundError(f"File not found: {input_path}")
    
    print(f"Чтение {result_json_path}...")
    with open(result_json_path, "r", encoding="utf-8") as f:
        data = json.load(f)
    
    # Определяем формат экспорта
    # Формат 1: {"name": ..., "type": ..., "id": ..., "messages": [...]}
    # Формат 2: {"chats": {"list": [...]}}
    chats_data = []
    
    if "chats" in data and "list" in data["chats"]:
        # Формат с несколькими чатами
        chats_data = data["chats"]["list"]
        source_name = data.get("about", "")
    elif "messages" in data:
        # Один чат
        chats_data = [data]
        source_name = data.get("name", "")
    else:
        raise ValueError("Unknown Telegram export format")
    
    # Парсинг чатов
    chats = []
    all_users = {}
    total_messages = 0
    total_media = 0
    total_service = 0
    
    print(f"Найдено чатов: {len(chats_data)}")
    
    for i, chat_data in enumerate(chats_data):
        print(f"  [{i+1}/{len(chats_data)}] Парсинг чата: {chat_data.get('name', 'Unknown')}")
        chat = parse_chat(chat_data)
        chats.append(chat)
        
        # Сбор пользователей
        for participant_id in chat["participants"]:
            if participant_id not in all_users and participant_id != "system":
                all_users[participant_id] = {
                    "id": participant_id,
                    "display_name": "Unknown",
                    "telegram_id": participant_id.replace("tg_", ""),
                    "first_seen": None,
                    "last_seen": None,
                    "messages_count": 0,
                }
        
        # Обновление пользователей из сообщений
        for msg in chat["messages"]:
            sender_id = msg["sender_id"]
            
            # Подсчёт статистики (до фильтрации системных)
            total_messages += 1
            if msg["attachments"]:
                total_media += len(msg["attachments"])
            if msg["msg_type"] == "service":
                total_service += 1
            
            # Пропускаем системные сообщения для списка пользователей
            if sender_id == "system":
                continue
            
            if sender_id in all_users:
                user = all_users[sender_id]
                sender_name = msg.get("meta", {}).get("sender_name", "")
                if sender_name and sender_name != "Unknown":
                    user["display_name"] = sender_name

                msg_time = msg.get("created_at")
                if msg_time:
                    if user["first_seen"] is None or msg_time < user["first_seen"]:
                        user["first_seen"] = msg_time
                    if user["last_seen"] is None or msg_time > user["last_seen"]:
                        user["last_seen"] = msg_time

                user["messages_count"] += 1
    
    return {
        "version": "1.0",
        "exported_at": datetime.now(timezone.utc).isoformat(),
        "source": {
            "type": "telegram_desktop_export",
            "name": source_name,
            "file": str(result_json_path),
        },
        "users": list(all_users.values()),
        "chats": [
            {
                "id": c["id"],
                "name": c["name"],
                "chat_type": c["chat_type"],
                "telegram_id": c["telegram_id"],
                "telegram_type": c["telegram_type"],
                "participants": c["participants"],
                "messages_count": c["messages_count"],
                "last_message_at": c["last_message_at"],
            }
            for c in chats
        ],
        "messages": [
            msg for chat in chats for msg in chat["messages"]
        ],
        "stats": {
            "total_users": len(all_users),
            "total_chats": len(chats),
            "total_messages": total_messages,
            "total_media_attachments": total_media,
            "total_service_messages": total_service,
        },
    }


def main():
    parser = argparse.ArgumentParser(
        description="Telegram JSON export converter во внутренний формат"
    )
    parser.add_argument(
        "input",
        help="Путь к result.json или директории с экспортом Telegram"
    )
    parser.add_argument(
        "--output", "-o",
        default=None,
        help="Путь к выходному файлу (по умолчанию: internal_export.json)"
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Форматированный JSON вывод"
    )
    parser.add_argument(
        "--stats-only",
        action="store_true",
        help="Вывести только статисти без данных"
    )
    
    args = parser.parse_args()
    
    # Определяем выходной путь
    output_path = args.output
    if not output_path:
        output_path = "internal_export.json"
    
    try:
        result = parse_telegram_export(args.input)
        
        if args.stats_only:
            print("\n📊 Статистика экспорта:")
            print(f"  Пользователей: {result['stats']['total_users']}")
            print(f"  Чатов: {result['stats']['total_chats']}")
            print(f"  Сообщений: {result['stats']['total_messages']}")
            print(f"  Медиа-вложений: {result['stats']['total_media_attachments']}")
            print(f"  Служебных сообщений: {result['stats']['total_service_messages']}")
            return
        
        # Запись результата
        print(f"\nЗапись в {output_path}...")
        indent = 2 if args.pretty else None
        with open(output_path, "w", encoding="utf-8") as f:
            json.dump(result, f, indent=indent, ensure_ascii=False)
        
        print(f"✅ Конвертация завершена!")
        print(f"  Файл: {output_path}")
        print(f"  Размер: {os.path.getsize(output_path) / 1024:.1f} KB")
        print(f"  Пользователей: {result['stats']['total_users']}")
        print(f"  Чатов: {result['stats']['total_chats']}")
        print(f"  Сообщений: {result['stats']['total_messages']}")
        
    except FileNotFoundError as e:
        print(f"❌ Ошибка: {e}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"❌ Ошибка парсинга JSON: {e}", file=sys.stderr)
        sys.exit(1)
    except ValueError as e:
        print(f"❌ Ошибка формата: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"❌ Неизвестная ошибка: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
