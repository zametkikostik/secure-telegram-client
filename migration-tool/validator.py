#!/usr/bin/env python3
"""
Валидация входных данных для миграции.

Модуль обеспечивает multi-layer валидацию:
1. Schema validation — проверка структуры и типов JSON/TXT
2. Business rules — дубликаты, orphan ссылки, целостность связей
3. Content validation — диапазоны дат, допустимые символы, размеры
4. SQLite integrity — проверка БД после записи

Использование:
    # Standalone валидация файла
    python3 validator.py --input result.json

    # Валидация с авто-исправлением
    python3 validator.py --input result.json --fix

    # Валидация SQLite БД
    python3 validator.py --input migrated.db --db-check

    # Строгий режим (предупреждения = ошибки)
    python3 validator.py --input result.json --strict
"""

import json
import re
import sqlite3
import sys
from collections import Counter
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Any, Optional


# ============================================================================
# Severity levels
# ============================================================================

class Severity(str, Enum):
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    CRITICAL = "critical"


@dataclass
class ValidationResult:
    """Один результат проверки."""
    rule: str
    severity: Severity
    message: str
    context: dict[str, Any] = field(default_factory=dict)
    suggestion: str = ""
    auto_fixable: bool = False

    def __str__(self) -> str:
        icon = {
            Severity.INFO: "ℹ️",
            Severity.WARNING: "⚠️",
            Severity.ERROR: "❌",
            Severity.CRITICAL: "🔥",
        }.get(self.severity, "?")
        return f"{icon} [{self.rule}] {self.message}"


@dataclass
class ValidationReport:
    """Полный отчёт валидации."""
    results: list[ValidationResult] = field(default_factory=list)
    start_time: float = 0.0
    end_time: float = 0.0
    source_file: str = ""
    source_format: str = ""

    @property
    def is_valid(self) -> bool:
        return not any(
            r.severity in (Severity.ERROR, Severity.CRITICAL)
            for r in self.results
        )

    @property
    def critical_count(self) -> int:
        return sum(1 for r in self.results if r.severity == Severity.CRITICAL)

    @property
    def error_count(self) -> int:
        return sum(1 for r in self.results if r.severity == Severity.ERROR)

    @property
    def warning_count(self) -> int:
        return sum(1 for r in self.results if r.severity == Severity.WARNING)

    @property
    def info_count(self) -> int:
        return sum(1 for r in self.results if r.severity == Severity.INFO)

    @property
    def auto_fixable_count(self) -> int:
        return sum(1 for r in self.results if r.auto_fixable)

    def add(self, rule: str, severity: Severity, message: str,
            context: dict | None = None, suggestion: str = "",
            auto_fixable: bool = False) -> None:
        self.results.append(ValidationResult(
            rule=rule, severity=severity, message=message,
            context=context or {}, suggestion=suggestion,
            auto_fixable=auto_fixable,
        ))

    def summary(self) -> str:
        lines = [
            "=" * 60,
            f"📋 Validation Report: {self.source_file}",
            f"   Format: {self.source_format}",
            f"   Duration: {self.end_time - self.start_time:.2f}s",
            "=" * 60,
            f"  ✅ Valid: {self.is_valid}",
            f"  🔥 Critical: {self.critical_count}",
            f"  ❌ Errors: {self.error_count}",
            f"  ⚠️  Warnings: {self.warning_count}",
            f"  ℹ️  Info: {self.info_count}",
            f"  🔧 Auto-fixable: {self.auto_fixable_count}",
            "=" * 60,
        ]

        # Группировка по severity
        for sev in [Severity.CRITICAL, Severity.ERROR, Severity.WARNING, Severity.INFO]:
            items = [r for r in self.results if r.severity == sev]
            if items:
                lines.append(f"\n--- {sev.value.upper()} ({len(items)}) ---")
                for r in items:
                    lines.append(f"  {r}")
                    if r.suggestion:
                        lines.append(f"    💡 {r.suggestion}")
                    if r.auto_fixable:
                        lines.append(f"    🔧 Auto-fixable")

        return "\n".join(lines)

    def to_dict(self) -> dict:
        return {
            "source_file": self.source_file,
            "source_format": self.source_format,
            "is_valid": self.is_valid,
            "duration_seconds": round(self.end_time - self.start_time, 2),
            "counts": {
                "critical": self.critical_count,
                "error": self.error_count,
                "warning": self.warning_count,
                "info": self.info_count,
                "auto_fixable": self.auto_fixable_count,
            },
            "results": [
                {
                    "rule": r.rule,
                    "severity": r.severity.value,
                    "message": r.message,
                    "context": r.context,
                    "suggestion": r.suggestion,
                    "auto_fixable": r.auto_fixable,
                }
                for r in self.results
            ],
        }


# ============================================================================
# Constants
# ============================================================================

