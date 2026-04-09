#!/usr/bin/env python3
"""
Unit tests для validator.py

Запуск:
    python3 tests/test_validator.py
    python3 -m pytest tests/test_validator.py -v
"""

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

# Добавляем родительскую директорию в path
sys.path.insert(0, str(Path(__file__).parent.parent))

from validator import (
    ValidationReport,
    ValidationResult,
    Severity,
    TelegramSchemaValidator,
    WhatsAppSchemaValidator,
    BusinessRulesValidator,
    SQLiteIntegrityValidator,
    AutoFixEngine,
    validate_telegram_json,
    validate_whatsapp_txt,
    validate_sqlite_db,
    auto_fix,
)


class TestValidationReport(unittest.TestCase):
    """Тесты ValidationReport."""

    def test_empty_report_is_valid(self):
        report = ValidationReport()
        self.assertTrue(report.is_valid)
        self.assertEqual(report.error_count, 0)
        self.assertEqual(report.warning_count, 0)

    def test_error_makes_invalid(self):
        report = ValidationReport()
        report.add("test.rule", Severity.ERROR, "Test error")
        self.assertFalse(report.is_valid)
        self.assertEqual(report.error_count, 1)

    def test_warning_does_not_invalidate(self):
        report = ValidationReport()
        report.add("test.rule", Severity.WARNING, "Test warning")
        self.assertTrue(report.is_valid)
        self.assertEqual(report.warning_count, 1)

    def test_critical_makes_invalid(self):
        report = ValidationReport()
        report.add("test.rule", Severity.CRITICAL, "Critical error")
        self.assertFalse(report.is_valid)
        self.assertEqual(report.critical_count, 1)

    def test_to_dict(self):
        report = ValidationReport(source_file="test.json", source_format="telegram")
        report.add("test.rule", Severity.ERROR, "Test error", context={"key": "val"}, suggestion="Fix it")
        d = report.to_dict()
        self.assertEqual(d["source_file"], "test.json")
        self.assertEqual(d["counts"]["error"], 1)
        self.assertEqual(len(d["results"]), 1)
        self.assertEqual(d["results"][0]["rule"], "test.rule")
        self.assertEqual(d["results"][0]["suggestion"], "Fix it")


class TestTelegramSchemaValidator(unittest.TestCase):
    """Тесты валидации схемы Telegram JSON."""

    def setUp(self):
        self.validator = TelegramSchemaValidator()

    def test_valid_single_chat(self):
        data = {
            "name": "Test Chat",
            "type": "personal_chat",
            "id": 123456,
            "messages": [
                {
                    "id": 1,
                    "type": "message",
                    "date": "2024-01-15T10:30:00",
                    "from": "Alice",
                    "from_id": 111111,
                    "text": "Hello",
                }
            ],
        }
        report = ValidationReport()
        self.validator.validate(data, report)
        errors = [r for r in report.results if r.severity in (Severity.ERROR, Severity.CRITICAL)]
        self.assertEqual(len(errors), 0, f"Unexpected errors: {[str(e) for e in errors]}")

    def test_missing_name(self):
        data = {"type": "personal_chat", "id": 123, "messages": []}
        report = ValidationReport()
        self.validator.validate(data, report)
        missing = [r for r in report.results if "required_field" in r.rule and "name" in r.message]
        self.assertTrue(len(missing) > 0)

    def test_missing_messages(self):
        data = {"name": "Test", "type": "personal_chat", "id": 123}
        report = ValidationReport()
        self.validator.validate(data, report)
        missing = [r for r in report.results if "required_field" in r.rule and "messages" in r.message]
        self.assertTrue(len(missing) > 0)

    def test_invalid_chat_type(self):
        data = {"name": "Test", "type": "invalid_type", "id": 123, "messages": []}
        report = ValidationReport()
        self.validator.validate(data, report)
        type_errors = [r for r in report.results if "chat_type" in r.rule]
        self.assertTrue(len(type_errors) > 0)

    def test_messages_not_array(self):
        data = {"name": "Test", "type": "personal_chat", "id": 123, "messages": "not_array"}
        report = ValidationReport()
        self.validator.validate(data, report)
        type_errors = [r for r in report.results if "messages_type" in r.rule]
        self.assertTrue(len(type_errors) > 0)

    def test_message_missing_id(self):
        data = {
            "name": "Test",
            "type": "personal_chat",
            "id": 123,
            "messages": [{"type": "message", "date": "2024-01-15T10:00:00", "text": "No ID"}],
        }
        report = ValidationReport()
        self.validator.validate(data, report)
        id_errors = [r for r in report.results if "missing_id" in r.rule]
        self.assertTrue(len(id_errors) > 0)

    def test_message_text_not_string_or_array(self):
        data = {
            "name": "Test",
            "type": "personal_chat",
            "id": 123,
            "messages": [{"id": 1, "type": "message", "date": "2024-01-15T10:00:00", "text": 12345}],
        }
        report = ValidationReport()
        self.validator.validate(data, report)
        text_errors = [r for r in report.results if "text_type" in r.rule]
        self.assertTrue(len(text_errors) > 0)

    def test_multi_chat_format(self):
        data = {
            "chats": {
                "list": [
                    {"name": "Chat1", "type": "personal_chat", "id": 1, "messages": []},
                    {"name": "Chat2", "type": "private_group", "id": 2, "messages": []},
                ]
            }
        }
        report = ValidationReport()
        self.validator.validate(data, report)
        errors = [r for r in report.results if r.severity in (Severity.ERROR, Severity.CRITICAL)]
        self.assertEqual(len(errors), 0)

    def test_unknown_format(self):
        data = {"foo": "bar"}
        report = ValidationReport()
        self.validator.validate(data, report)
        unknown = [r for r in report.results if "unknown_format" in r.rule]
        self.assertTrue(len(unknown) > 0)

    def test_root_not_dict(self):
        report = ValidationReport()
        self.validator.validate("not a dict", report)
        critical = [r for r in report.results if r.severity == Severity.CRITICAL]
        self.assertTrue(len(critical) > 0)


