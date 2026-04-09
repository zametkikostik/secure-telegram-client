#!/usr/bin/env python3
"""
WhatsApp Importer — парсинг WhatsApp TXT export во внутренний формат.

Конвертирует экспорт WhatsApp (_chat.txt) во внутренний формат мессенджера,
совместимый с Rust-структурой CachedMessage.

Использование:
    python3 whatsapp_importer.py <path_to_chat.txt> [--output <output.json>]

Формат WhatsApp TXT:
    - DD/MM/YYYY, HH:MM - Sender: message text
    - DD/MM/YYYY, HH:MM - <Media omitted>
    - DD/MM/YYYY, HH:MM - <attached: filename>
    - Системные сообщения: "You were added", "You left", etc.

Выходной формат:
    - messages: список в формате CachedMessage (id, chat_id, sender_id,
      encrypted_content, signature, created_at, msg_type, delivery_status,
      reply_to, attachments, is_deleted, meta)
"""

import argparse
import base64
import hashlib
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional


# ============================================================================
# Паттерны парсинга WhatsApp TXT
# ============================================================================

# Основной формат строки сообщения (поддержка 24h и 12h)
# DD/MM/YYYY, HH:MM - Sender: text
# MM/DD/YYYY, HH:MM - Sender: text (US формат)
LINE_PATTERN = re.compile(
    r'^'
    r'(?P<date>\d{1,2}[/\-.]\d{1,2}[/\-.]\d{2,4})'  # дата
    r'[,\s]+'
    r'(?P<time>\d{1,2}:\d{2}(?::\d{2})?(?:\s*[AaPp][Mm])?)'  # время
    r'\s*[-–—]\s*'
    r'(?P<sender>[^:]+?)'  # отправитель
    r':\s*'
    r'(?P<message>.*)'
    r'$',
    re.DOTALL
)

# Продолжение предыдущего сообщения (многострочные)
CONTINUATION_PATTERN = re.compile(r'^\s')

# Системные сообщения (без отправителя)
SYSTEM_PATTERNS = [
    re.compile(r'^(Messages? and calls? are end-to-end encrypted)', re.IGNORECASE),
    re.compile(r'^(You (were added|left|joined|created))', re.IGNORECASE),
    re.compile(r'^(\+?\d+ (was added|left|joined|removed))', re.IGNORECASE),
    re.compile(r'^("(?P<name>[^"]+)" (was added|left|joined|removed))', re.IGNORECASE),
    re.compile(r'^(You changed (the subject|the group description|the group icon))', re.IGNORECASE),
    re.compile(r'^("(?P<name>[^"]+)" changed (the subject|the group description|the group icon))', re.IGNORECASE),
    re.compile(r'^(This group has \d+ (members|participants))', re.IGNORECASE),
    re.compile(r'^(\+?\d+ added \+?\d+)', re.IGNORECASE),
    re.compile(r'^(Security code (with|of) .+ changed)', re.IGNORECASE),
    re.compile(r'^(Call duration: .+)', re.IGNORECASE),
]