# Допустимые значения полей
VALID_CHAT_TYPES = {"direct", "group", "channel"}
VALID_MSG_TYPES = {
    "text", "photo", "video", "document", "voice", "video_note",
    "gif", "sticker", "audio", "location", "contact", "poll",
    "service",
}
VALID_DELIVERY_STATUSES = {"sent", "delivered", "read", "failed", "pending"}
VALID_TELEGRAM_CHAT_TYPES = {
    "personal_chat", "private_chat", "private_group", "private_supergroup",
    "public_group", "public_channel", "private_channel",
}
VALID_MSG_FORMATTING_TYPES = {
    "plain", "bold", "italic", "code", "pre", "link", "mention",
    "underline", "strikethrough", "spoiler",
}

# Диапазоны
MIN_TIMESTAMP = 946684800       # 2000-01-01
MAX_TIMESTAMP = 2524608000      # 2050-01-01
MAX_MESSAGE_LENGTH = 4096       # Telegram limit
MAX_ATTACHMENTS_PER_MESSAGE = 10
MAX_USERS_PER_EXPORT = 100000
MAX_MESSAGES_PER_EXPORT = 5000000
MAX_CHAT_NAME_LENGTH = 255
MAX_DISPLAY_NAME_LENGTH = 255

# Regex паттерны
TIMESTAMP_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}")
WHATSAPP_LINE_RE = re.compile(
    r"^(\[?\d{1,4}[-/.]\d{1,2}[-/.]\d{1,4}[, ]+\d{1,2}:\d{2}(?::\d{2})?(?:\s*[AP]M)?\]?)\s*-?\s*(.*)"
)
SAFE_FILENAME_RE = re.compile(r"^[\w\s\-\.()\[\]]+$")


# ============================================================================
# Schema Validators
# ============================================================================