class TestBusinessRulesValidator(unittest.TestCase):
    """Тесты бизнес-правил."""

    def setUp(self):
        self.validator = BusinessRulesValidator()

    def test_valid_data(self):
        data = {
            "users": [
                {"id": "tg_111", "display_name": "Alice", "first_seen": 1705312200, "last_seen": 1705312200, "messages_count": 1},
                {"id": "tg_222", "display_name": "Bob", "first_seen": 1705312260, "last_seen": 1705312260, "messages_count": 1},
            ],
            "chats": [
                {"id": "abc123", "name": "Test Chat", "chat_type": "direct", "participants": ["tg_111", "tg_222"], "messages_count": 2, "last_message_at": 1705312260},
            ],
            "messages": [
                {"id": "msg1", "chat_id": "abc123", "sender_id": "tg_111", "encrypted_content": "SGVsbG8=", "signature": "c3R1Yg==", "created_at": 1705312200, "msg_type": "text", "delivery_status": "delivered", "reply_to": None, "attachments": []},
                {"id": "msg2", "chat_id": "abc123", "sender_id": "tg_222", "encrypted_content": "SGk=", "signature": "c3R1Yg==", "created_at": 1705312260, "msg_type": "text", "delivery_status": "delivered", "reply_to": "msg1", "attachments": []},
            ],
        }
        report = ValidationReport()
        self.validator.validate(data, report)
        errors = [r for r in report.results if r.severity in (Severity.ERROR, Severity.CRITICAL)]
        self.assertEqual(len(errors), 0)

    def test_duplicate_user_ids(self):
        data = {
            "users": [
                {"id": "tg_111", "display_name": "Alice"},
                {"id": "tg_111", "display_name": "Alice Copy"},
            ],
            "chats": [],
            "messages": [],
        }
        report = ValidationReport()
        self.validator.validate(data, report)
        dupes = [r for r in report.results if "duplicate_users" in r.rule]
        self.assertTrue(len(dupes) > 0)

    def test_duplicate_message_ids(self):
        data = {
            "users": [],
            "chats": [{"id": "abc", "name": "Chat", "chat_type": "direct"}],
            "messages": [
                {"id": "msg1", "chat_id": "abc", "sender_id": "tg_1"},
                {"id": "msg1", "chat_id": "abc", "sender_id": "tg_2"},
            ],
        }
        report = ValidationReport()
        self.validator.validate(data, report)
        dupes = [r for r in report.results if "duplicate_messages" in r.rule]
        self.assertTrue(len(dupes) > 0)

    def test_orphan_reply(self):
        data = {
            "users": [],
            "chats": [{"id": "abc", "name": "Chat", "chat_type": "direct"}],
            "messages": [
                {"id": "msg1", "chat_id": "abc", "sender_id": "tg_1", "reply_to": "nonexistent"},
            ],
        }
        report = ValidationReport()
        self.validator.validate(data, report)
        orphans = [r for r in report.results if "orphan_reply" in r.rule]
        self.assertTrue(len(orphans) > 0)