# Медиа-плейсхолдеры
MEDIA_PATTERNS = {
    "voice": [
        re.compile(r'^<attached: .+\.opus>', re.IGNORECASE),
        re.compile(r'^voice message', re.IGNORECASE),
    ],
    "video": [
        re.compile(r'^<video (omitted|deleted)>?', re.IGNORECASE),
        re.compile(r'^<attached: .+\.(mp4|mov|avi|mkv|webm)>', re.IGNORECASE),
        re.compile(r'^video (omitted|deleted)', re.IGNORECASE),
    ],
    "audio": [
        re.compile(r'^<audio (omitted|deleted)>?', re.IGNORECASE),
        re.compile(r'^<attached: .+\.(mp3|m4a|wav)>', re.IGNORECASE),
        re.compile(r'^audio (omitted|deleted)', re.IGNORECASE),
    ],
    "photo": [
        re.compile(r'^<photo (deleted|omitted)>?', re.IGNORECASE),
        re.compile(r'^<attached: .+\.(jpg|jpeg|png|gif|webp)>', re.IGNORECASE),
        re.compile(r'^image (omitted|deleted)', re.IGNORECASE),
    ],
    "document": [
        re.compile(r'^<document (omitted|deleted)>?', re.IGNORECASE),
        re.compile(r'^<attached: .+\.(pdf|doc|docx|xls|xlsx|ppt|pptx|txt|zip|rar)>', re.IGNORECASE),
        re.compile(r'^document (omitted|deleted)', re.IGNORECASE),
    ],
    "sticker": [
        re.compile(r'^<sticker (omitted|deleted)>?', re.IGNORECASE),
        re.compile(r'^sticker (omitted|deleted)', re.IGNORECASE),
    ],
    "vcard": [
        re.compile(r'^<attached: .+\.vcf>', re.IGNORECASE),
        re.compile(r'^contact card', re.IGNORECASE),
    ],
    "location": [
        re.compile(r'^location (omitted|deleted|shared)', re.IGNORECASE),
    ],
}

# Имя файла из <attached: filename>
ATTACHED_FILE_PATTERN = re.compile(r'<attached:\s*(.+)>')

# Форматы дат WhatsApp
DATE_FORMATS = [
    "%d/%m/%Y",   # DD/MM/YYYY (европейский, по умолчанию)
    "%m/%d/%Y",   # MM/DD/YYYY (американский)
    "%d-%m-%Y",   # DD-MM-YYYY
    "%m-%d-%Y",   # MM-DD-YYYY
    "%d/%m/%y",   # DD/MM/YY
    "%m/%d/%y",   # MM/DD/YY
    "%Y/%m/%d",   # YYYY/MM/DD (ISO)
    "%Y-%m-%d",   # YYYY-MM-DD (ISO)
]

TIME_FORMATS = [
    "%H:%M",      # 24h: 14:30
    "%H:%M:%S",   # 24h с секундами: 14:30:00
    "%I:%M %p",   # 12h: 2:30 PM
    "%I:%M%p",    # 12h без пробела: 2:30PM
    "%I:%M:%S %p",  # 12h с секундами: 2:30:00 PM
]


# ============================================================================
# Генерация ID (совместимо с telegram_importer.py)
# ============================================================================

def generate_user_id(phone_or_name: str) -> str:
    """Генерация внутреннего user_id из номера телефона или имени."""
    return f"wa_{hashlib.sha256(phone_or_name.encode()).hexdigest()[:12]}"


def generate_message_id(chat_id: str, line_number: int) -> str:
    """Генерация внутреннего message_id."""
    raw = f"{chat_id}:msg:{line_number}"
    return hashlib.sha256(raw.encode()).hexdigest()[:16]


def generate_chat_id(phone_number: str, group_name: str) -> str:
    """Генерация внутреннего chat_id."""
    raw = f"wa_chat:{phone_number}:{group_name}"
    return hashlib.sha256(raw.encode()).hexdigest()[:16]


def parse_whatsapp_timestamp(date_str: str, time_str: str, date_format_hint: str = "%d/%m/%Y") -> Optional[int]:
    """
    Парсинг даты/времени WhatsApp в epoch seconds (i64 для Rust).
    
    Пробует несколько форматов, начиная с подсказки.
    """
    date_str = date_str.strip()
    time_str = time_str.strip()
    
    # Определяем формат даты
    date_formats_to_try = [date_format_hint] + [f for f in DATE_FORMATS if f != date_format_hint]
    
    for df in date_formats_to_try:
        for tf in TIME_FORMATS:
            try:
                dt = datetime.strptime(f"{date_str} {time_str}", f"{df} {tf}")
                dt = dt.replace(tzinfo=timezone.utc)
                return int(dt.timestamp())
            except ValueError:
                continue
    
    # Fallback: пробуем dateutil если доступен
    try:
        from dateutil import parser as dateutil_parser
        dt = dateutil_parser.parse(f"{date_str} {time_str}")
        dt = dt.replace(tzinfo=timezone.utc)
        return int(dt.timestamp())
    except (ImportError, ValueError):
        pass
    
    return int(datetime.now(timezone.utc).timestamp())