class TelegramSchemaValidator:
    """Валидация схемы Telegram Desktop JSON export."""

    def validate(self, data: Any, report: ValidationReport) -> None:
        if not isinstance(data, dict):
            report.add(
                "schema.root_type", Severity.CRITICAL,
                "Root element is not a JSON object",
                suggestion="File must be a valid JSON object {}",
            )
            return

        self._check_format_variant(data, report)

        # Определяем формат: single-chat или multi-chat
        has_messages = "messages" in data
        has_chats = "chats" in data

        if has_messages and not has_chats:
            # Single-chat format
            self._validate_single_chat(data, report)
        elif has_chats and not has_messages:
            # Multi-chat format
            self._validate_multi_chat(data, report)
        elif has_messages and has_chats:
            # Оба поля — приоритет messages
            self._validate_single_chat(data, report)
        else:
            # Ни одного из полей — проверяем required
            report.add(
                "schema.unknown_format", Severity.CRITICAL,
                "Unrecognized Telegram export format — neither 'messages' nor 'chats.list' found",
                suggestion="Export via Telegram Desktop: Settings → Advanced → Export Telegram Data → JSON",
            )
            for fld in ("name", "type", "id", "messages"):
                if fld not in data:
                    report.add(
                        f"schema.required_field", Severity.ERROR,
                        f"Missing required field: '{fld}'",
                        context={"field": fld},
                    )

    def _check_format_variant(self, data: dict, report: ValidationReport) -> None:
        report.source_format = "telegram_desktop_json"

    def _validate_single_chat(self, data: dict, report: ValidationReport) -> None:
        # Required fields
        for fld in ("name", "type", "id", "messages"):
            if fld not in data:
                report.add(
                    f"schema.required_field", Severity.ERROR,
                    f"Missing required field: '{fld}'",
                    context={"field": fld},
                    suggestion=f"Add '{fld}' to the root object",
                )

        # Field types
        if "name" in data and not isinstance(data["name"], str):
            report.add("schema.name_type", Severity.WARNING,
                       f"'name' should be string, got {type(data['name']).__name__}",
                       context={"value_type": type(data["name"]).__name__})

        if "type" in data and data["type"] not in VALID_TELEGRAM_CHAT_TYPES:
            report.add("schema.chat_type", Severity.WARNING,
                       f"Unknown chat type: '{data['type']}'",
                       context={"value": data["type"], "valid": list(VALID_TELEGRAM_CHAT_TYPES)},
                       suggestion="Will be mapped to 'direct' by default",
                       auto_fixable=True)

        if "id" in data and not isinstance(data["id"], (int, str)):
            report.add("schema.id_type", Severity.ERROR,
                       f"'id' should be int or string, got {type(data['id']).__name__}",
                       context={"value_type": type(data["id"]).__name__})

        if "messages" in data:
            if not isinstance(data["messages"], list):
                report.add("schema.messages_type", Severity.CRITICAL,
                           "'messages' must be an array",
                           suggestion="Expected: \"messages\": []")
            else:
                self._validate_messages_list(data["messages"], report)

        # Optional fields type check
        if "about" in data and not isinstance(data["about"], str):
            report.add("schema.about_type", Severity.INFO,
                       f"'about' should be string")

    def _validate_multi_chat(self, data: dict, report: ValidationReport) -> None:
        if not isinstance(data.get("chats"), dict):
            report.add("schema.chats_type", Severity.CRITICAL,
                       "'chats' must be an object")
            return

        if "list" not in data["chats"]:
            report.add("schema.chats.list_missing", Severity.CRITICAL,
                       "'chats.list' array not found",
                       suggestion="Multi-chat format requires 'chats': {'list': [...]}")
            return

        if not isinstance(data["chats"]["list"], list):
            report.add("schema.chats.list_type", Severity.CRITICAL,
                       "'chats.list' must be an array")
            return

        for i, chat in enumerate(data["chats"]["list"]):
            prefix = f"schema.chats[{i}]"
            if not isinstance(chat, dict):
                report.add(prefix + ".type", Severity.ERROR,
                           f"Chat at index {i} is not an object")
                continue
            self._validate_single_chat(chat, report)

    def _validate_messages_list(self, messages: list, report: ValidationReport) -> None:
        if len(messages) > MAX_MESSAGES_PER_EXPORT:
            report.add("schema.messages_limit", Severity.WARNING,
                       f"Message count {len(messages)} exceeds recommended limit {MAX_MESSAGES_PER_EXPORT}",
                       context={"count": len(messages), "limit": MAX_MESSAGES_PER_EXPORT})

        for i, msg in enumerate(messages):
            prefix = f"schema.msg[{i}]"
            if not isinstance(msg, dict):
                report.add(prefix + ".type", Severity.ERROR,
                           f"Message at index {i} is not an object")
                continue

            # Required fields
            if "id" not in msg:
                report.add(prefix + ".missing_id", Severity.ERROR,
                           "Message missing 'id' field",
                           suggestion="Every message must have a unique 'id'")

            if "date" not in msg:
                report.add(prefix + ".missing_date", Severity.WARNING,
                           "Message missing 'date' field — will use current time",
                           auto_fixable=True)

            # Type checks
            if "id" in msg and not isinstance(msg["id"], int):
                report.add(prefix + ".id_type", Severity.ERROR,
                           f"Message 'id' should be integer, got {type(msg['id']).__name__}")

            if "date" in msg and not isinstance(msg["date"], str):
                report.add(prefix + ".date_type", Severity.ERROR,
                           f"Message 'date' should be ISO 8601 string")

            if "date" in msg and isinstance(msg["date"], str):
                if not TIMESTAMP_RE.match(msg["date"]):
                    report.add(prefix + ".date_format", Severity.WARNING,
                               f"Suspicious date format: '{msg['date']}'",
                               context={"value": msg["date"]},
                               suggestion="Expected: YYYY-MM-DDTHH:MM:SS")

            if "type" in msg and msg["type"] not in ("message", "service"):
                report.add(prefix + ".msg_type_unknown", Severity.INFO,
                           f"Unknown message type: '{msg.get('type')}'")

            # Text field validation
            if "text" in msg:
                text = msg["text"]
                if isinstance(text, str):
                    if len(text) > MAX_MESSAGE_LENGTH * 2:  # formatted can be longer
                        report.add(prefix + ".text_length", Severity.WARNING,
                                   f"Text length {len(text)} exceeds typical limit {MAX_MESSAGE_LENGTH}")
                elif isinstance(text, list):
                    for j, entity in enumerate(text):
                        if isinstance(entity, dict):
                            if "type" in entity and entity["type"] not in VALID_MSG_FORMATTING_TYPES:
                                report.add(prefix + f".text[{j}].format_type", Severity.WARNING,
                                           f"Unknown formatting type: '{entity['type']}'",
                                           context={"valid": list(VALID_MSG_FORMATTING_TYPES)})
                            if "text" not in entity:
                                report.add(prefix + f".text[{j}].missing_text", Severity.WARNING,
                                           f"Formatting entity missing 'text' field")
                elif not isinstance(text, (str, list)):
                    report.add(prefix + ".text_type", Severity.ERROR,
                               f"'text' should be string or array, got {type(text).__name__}")

            # Media validation
            media_fields = {"photo", "video", "document", "voice_message",
                           "video_message", "animation", "audio", "sticker"}
            for mf in media_fields:
                if mf in msg and not isinstance(msg[mf], (dict, list)):
                    report.add(prefix + f".{mf}_type", Severity.WARNING,
                               f"'{mf}' should be object or array")

            # from_id validation
            if "from_id" in msg:
                fid = msg["from_id"]
                if not isinstance(fid, (int, str)):
                    report.add(prefix + ".from_id_type", Severity.ERROR,
                               f"'from_id' should be int or string")
                elif isinstance(fid, str) and not fid.startswith("user") and not fid.isdigit():
                    report.add(prefix + ".from_id_format", Severity.INFO,
                               f"Unusual from_id format: '{fid}'")