class TestAutoFixEngine(unittest.TestCase):
    """Тесты авто-исправлений."""

    def setUp(self):
        self.fixer = AutoFixEngine()

    def test_fix_chat_type(self):
        data = {
            "chats": [{"id": "abc", "name": "Chat", "chat_type": "invalid_type"}],
            "messages": [],
        }
        report = ValidationReport()
        result = self.fixer.fix(data, report)
        self.assertEqual(result["chats"][0]["chat_type"], "direct")

    def test_fix_long_chat_name(self):
        long_name = "A" * 300
        data = {
            "chats": [{"id": "abc", "name": long_name, "chat_type": "direct"}],
            "messages": [],
        }
        report = ValidationReport()
        result = self.fixer.fix(data, report)
        self.assertLessEqual(len(result["chats"][0]["name"]), 255)

    def test_fix_empty_user_name(self):
        data = {
            "users": [{"id": "tg_12345678", "display_name": ""}],
            "chats": [],
            "messages": [],
        }
        report = ValidationReport()
        result = self.fixer.fix(data, report)
        self.assertTrue(result["users"][0]["display_name"].startswith("User_"))

    def test_fix_future_timestamp(self):
        import time
        future_ts = int(time.time()) + 1000000
        data = {
            "users": [],
            "chats": [],
            "messages": [{"id": "msg1", "created_at": future_ts}],
        }
        report = ValidationReport()
        result = self.fixer.fix(data, report)
        self.assertLess(result["messages"][0]["created_at"], future_ts)

    def test_remove_duplicate_messages(self):
        data = {
            "users": [],
            "chats": [],
            "messages": [
                {"id": "msg1", "text": "first"},
                {"id": "msg1", "text": "duplicate"},
                {"id": "msg2", "text": "unique"},
            ],
        }
        report = ValidationReport()
        result = self.fixer.fix(data, report)
        self.assertEqual(len(result["messages"]), 2)

    def test_fix_ancient_timestamp(self):
        data = {
            "users": [],
            "chats": [],
            "messages": [{"id": "msg1", "created_at": 631152000}],  # 1990-01-01
        }
        report = ValidationReport()
        result = self.fixer.fix(data, report)
        self.assertGreaterEqual(result["messages"][0]["created_at"], 946684800)  # 2000-01-01


class TestValidateTelegramJson(unittest.TestCase):
    """Integration тест validate_telegram_json."""

    def test_valid_export(self):
        data = {
            "name": "Test",
            "type": "personal_chat",
            "id": 123,
            "messages": [
                {"id": 1, "type": "message", "date": "2024-01-15T10:00:00", "from": "Alice", "from_id": 111, "text": "Hi"}
            ],
        }
        report = validate_telegram_json(data)
        self.assertTrue(report.is_valid)

    def test_error_export(self):
        data = {
            "name": "Test",
            "type": "personal_chat",
            "id": 123,
            "messages": [
                {"id": 1, "type": "message", "date": "2024-01-15T10:00:00", "from": "Alice", "from_id": 111, "text": "Hi"},
                {"id": 1, "type": "message", "date": "2024-01-15T10:01:00", "from": "Alice", "from_id": 111, "text": "Dup"},
            ],
        }
        report = validate_telegram_json(data)
        # Schema-level passes, business rules catch duplicate
        biz_errors = [r for r in report.results if r.severity in (Severity.ERROR, Severity.CRITICAL)]
        self.assertTrue(len(biz_errors) > 0)