def detect_date_format(sample_line: str) -> str:
    """Определение формата даты по первой строке."""
    # Если день > 12 → европейский формат (DD/MM/YYYY)
    match = re.search(r'(\d{1,2})[/\-.](\d{1,2})[/\-.](\d{2,4})', sample_line)
    if match:
        first, second = int(match.group(1)), int(match.group(2))
        if first > 12:
            return "%d/%m/%Y" if "/" in match.group(0) else "%d-%m-%Y"
        if second > 12:
            return "%m/%d/%Y" if "/" in match.group(0) else "%m-%d-%Y"
    return "%d/%m/%Y"  # default


def is_system_message(message: str) -> bool:
    """Проверка, является ли сообщение системным."""
    for pattern in SYSTEM_PATTERNS:
        if pattern.match(message):
            return True
    return False


def detect_media_type(message: str) -> Optional[str]:
    """Определение типа медиа по тексту сообщения."""
    msg_stripped = message.strip()
    
    # Проверяем каждую строку (для многострочных сообщений)
    lines = msg_stripped.split("\n")
    for line in lines:
        line = line.strip()
        for media_type, patterns in MEDIA_PATTERNS.items():
            for pattern in patterns:
                if pattern.match(line):
                    return media_type
    
    return None


def extract_filename(message: str) -> Optional[str]:
    """Извлечение имени файла из <attached: filename>."""
    match = ATTACHED_FILE_PATTERN.search(message)
    if match:
        return match.group(1).strip()
    return None


def encrypt_stub(plaintext: str) -> str:
    """
    Заглушка шифрования: plaintext → base64.
    
    TODO: реальное шифрование (ChaCha20-Poly1305)
    """
    return base64.b64encode(plaintext.encode()).decode('ascii')


def signature_stub() -> str:
    """
    Заглушка подписи Ed25519.
    
    TODO: реальная подпись (ed25519-dalek)
    """
    return base64.b64encode(b"stub").decode('ascii')


# ============================================================================
# Парсинг сообщений
# ============================================================================

def parse_line(line: str) -> Optional[dict]:
    """
    Парсинг одной строки WhatsApp чата.
    
    Returns:
        dict с полями: date, time, sender, message
        или None если строка — продолжение предыдущего сообщения
    """
    match = LINE_PATTERN.match(line)
    if match:
        return {
            "date": match.group("date"),
            "time": match.group("time"),
            "sender": match.group("sender").strip(),
            "message": match.group("message").strip(),
        }
    return None