class WhatsAppSchemaValidator:
    """Валидация WhatsApp TXT export."""

    def validate(self, file_path: Path, report: ValidationReport) -> None:
        report.source_format = "whatsapp_txt"

        if not file_path.exists():
            report.add("schema.file_missing", Severity.CRITICAL,
                       f"File not found: {file_path}")
            return

        file_size = file_path.stat().st_size
        if file_size == 0:
            report.add("schema.file_empty", Severity.CRITICAL,
                       "File is empty")
            return

        if file_size > 100 * 1024 * 1024:  # 100MB
            size_mb = file_size / 1024 / 1024
            report.add("schema.file_too_large", Severity.WARNING,
                       f"File size {size_mb:.1f}MB is very large — may be slow",
                       context={"size_mb": size_mb})

        try:
            content = file_path.read_text(encoding="utf-8", errors="replace")
        except Exception as e:
            report.add("schema.file_read_error", Severity.CRITICAL,
                       f"Cannot read file: {e}")
            return

        lines = content.splitlines()
        report.add("schema.line_count", Severity.INFO,
                   f"Total lines: {len(lines)}",
                   context={"lines": len(lines)})

        parsed_lines = 0
        unparsed_lines = 0
        date_formats_found: Counter = Counter()

        for i, line in enumerate(lines):
            stripped = line.strip()
            if not stripped:
                continue

            match = WHATSAPP_LINE_RE.match(stripped)
            if match:
                parsed_lines += 1
                date_str = match.group(1).strip("[]")
                # Определяем формат даты
                if re.match(r"\d{4}[-/.]", date_str):
                    date_formats_found["YYYY-MM-DD"] += 1
                elif re.match(r"\d{1,2}/\d{1,2}/\d{2,4}", date_str):
                    date_formats_found["DD/MM/YYYY or MM/DD/YYYY"] += 1
                elif re.match(r"\d{1,2}-\d{1,2}-\d{2,4}", date_str):
                    date_formats_found["DD-MM-YYYY or MM-DD-YYYY"] += 1
            else:
                unparsed_lines += 1
                if unparsed_lines <= 5:  # Покажем только первые 5
                    report.add("schema.line_unparsed", Severity.WARNING,
                               f"Line {i+1} doesn't match expected format",
                               context={"line_num": i+1, "preview": stripped[:100]},
                               suggestion="This may be a continuation of a multi-line message")

        if parsed_lines == 0:
            report.add("schema.no_valid_lines", Severity.CRITICAL,
                       "No valid WhatsApp-formatted lines found",
                       suggestion="Check file format: expected 'DD/MM/YYYY, HH:MM - Name: message'")
        else:
            report.add("schema.parse_rate", Severity.INFO,
                       f"Parsed {parsed_lines}/{parsed_lines + unparsed_lines} lines ({parsed_lines/(parsed_lines+unparsed_lines)*100:.1f}%)",
                       context={"parsed": parsed_lines, "unparsed": unparsed_lines})

        if date_formats_found:
            most_common = date_formats_found.most_common(1)[0]
            report.add("schema.date_format_detected", Severity.INFO,
                       f"Detected date format: {most_common[0]}",
                       context={"formats": dict(date_formats_found)})


# ============================================================================
# Business Rules Validator
# ============================================================================