class TestValidateWhatsappTxt(unittest.TestCase):
    """Тесты валидации WhatsApp TXT."""

    def test_valid_whatsapp_file(self):
        test_file = Path(__file__).parent.parent / "test_data" / "whatsapp_chat.txt"
        if test_file.exists():
            report = validate_whatsapp_txt(test_file)
            self.assertTrue(report.is_valid)

    def test_missing_file(self):
        report = validate_whatsapp_txt(Path("/nonexistent/file.txt"))
        self.assertFalse(report.is_valid)
        critical = [r for r in report.results if r.severity == Severity.CRITICAL]
        self.assertTrue(len(critical) > 0)

    def test_empty_file(self):
        with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as f:
            tmp_path = f.name
        try:
            report = validate_whatsapp_txt(Path(tmp_path))
            self.assertFalse(report.is_valid)
        finally:
            os.unlink(tmp_path)


class TestSQLiteIntegrityValidator(unittest.TestCase):
    """Тесты валидации SQLite."""

    def test_missing_db(self):
        report = validate_sqlite_db("/nonexistent/database.db")
        self.assertFalse(report.is_valid)

    def test_valid_empty_db(self):
        import sqlite3
        with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as f:
            tmp_path = f.name

        try:
            conn = sqlite3.connect(tmp_path)
            conn.execute("""
                CREATE TABLE contacts (
                    id TEXT PRIMARY KEY, user_id TEXT NOT NULL UNIQUE,
                    encrypted_display_name BLOB NOT NULL,
                    encrypted_avatar TEXT,
                    public_key_x25519 BLOB NOT NULL,
                    public_key_kyber BLOB NOT NULL,
                    public_key_ed25519 BLOB NOT NULL,
                    encrypted_notes BLOB, added_at INTEGER NOT NULL,
                    last_contacted INTEGER, is_blocked INTEGER NOT NULL DEFAULT 0,
                    is_favorite INTEGER NOT NULL DEFAULT 0
                )
            """)
            conn.execute("""
                CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL)
            """)
            conn.execute("""
                CREATE TABLE chat_history (
                    id TEXT PRIMARY KEY, chat_id TEXT NOT NULL, chat_type TEXT NOT NULL,
                    peer_id TEXT, encrypted_name BLOB, last_message_id TEXT,
                    last_message_at INTEGER, unread_count INTEGER NOT NULL DEFAULT 0,
                    is_archived INTEGER NOT NULL DEFAULT 0, is_muted INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
                )
            """)
            conn.execute("""
                CREATE TABLE messages_cache (
                    id TEXT PRIMARY KEY, chat_id TEXT NOT NULL, sender_id TEXT NOT NULL,
                    encrypted_content BLOB NOT NULL, signature BLOB NOT NULL,
                    created_at INTEGER NOT NULL, msg_type TEXT NOT NULL,
                    delivery_status TEXT NOT NULL, reply_to TEXT,
                    attachments TEXT, is_deleted INTEGER NOT NULL DEFAULT 0
                )
            """)
            conn.commit()
            conn.close()

            report = validate_sqlite_db(tmp_path)
            self.assertTrue(report.is_valid)
        finally:
            os.unlink(tmp_path)


class TestValidatorWithRealFile(unittest.TestCase):
    """Тесты с реальными тестовыми файлами."""

    def test_valid_test_data(self):
        test_file = Path(__file__).parent.parent / "test_data" / "result.json"
        if test_file.exists():
            with open(test_file, "r", encoding="utf-8") as f:
                data = json.load(f)
            report = validate_telegram_json(data)
            self.assertTrue(report.is_valid, f"Test data should be valid, but got: {[str(r) for r in report.results if r.severity in (Severity.ERROR, Severity.CRITICAL)]}")

    def test_error_test_data(self):
        test_file = Path(__file__).parent.parent / "test_data" / "result_with_errors.json"
        if test_file.exists():
            with open(test_file, "r", encoding="utf-8") as f:
                data = json.load(f)
            report = validate_telegram_json(data)
            errors = [r for r in report.results if r.severity in (Severity.ERROR, Severity.CRITICAL)]
            # Ожидаем ошибки: дубликаты, missing ID, etc.
            self.assertTrue(len(errors) > 0, "Error test data should have validation errors")


if __name__ == "__main__":
    unittest.main()