def parse_whatsapp_chat(
    file_path: str,
    chat_id: Optional[str] = None,
    chat_name: Optional[str] = None,
    date_format_hint: str = "%d/%m/%Y",
) -> dict:
    """
    Полный парсинг WhatsApp TXT export.
    
    Args:
        file_path: путь к _chat.txt
        chat_id: внутренний chat_id (если None — генерируется)
        chat_name: имя чата/группы
        date_format_hint: подсказка формата даты
    
    Returns:
        dict с полями:
        - chat_id: идентификатор чата
        - chat_name: имя чата
        - chat_type: direct | group
        - users: список пользователей
        - messages: список сообщений в формате CachedMessage
        - stats: статистика
    """
    file_path = Path(file_path)
    if not file_path.exists():
        raise FileNotFoundError(f"File not found: {file_path}")
    
    # Определяем chat_id
    if chat_id is None:
        chat_id = generate_chat_id("", chat_name or file_path.stem)
    
    print(f"Чтение {file_path}...")
    with open(file_path, "r", encoding="utf-8", errors="replace") as f:
        lines = f.readlines()
    
    if not lines:
        raise ValueError("Empty file")
    
    # Определяем формат даты по первой строке
    date_format = detect_date_format(lines[0])
    
    # Парсинг строк с объединением многострочных сообщений
    messages_raw: list[dict] = []
    current_msg: Optional[dict] = None
    line_number = 0
    
    for i, line in enumerate(lines):
        line = line.rstrip("\n\r")
        
        if not line:
            continue
        
        parsed = parse_line(line)
        
        if parsed:
            # Сохраняем предыдущее сообщение
            if current_msg:
                messages_raw.append(current_msg)
            
            line_number += 1
            current_msg = {
                "line_number": line_number,
                "date": parsed["date"],
                "time": parsed["time"],
                "sender": parsed["sender"],
                "message": parsed["message"],
            }
        elif current_msg:
            # Продолжение предыдущего сообщения (многострочное)
            current_msg["message"] += "\n" + line
        else:
            # Первая строка — системное сообщение без формата
            line_number += 1
            current_msg = {
                "line_number": line_number,
                "date": lines[0].split(",")[0] if "," in lines[0] else "01/01/2020",
                "time": "00:00",
                "sender": "system",
                "message": line,
            }
    
    # Последнее сообщение
    if current_msg:
        messages_raw.append(current_msg)
    
    print(f"Найдено сообщений: {len(messages_raw)}")
    
    # Конвертация во внутренний формат CachedMessage
    messages = []
    users: dict[str, dict] = {}
    total_media = 0
    total_service = 0
    
    for msg_raw in messages_raw:
        sender = msg_raw["sender"]
        message_text = msg_raw["message"]
        
        # Парсинг timestamp
        created_at = parse_whatsapp_timestamp(
            msg_raw["date"], msg_raw["time"], date_format
        )
        
        # Определяем тип сообщения
        is_system = sender == "system" or is_system_message(message_text)
        media_type = detect_media_type(message_text) if not is_system else None
        filename = extract_filename(message_text) if not is_system else None
        
        # sender_id
        if is_system:
            sender_id = "system"
        else:
            sender_id = generate_user_id(sender)
            if sender_id not in users:
                users[sender_id] = {
                    "id": sender_id,
                    "display_name": sender,
                    "phone_number": None,
                    "first_seen": created_at,
                    "last_seen": created_at,
                    "messages_count": 0,
                }
            user = users[sender_id]
            if user["first_seen"] is None or created_at < user["first_seen"]:
                user["first_seen"] = created_at
            if user["last_seen"] is None or created_at > user["last_seen"]:
                user["last_seen"] = created_at
            user["messages_count"] += 1
        
        # Контент → encrypted_content
        plaintext = message_text
        encrypted_content = encrypt_stub(plaintext)
        
        # Attachments → только пути (List[str])
        attachments: list[str] = []
        media_details: list[dict] = []
        
        if filename:
            attachments.append(filename)
            media_details.append({
                "type": media_type or "document",
                "file_name": filename,
                "file_path": filename,
            })
        elif media_type and media_type not in ("voice",):
            # Общие медиа без конкретного файла
            media_details.append({
                "type": media_type,
                "file_name": None,
                "file_path": None,
            })
        
        total_media += len(media_details)
        
        # Определяем msg_type
        if is_system:
            msg_type = "service"
            total_service += 1
            plaintext = ""  # Системные сообщения без content
            encrypted_content = encrypt_stub("")
        elif media_type == "voice":
            msg_type = "voice"
            if filename:
                attachments = [filename]
                media_details = [{
                    "type": "voice",
                    "file_name": filename,
                    "file_path": filename,
                }]
                total_media += 1
                total_service -= 0  # не сервис
        elif media_type:
            msg_type = media_type
        else:
            msg_type = "text"
        
        # Создаём CachedMessage
        cached_msg = {
            "id": generate_message_id(chat_id, msg_raw["line_number"]),
            "chat_id": chat_id,
            "sender_id": sender_id,
            "encrypted_content": encrypted_content,        # TODO: реальное шифрование (ChaCha20-Poly1305)
            "signature": signature_stub(),                 # TODO: реальная подпись (ed25519-dalek)
            "created_at": created_at,
            "msg_type": msg_type,
            "delivery_status": "delivered",
            "reply_to": None,                              # WhatsApp не экспортирует reply в TXT
            "attachments": attachments,
            "is_deleted": False,
            "meta": {
                "sender_name": sender,
                "original_text": message_text,
                "media_details": media_details,
                "raw_line_number": msg_raw["line_number"],
            },
        }
        
        if is_system:
            cached_msg["meta"]["service_action"] = message_text
        
        messages.append(cached_msg)
    
    # Определяем тип чата
    chat_type = "group" if len(users) > 2 else "direct"
    
    return {
        "version": "1.0",
        "exported_at": datetime.now(timezone.utc).isoformat(),
        "source": {
            "type": "whatsapp_txt_export",
            "name": chat_name or file_path.stem,
            "file": str(file_path),
        },
        "chat_id": chat_id,
        "chat_name": chat_name or file_path.stem,
        "chat_type": chat_type,
        "users": list(users.values()),
        "messages": messages,
        "stats": {
            "total_users": len(users),
            "total_messages": len(messages),
            "total_media_attachments": total_media,
            "total_service_messages": total_service,
            "date_range": {
                "first": messages[0]["created_at"] if messages else None,
                "last": messages[-1]["created_at"] if messages else None,
            },
        },
    }