class BusinessRulesValidator:
    """Валидация бизнес-правил импортированных данных."""

    def validate(self, data: dict, report: ValidationReport) -> None:
        self._validate_users(data, report)
        self._validate_chats(data, report)
        self._validate_messages(data, report)
        self._validate_relationships(data, report)
        self._validate_content(data, report)

    def _validate_users(self, data: dict, report: ValidationReport) -> None:
        users = data.get("users", [])
        if not users:
            report.add("biz.no_users", Severity.WARNING,
                       "No users found in export",
                       suggestion="Messages will have 'unknown' sender")
            return

        if len(users) > MAX_USERS_PER_EXPORT:
            report.add("biz.users_limit", Severity.WARNING,
                       f"User count {len(users)} exceeds recommended limit",
                       context={"count": len(users), "limit": MAX_USERS_PER_EXPORT})

        # Duplicate user IDs
        user_ids = [u.get("id") for u in users if u.get("id")]
        duplicates = [uid for uid, count in Counter(user_ids).items() if count > 1]
        if duplicates:
            report.add("biz.duplicate_users", Severity.ERROR,
                       f"Duplicate user IDs found: {len(duplicates)}",
                       context={"duplicates": duplicates[:10]},
                       suggestion="Each user must have a unique ID")

        # Empty display names
        for user in users:
            uid = user.get("id", "?")
            name = user.get("display_name", "").strip()
            if not name or name == "Unknown":
                report.add("biz.user_empty_name", Severity.WARNING,
                           f"User '{uid}' has empty/default name",
                           context={"user_id": uid},
                           auto_fixable=True)

    def _validate_chats(self, data: dict, report: ValidationReport) -> None:
        chats = data.get("chats", [])
        if not chats:
            report.add("biz.no_chats", Severity.WARNING,
                       "No chats found — checking for single-chat format")
            if "chat_id" in data:
                report.add("biz.single_chat", Severity.INFO,
                           "Single-chat format detected")
            return

        # Duplicate chat IDs
        chat_ids = [c.get("id") for c in chats if c.get("id")]
        duplicates = [cid for cid, count in Counter(chat_ids).items() if count > 1]
        if duplicates:
            report.add("biz.duplicate_chats", Severity.ERROR,
                       f"Duplicate chat IDs: {len(duplicates)}",
                       context={"duplicates": duplicates[:5]})

        # Invalid chat types
        for chat in chats:
            cid = chat.get("id", "?")
            ctype = chat.get("chat_type", "")
            if ctype and ctype not in VALID_CHAT_TYPES:
                report.add("biz.invalid_chat_type", Severity.WARNING,
                           f"Chat '{cid}' has invalid type: '{ctype}'",
                           context={"chat_id": cid, "value": ctype, "valid": list(VALID_CHAT_TYPES)},
                           auto_fixable=True)

            # Long names
            name = chat.get("name", "")
            if len(name) > MAX_CHAT_NAME_LENGTH:
                report.add("biz.chat_name_too_long", Severity.WARNING,
                           f"Chat name exceeds {MAX_CHAT_NAME_LENGTH} chars ({len(name)})",
                           context={"chat_id": cid, "length": len(name)},
                           auto_fixable=True)

    def _validate_messages(self, data: dict, report: ValidationReport) -> None:
        messages = data.get("messages", [])
        if not messages:
            report.add("biz.no_messages", Severity.WARNING,
                       "No messages found in export")
            return

        # Duplicate message IDs
        msg_ids = [m.get("id") for m in messages if m.get("id")]
        duplicates = [mid for mid, count in Counter(msg_ids).items() if count > 1]
        if duplicates:
            report.add("biz.duplicate_messages", Severity.ERROR,
                       f"Duplicate message IDs: {len(duplicates)}",
                       context={"duplicates": duplicates[:5]},
                       suggestion="Duplicate IDs will cause data loss on import")

        # Invalid msg_type
        for msg in messages:
            mid = msg.get("id", "?")
            mtype = msg.get("msg_type", "")
            if mtype and mtype not in VALID_MSG_TYPES:
                report.add("biz.invalid_msg_type", Severity.WARNING,
                           f"Message '{mid}' has unknown type: '{mtype}'",
                           context={"msg_id": mid, "value": mtype})

            # Invalid delivery_status
            ds = msg.get("delivery_status", "")
            if ds and ds not in VALID_DELIVERY_STATUSES:
                report.add("biz.invalid_delivery_status", Severity.INFO,
                           f"Message '{mid}' has unknown status: '{ds}'",
                           context={"msg_id": mid, "value": ds})

    def _validate_relationships(self, data: dict, report: ValidationReport) -> None:
        messages = data.get("messages", [])
        user_ids = {u.get("id") for u in data.get("users", []) if u.get("id")}
        chat_ids = {c.get("id") for c in data.get("chats", []) if c.get("id")}
        if "chat_id" in data:
            chat_ids.add(data["chat_id"])

        msg_ids = {m.get("id") for m in messages if m.get("id")}

        orphan_chats = 0
        orphan_senders = 0
        orphan_replies = 0
        unknown_chat_refs = 0

        for msg in messages:
            mid = msg.get("id", "?")
            chat_id = msg.get("chat_id", "")
            sender_id = msg.get("sender_id", "")
            reply_to = msg.get("reply_to")

            if chat_id and chat_id not in chat_ids:
                unknown_chat_refs += 1
                if unknown_chat_refs <= 3:
                    report.add("biz.orphan_chat_ref", Severity.WARNING,
                               f"Message '{mid}' references unknown chat '{chat_id}'",
                               context={"msg_id": mid, "chat_id": chat_id},
                               suggestion="Chat may have been filtered out during import")

            if sender_id and sender_id != "system" and sender_id not in user_ids:
                orphan_senders += 1
                if orphan_senders <= 3:
                    report.add("biz.orphan_sender", Severity.INFO,
                               f"Message '{mid}' has unknown sender '{sender_id}'",
                               context={"msg_id": mid, "sender_id": sender_id})

            if reply_to and reply_to not in msg_ids:
                orphan_replies += 1
                if orphan_replies <= 3:
                    report.add("biz.orphan_reply", Severity.WARNING,
                               f"Message '{mid}' replies to non-existent message '{reply_to}'",
                               context={"msg_id": mid, "reply_to": reply_to},
                               suggestion="Original message may have been deleted or not exported")

        if orphan_senders > 3:
            report.add("biz.orphan_senders_summary", Severity.INFO,
                       f"Total orphan senders: {orphan_senders} (first 3 shown)")
        if orphan_replies > 3:
            report.add("biz.orphan_replies_summary", Severity.WARNING,
                       f"Total orphan replies: {orphan_replies} (first 3 shown)")

    def _validate_content(self, data: dict, report: ValidationReport) -> None:
        messages = data.get("messages", [])

        empty_content = 0
        future_dates = 0
        ancient_dates = 0
        large_messages = 0

        now_ts = datetime.now(timezone.utc).timestamp()

        for msg in messages:
            mid = msg.get("id", "?")
            created_at = msg.get("created_at")

            if created_at:
                try:
                    ts = int(created_at)
                    if ts > now_ts + 86400:  # +1 day tolerance
                        future_dates += 1
                    if ts < MIN_TIMESTAMP:
                        ancient_dates += 1
                except (ValueError, TypeError):
                    report.add("biz.invalid_timestamp", Severity.ERROR,
                               f"Message '{mid}' has non-numeric timestamp: '{created_at}'",
                               context={"msg_id": mid, "value": str(created_at)})

            # Content size check
            enc = msg.get("encrypted_content", "")
            if isinstance(enc, str):
                import base64
                try:
                    decoded = base64.b64decode(enc)
                    if len(decoded) > MAX_MESSAGE_LENGTH * 4:
                        large_messages += 1
                except Exception:
                    report.add("biz.invalid_base64", Severity.WARNING,
                               f"Message '{mid}' has invalid base64 content",
                               context={"msg_id": mid})

        if future_dates > 0:
            report.add("biz.future_dates", Severity.WARNING,
                       f"{future_dates} messages have future dates",
                       context={"count": future_dates})
        if ancient_dates > 0:
            report.add("biz.ancient_dates", Severity.WARNING,
                       f"{ancient_dates} messages have dates before 2000",
                       context={"count": ancient_dates})
        if large_messages > 0:
            report.add("biz.large_messages", Severity.INFO,
                       f"{large_messages} messages exceed {MAX_MESSAGE_LENGTH} chars after decode",
                       context={"count": large_messages})

        # Attachments count
        for msg in messages:
            mid = msg.get("id", "?")
            attachments = msg.get("attachments", [])
            if isinstance(attachments, list) and len(attachments) > MAX_ATTACHMENTS_PER_MESSAGE:
                report.add("biz.too_many_attachments", Severity.WARNING,
                           f"Message '{mid}' has {len(attachments)} attachments (max {MAX_ATTACHMENTS_PER_MESSAGE})",
                           context={"msg_id": mid, "count": len(attachments)})