def main():
    parser = argparse.ArgumentParser(
        description="WhatsApp TXT export converter во внутренний формат"
    )
    parser.add_argument(
        "input",
        help="Путь к файлу _chat.txt"
    )
    parser.add_argument(
        "--output", "-o",
        default=None,
        help="Путь к выходному файлу (по умолчанию: wa_internal_export.json)"
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
    parser.add_argument(
        "--chat-name",
        default=None,
        help="Имя чата/группы (если не определяется автоматически)"
    )
    parser.add_argument(
        "--date-format",
        default="%d/%m/%Y",
        choices=["%d/%m/%Y", "%m/%d/%Y", "%Y/%m/%d", "%d-%m-%Y"],
        help="Формат даты в файле (по умолчанию: DD/MM/YYYY)"
    )
    
    args = parser.parse_args()
    
    # Определяем выходной путь
    output_path = args.output or "wa_internal_export.json"
    
    try:
        result = parse_whatsapp_chat(
            args.input,
            chat_name=args.chat_name,
            date_format_hint=args.date_format,
        )
        
        if args.stats_only:
            print("\n📊 Статистика экспорта:")
            print(f"  Чат: {result['chat_name']}")
            print(f"  Тип: {result['chat_type']}")
            print(f"  Пользователей: {result['stats']['total_users']}")
            print(f"  Сообщений: {result['stats']['total_messages']}")
            print(f"  Медиа-вложений: {result['stats']['total_media_attachments']}")
            print(f"  Служебных сообщений: {result['stats']['total_service_messages']}")
            
            date_range = result['stats']['date_range']
            if date_range["first"]:
                first = datetime.fromtimestamp(date_range["first"], tz=timezone.utc)
                last = datetime.fromtimestamp(date_range["last"], tz=timezone.utc)
                print(f"  Период: {first.strftime('%Y-%m-%d')} — {last.strftime('%Y-%m-%d')}")
            return
        
        # Запись результата
        print(f"\nЗапись в {output_path}...")
        indent = 2 if args.pretty else None
        with open(output_path, "w", encoding="utf-8") as f:
            json.dump(result, f, indent=indent, ensure_ascii=False)
        
        print(f"✅ Конвертация завершена!")
        print(f"  Файл: {output_path}")
        print(f"  Размер: {os.path.getsize(output_path) / 1024:.1f} KB")
        print(f"  Чат: {result['chat_name']} ({result['chat_type']})")
        print(f"  Пользователей: {result['stats']['total_users']}")
        print(f"  Сообщений: {result['stats']['total_messages']}")
        print(f"  Медиа: {result['stats']['total_media_attachments']}")
        
    except FileNotFoundError as e:
        print(f"❌ Ошибка: {e}", file=sys.stderr)
        sys.exit(1)
    except ValueError as e:
        print(f"❌ Ошибка формата: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"❌ Неизвестная ошибка: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