# ============================================================================
# SQLite Integrity Validator
# ============================================================================

class SQLiteIntegrityValidator:
    """Валидация целостности SQLite БД после записи."""

    def validate(self, db_path: str, report: ValidationReport) -> None:
        report.source_format = "sqlite_db"

        if not Path(db_path).exists():
            report.add("sqlite.file_missing", Severity.CRITICAL,
                       f"Database file not found: {db_path}")
            return

        try:
            conn = sqlite3.connect(db_path)
            cursor = conn.cursor()

            self._check_integrity(cursor, report)
            self._check_tables(cursor, report)
            self._check_foreign_keys(cursor, report)
            self._check_orphan_records(cursor, report)
            self._check_stats(cursor, report)
            self._check_indices(cursor, report)

            conn.close()
        except sqlite3.Error as e:
            report.add("sqlite.connection_error", Severity.CRITICAL,
                       f"Cannot connect to database: {e}")

    def _check_integrity(self, cursor: sqlite3.Cursor, report: ValidationReport) -> None:
        try:
            result = cursor.execute("PRAGMA integrity_check").fetchone()
            if result and result[0] == "ok":
                report.add("sqlite.integrity", Severity.INFO,
                           "SQLite integrity check: OK")
            else:
                report.add("sqlite.integrity_failed", Severity.CRITICAL,
                           f"SQLite integrity check failed: {result}",
                           context={"result": str(result)})
        except sqlite3.Error as e:
            report.add("sqlite.integrity_error", Severity.CRITICAL,
                       f"Integrity check error: {e}")

    def _check_tables(self, cursor: sqlite3.Cursor, report: ValidationReport) -> None:
        expected_tables = {"contacts", "chat_history", "messages_cache", "settings"}
        tables = {row[0] for row in cursor.execute(
            "SELECT name FROM sqlite_master WHERE type='table'"
        ).fetchall()}

        missing = expected_tables - tables
        if missing:
            report.add("sqlite.missing_tables", Severity.ERROR,
                       f"Missing expected tables: {missing}",
                       context={"missing": list(missing), "found": list(tables)})
        else:
            report.add("sqlite.tables", Severity.INFO,
                       f"All expected tables present: {sorted(expected_tables)}")

        for table in expected_tables & tables:
            count = cursor.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
            report.add(f"sqlite.row_count.{table}", Severity.INFO,
                       f"Table '{table}': {count} rows",
                       context={"table": table, "rows": count})

    def _check_foreign_keys(self, cursor: sqlite3.Cursor, report: ValidationReport) -> None:
        # messages_cache → chat_history
        orphan_messages = cursor.execute("""
            SELECT COUNT(*) FROM messages_cache mc
            LEFT JOIN chat_history ch ON mc.chat_id = ch.chat_id
            WHERE ch.chat_id IS NULL
        """).fetchone()[0]

        if orphan_messages > 0:
            report.add("sqlite.orphan_messages", Severity.WARNING,
                       f"{orphan_messages} messages reference non-existent chats",
                       context={"count": orphan_messages},
                       suggestion="These messages will be invisible in the app")
        else:
            report.add("sqlite.no_orphan_messages", Severity.INFO,
                       "All messages reference valid chats")

        # contacts referenced by messages
        orphan_senders = cursor.execute("""
            SELECT COUNT(DISTINCT mc.sender_id) FROM messages_cache mc
            LEFT JOIN contacts c ON mc.sender_id = c.user_id
            WHERE c.user_id IS NULL AND mc.sender_id != 'system'
        """).fetchone()[0]

        if orphan_senders > 0:
            report.add("sqlite.orphan_senders", Severity.INFO,
                       f"{orphan_senders} senders have no contact record (normal for imported data)",
                       context={"count": orphan_senders})

    def _check_orphan_records(self, cursor: sqlite3.Cursor, report: ValidationReport) -> None:
        # Chat entries with no messages
        empty_chats = cursor.execute("""
            SELECT COUNT(*) FROM chat_history ch
            LEFT JOIN messages_cache mc ON ch.chat_id = mc.chat_id
            WHERE mc.id IS NULL
        """).fetchone()[0]

        if empty_chats > 0:
            report.add("sqlite.empty_chats", Severity.WARNING,
                       f"{empty_chats} chats have no messages",
                       context={"count": empty_chats})

    def _check_stats(self, cursor: sqlite3.Cursor, report: ValidationReport) -> None:
        try:
            meta = cursor.execute(
                "SELECT value FROM settings WHERE key='migration_meta'"
            ).fetchone()
            if meta:
                import json
                migration_data = json.loads(meta[0])
                stats = migration_data.get("stats", {})
                report.add("sqlite.migration_stats", Severity.INFO,
                           f"Migration metadata found",
                           context={"stats": stats})
        except Exception:
            pass

        version = cursor.execute(
            "SELECT value FROM settings WHERE key='migration_version'"
        ).fetchone()
        if version:
            report.add("sqlite.migration_version", Severity.INFO,
                       f"Migration version: {version[0]}",
                       context={"version": version[0]})
        else:
            report.add("sqlite.missing_version", Severity.WARNING,
                       "No migration_version in settings",
                       suggestion="Migration metadata was not saved")

    def _check_indices(self, cursor: sqlite3.Cursor, report: ValidationReport) -> None:
        indices = cursor.execute(
            "SELECT name, tbl_name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'"
        ).fetchall()

        if indices:
            report.add("sqlite.indices", Severity.INFO,
                       f"Found {len(indices)} custom indices",
                       context={"indices": [f"{name} on {tbl}" for name, tbl in indices]})
        else:
            report.add("sqlite.no_indices", Severity.WARNING,
                       "No custom indices found — queries may be slow",
                       suggestion="Consider adding indexes on chat_id, sender_id, created_at")


# ============================================================================
# Auto-Fix Engine
# ============================================================================

class AutoFixEngine:
    """Автоматическое исправление типовых проблем."""

    def fix(self, data: dict, report: ValidationReport) -> dict:
        fixes_applied = 0

        fixes_applied += self._fix_chat_types(data, report)
        fixes_applied += self._fix_chat_names(data, report)
        fixes_applied += self._fix_user_names(data, report)
        fixes_applied += self._fix_timestamps(data, report)
        fixes_applied += self._remove_duplicate_messages(data, report)

        if fixes_applied > 0:
            report.add("fix.applied", Severity.INFO,
                       f"Auto-fix applied: {fixes_applied} corrections",
                       context={"fixes": fixes_applied})

        return data

    def _fix_chat_types(self, data: dict, report: ValidationReport) -> int:
        fixed = 0
        for chat in data.get("chats", []):
            if chat.get("chat_type") not in VALID_CHAT_TYPES:
                chat["chat_type"] = "direct"
                fixed += 1
        return fixed

    def _fix_chat_names(self, data: dict, report: ValidationReport) -> int:
        fixed = 0
        for chat in data.get("chats", []):
            name = chat.get("name", "")
            if len(name) > MAX_CHAT_NAME_LENGTH:
                chat["name"] = name[:MAX_CHAT_NAME_LENGTH]
                fixed += 1
        return fixed

    def _fix_user_names(self, data: dict, report: ValidationReport) -> int:
        fixed = 0
        for user in data.get("users", []):
            if not user.get("display_name", "").strip() or user["display_name"] == "Unknown":
                uid = user.get("id", "unknown")
                user["display_name"] = f"User_{uid[:8]}"
                fixed += 1
        return fixed

    def _fix_timestamps(self, data: dict, report: ValidationReport) -> int:
        import time
        fixed = 0
        now_ts = int(time.time())
        for msg in data.get("messages", []):
            ts = msg.get("created_at")
            if ts is not None:
                try:
                    ts_int = int(ts)
                    if ts_int > now_ts + 86400:
                        msg["created_at"] = now_ts
                        fixed += 1
                    elif ts_int < MIN_TIMESTAMP:
                        msg["created_at"] = MIN_TIMESTAMP
                        fixed += 1
                except (ValueError, TypeError):
                    msg["created_at"] = now_ts
                    fixed += 1
        return fixed

    def _remove_duplicate_messages(self, data: dict, report: ValidationReport) -> int:
        messages = data.get("messages", [])
        seen_ids: set[str] = set()
        unique: list[dict] = []
        removed = 0

        for msg in messages:
            mid = msg.get("id")
            if mid and mid not in seen_ids:
                seen_ids.add(mid)
                unique.append(msg)
            else:
                removed += 1

        if removed > 0:
            data["messages"] = unique
            # Обновляем статистику
            if "stats" in data:
                data["stats"]["total_messages"] = len(unique)
                data["stats"]["removed_duplicates"] = removed

        return removed


# ============================================================================
# Public API
# ============================================================================

def validate_telegram_json(data: dict, strict: bool = False) -> ValidationReport:
    """Полная валидация Telegram JSON export."""
    import time
    report = ValidationReport(start_time=time.time())

    schema_val = TelegramSchemaValidator()
    schema_val.validate(data, report)

    if report.is_valid or not strict:
        biz_val = BusinessRulesValidator()
        biz_val.validate(data, report)

    report.end_time = time.time()
    return report


def validate_whatsapp_txt(file_path: Path, strict: bool = False) -> ValidationReport:
    """Полная валидация WhatsApp TXT export."""
    import time
    report = ValidationReport(start_time=time.time(), source_file=str(file_path))

    wa_val = WhatsAppSchemaValidator()
    wa_val.validate(file_path, report)

    report.end_time = time.time()
    return report


def validate_sqlite_db(db_path: str) -> ValidationReport:
    """Валидация SQLite базы после миграции."""
    import time
    report = ValidationReport(start_time=time.time(), source_file=db_path)

    db_val = SQLiteIntegrityValidator()
    db_val.validate(db_path, report)

    report.end_time = time.time()
    return report


def auto_fix(data: dict, report: ValidationReport) -> dict:
    """Применить авто-исправления к данным."""
    fixer = AutoFixEngine()
    return fixer.fix(data, report)


# ============================================================================
# CLI
# ============================================================================

def main():
    import argparse
    import time

    parser = argparse.ArgumentParser(
        description="Validation tool for migration data",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 validator.py --input result.json
  python3 validator.py --input result.json --fix --output fixed.json
  python3 validator.py --input migrated.db --db-check
  python3 validator.py --input result.json --strict
  python3 validator.py --input result.json --output report.json --format json
        """
    )

    parser.add_argument("--input", "-i", required=True, help="Input file path")
    parser.add_argument("--output", "-o", default=None, help="Output report path (JSON)")
    parser.add_argument("--fix", action="store_true", help="Auto-fix issues")
    parser.add_argument("--fixed-output", default=None, help="Output path for fixed data")
    parser.add_argument("--db-check", action="store_true", help="Validate SQLite database")
    parser.add_argument("--strict", action="store_true",
                        help="Strict mode: warnings are treated as errors")
    parser.add_argument("--format", choices=["text", "json"], default="text",
                        help="Output format (default: text)")

    args = parser.parse_args()
    input_path = Path(args.input)

    if not input_path.exists():
        print(f"❌ File not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    report = ValidationReport(start_time=time.time(), source_file=str(input_path))

    try:
        if args.db_check:
            # SQLite validation
            db_val = SQLiteIntegrityValidator()
            db_val.validate(str(input_path), report)
        elif input_path.suffix == ".json":
            # Telegram JSON validation
            with open(input_path, "r", encoding="utf-8") as f:
                data = json.load(f)

            schema_val = TelegramSchemaValidator()
            schema_val.validate(data, report)

            biz_val = BusinessRulesValidator()
            if not args.strict or report.is_valid:
                biz_val.validate(data, report)

            # Auto-fix
            if args.fix and not report.is_valid:
                print("🔧 Applying auto-fix...")
                fixer = AutoFixEngine()
                data = fixer.fix(data, report)

                if args.fixed_output:
                    with open(args.fixed_output, "w", encoding="utf-8") as f:
                        json.dump(data, f, indent=2, ensure_ascii=False)
                    print(f"✅ Fixed data written to: {args.fixed_output}")

        elif input_path.suffix == ".txt":
            # WhatsApp TXT validation
            wa_val = WhatsAppSchemaValidator()
            wa_val.validate(input_path, report)
        else:
            print(f"❌ Unsupported file format: {input_path.suffix}", file=sys.stderr)
            sys.exit(1)

    except json.JSONDecodeError as e:
        report.add("parse.json_error", Severity.CRITICAL,
                   f"JSON parse error: {e}")
    except Exception as e:
        report.add("unknown_error", Severity.CRITICAL,
                   f"Unexpected error: {e}")

    report.end_time = time.time()

    # Output
    if args.format == "json":
        output = report.to_dict()
        if args.output:
            with open(args.output, "w", encoding="utf-8") as f:
                json.dump(output, f, indent=2, ensure_ascii=False)
            print(f"📋 Report written to: {args.output}")
        else:
            print(json.dumps(output, indent=2, ensure_ascii=False))
    else:
        print(report.summary())

    # Exit code
    if not report.is_valid:
        sys.exit(2)
    elif args.strict and report.warning_count > 0:
        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == "__main__":
    main()
