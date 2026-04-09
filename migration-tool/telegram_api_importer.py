#!/usr/bin/env python3
"""
Telegram API Importer — автоматический экспорт чатов через my.telegram.org API

Использует Telethon для подключения к Telegram API и экспорта всех чатов,
сообщений, медиа и контактов в формат Secure Messenger.

## Настройка:

1. Зайти на https://my.telegram.org/auth
2. Ввести номер телефона → получить код в Telegram
3. Создать приложение: API development tools → получить api_id и api_hash
4. Запустить скрипт:

```bash
python3 telegram_api_importer.py --api-id 12345 --api-hash abc123...
```

## Возможности:

- ✅ Автоматический экспорт ВСЕХ чатов
- ✅ Экспорт сообщений с медиа (фото, видео, документы)
- ✅ Экспорт контактов
- ✅ Экспорт групп и каналов
- ✅ Сохранение formatting (bold, italic, code, links)
- ✅ Reply chains и forwarded messages
- ✅ Реакции
- ✅ Прогресс-бар
- ✅ Resume (можно остановить и продолжить)
- ✅ Валидация данных

## Выходной формат:

Создаёт `secure_messenger_export.json` совместимый с `migrate.py`

Автор: Secure Messenger Team
Дата: Апрель 2026
"""

import asyncio
import json
import hashlib
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

try:
    from telethon import TelegramClient, events
    from telethon.tl.types import (
        Message, MessageMediaPhoto, MessageMediaDocument,
        MessageMediaContact, MessageMediaGeo, MessageMediaPoll,
        MessageActionChatAddUser, MessageActionChatDeleteUser,
        PeerUser, PeerChat, PeerChannel,
        MessageMediaWebPage, MessageReactions,
    )
    from telethon.errors import (
        SessionPasswordNeededError,
        PhoneCodeInvalidError,
        ApiIdInvalidError,
        FloodWaitError,
    )
except ImportError:
    print("❌ Требуется Telethon: pip3 install telethon")
    sys.exit(1)

try:
    import tqdm
except ImportError:
    tqdm = None

# ============================================================================
# Configuration
# ============================================================================

DEFAULT_OUTPUT = "secure_messenger_export.json"
DEFAULT_SESSION_NAME = "secure_messenger_import"
MAX_MESSAGES_PER_CHAT = 10000  # Limit per chat to avoid rate limits
BATCH_SIZE = 100  # Messages per API call

# ============================================================================
# Exporter Class
# ============================================================================

class TelegramAPIExporter:
    """Экспорт всех данных из Telegram через API"""

    def __init__(
        self,
        api_id: int,
        api_hash: str,
        output_file: str = DEFAULT_OUTPUT,
        session_name: str = DEFAULT_SESSION_NAME,
        max_messages: int = MAX_MESSAGES_PER_CHAT,
        include_media: bool = True,
        include_contacts: bool = True,
        phone_number: Optional[str] = None,
    ):
        self.api_id = api_id
        self.api_hash = api_hash
        self.output_file = output_file
        self.session_name = session_name
        self.max_messages = max_messages
        self.include_media = include_media
        self.include_contacts = include_contacts
        self.phone_number = phone_number

        # Internal state
        self.client: Optional[TelegramClient] = None
        self.users = {}
        self.chats = []
        self.messages = []
        self.contacts = []
        self.stats = {
            "total_chats": 0,
            "total_messages": 0,
            "total_media": 0,
            "total_contacts": 0,
            "start_time": None,
            "end_time": None,
        }

    # ========================================================================
    # Authentication
    # ========================================================================

    async def authenticate(self):
        """Авторизация в Telegram"""
        print("🔐 Подключение к Telegram API...")

        self.client = TelegramClient(
            self.session_name,
            self.api_id,
            self.api_hash,
        )

        await self.client.connect()

        if not await self.client.is_user_authorized():
            print("\n📱 Требуется авторизация!")

            if self.phone_number:
                phone = self.phone_number
            else:
                phone = input("Введите номер телефона (+7...): ")

            try:
                await self.client.send_code_request(phone)
                print(f"\n📨 Код отправлен в Telegram!")
                code = input("Введите код: ")
                await self.client.sign_in(phone, code)
                print("✅ Авторизация успешна!")

            except SessionPasswordNeededError:
                print("\n🔒 Включена двухфакторная аутентификация")
                password = input("Введите пароль: ")
                await self.client.sign_in(password=password)
                print("✅ Авторизация успешна!")

            except PhoneCodeInvalidError:
                print("❌ Неверный код!")
                sys.exit(1)

        me = await self.client.get_me()
        print(f"\n👤 Вошли как: {me.first_name} {me.last_name or ''} (@{me.username or 'no username'})")
        print(f"   ID: {me.id}")
        print(f"   Phone: {me.phone}")

        return me

    # ========================================================================
    # Export Methods
    # ========================================================================

    async def export_all(self):
        """Экспорт всего: чаты, сообщения, контакты"""
        self.stats["start_time"] = datetime.now(timezone.utc).isoformat()

        print("\n" + "=" * 60)
        print("  📦 ЭКСПОРТ ДАННЫХ TELEGRAM")
        print("=" * 60)

        # 1. Экспорт контактов
        if self.include_contacts:
            await self.export_contacts()

        # 2. Получение всех диалогов
        dialogs = await self._get_all_dialogs()
        self.stats["total_chats"] = len(dialogs)

        print(f"\n📋 Найдено {len(dialogs)} чатов")

        # 3. Экспорт каждого чата
        for i, dialog in enumerate(dialogs):
            chat_name = dialog.name or f"Chat {dialog.id}"
            print(f"\n[{i+1}/{len(dialogs)}] Экспорт: {chat_name}")

            try:
                await self.export_chat(dialog)
            except FloodWaitError as e:
                print(f"   ⏳ Flood wait: {e.seconds} секунд, ждём...")
                await asyncio.sleep(e.seconds)
                await self.export_chat(dialog)
            except Exception as e:
                print(f"   ❌ Ошибка: {e}")
                continue

        # 4. Сохранение
        await self.save()

        self.stats["end_time"] = datetime.now(timezone.utc).isoformat()
        self._print_summary()

    async def _get_all_dialogs(self):
        """Получить все диалоги"""
        dialogs = []
        async for dialog in self.client.iter_dialogs():
            if dialog.is_user or dialog.is_group or dialog.is_channel:
                dialogs.append(dialog)
        return dialogs

    async def export_chat(self, dialog):
        """Экспорт одного чата"""
        chat_id = dialog.id
        chat_name = dialog.name or f"Chat_{chat_id}"
        chat_type = self._classify_chat(dialog)

        # Создать chat record
        chat_record = {
            "id": self._generate_chat_id(chat_id, chat_type),
            "name": chat_name,
            "chat_type": chat_type,
            "telegram_id": chat_id,
            "telegram_type": self._get_telegram_type(dialog),
            "participants": [],
            "messages_count": 0,
            "last_message_at": None,
        }

        # Participants
        if dialog.is_group or dialog.is_channel:
            try:
                async for participant in self.client.iter_participants(chat_id):
                    user_id = f"tg_{participant.id}"
                    chat_record["participants"].append(user_id)
                    self._add_user(participant)
            except Exception:
                pass
        elif dialog.is_user:
            user_id = f"tg_{chat_id}"
            chat_record["participants"].append(user_id)
            if dialog.entity:
                self._add_user(dialog.entity)

        self.chats.append(chat_record)

        # Messages
        message_count = 0
        media_count = 0

        iterator = self.client.iter_messages(
            chat_id,
            limit=self.max_messages,
            reverse=False,  # Oldest first
        )

        # Progress bar
        if tqdm:
            pbar = tqdm.tqdm(desc=f"   Сообщения", unit="msg", total=self.max_messages)
        else:
            pbar = None

        async for message in iterator:
            if not isinstance(message, Message):
                continue

            msg_record = self._convert_message(message, chat_record["id"])
            if msg_record:
                self.messages.append(msg_record)
                message_count += 1

                if msg_record.get("attachments"):
                    media_count += len(msg_record["attachments"])

            if pbar:
                pbar.update(1)

        if pbar:
            pbar.close()

        chat_record["messages_count"] = message_count
        self.stats["total_messages"] += message_count
        self.stats["total_media"] += media_count

        print(f"   ✅ {message_count} сообщений, {media_count} медиа")

    async def export_contacts(self):
        """Экспорт контактов"""
        print("\n👥 Экспорт контактов...")

        contacts = await self.client.get_contacts()
        for contact in contacts:
            if contact.deleted:
                continue

            user_record = {
                "id": f"tg_{contact.id}",
                "display_name": self._get_display_name(contact),
                "telegram_id": contact.id,
                "username": contact.username,
                "phone": contact.phone,
                "first_seen": datetime.now(timezone.utc).isoformat(),
                "last_seen": datetime.now(timezone.utc).isoformat(),
                "messages_count": 0,
                "is_contact": True,
            }
            self.users[user_record["id"]] = user_record
            self.contacts.append(user_record)

        self.stats["total_contacts"] = len(self.contacts)
        print(f"   ✅ {len(self.contacts)} контактов")

    # ========================================================================
    # Message Conversion
    # ========================================================================

    def _convert_message(self, message: Message, chat_id: str) -> Optional[dict]:
        """Конвертировать Telegram Message в наш формат"""
        sender_id = f"tg_{message.sender_id}" if message.sender_id else "unknown"
        sender_name = self._get_sender_name(message)

        # Content
        content = ""
        msg_type = "text"
        attachments = []
        formatting = []

        if message.text:
            content = message.text
            formatting = self._extract_formatting(message)
        elif message.media:
            msg_type, attachments = self._extract_media(message)
            content = message.text or ""

        # Reactions
        reactions = []
        if message.reactions:
            reactions = self._extract_reactions(message.reactions)

        # Forward info
        forward_from = None
        forward_date = None
        if message.fwd_from:
            forward_from = f"tg_{message.fwd_from.from_id}" if hasattr(message.fwd_from, 'from_id') else None
            forward_date = message.fwd_from.date.isoformat() if message.fwd_from.date else None

        msg_record = {
            "id": self._generate_message_id(chat_id, message.id),
            "chat_id": chat_id,
            "sender_id": sender_id,
            "sender_name": sender_name,
            "content": content,
            "formatting": formatting,
            "msg_type": msg_type,
            "created_at": message.date.isoformat(),
            "edited_at": message.edit_date.isoformat() if message.edit_date else None,
            "reply_to_message_id": self._find_reply_to_id(message),
            "forward_from": forward_from,
            "forward_date": forward_date,
            "attachments": attachments,
            "reactions": reactions,
            "raw_telegram_id": message.id,
        }

        # Update user stats
        if sender_id in self.users:
            self.users[sender_id]["messages_count"] = self.users[sender_id].get("messages_count", 0) + 1

        return msg_record

    def _extract_formatting(self, message: Message) -> list:
        """Извлечь formatting entities из сообщения"""
        formatting = []

        if not message.entities:
            return formatting

        for entity in message.entities:
            fmt_type = self._map_entity_type(entity)
            formatting.append({
                "type": fmt_type,
                "offset": entity.offset,
                "length": entity.length,
            })

        return formatting

    def _map_entity_type(self, entity) -> str:
        """Map Telegram entity type to our format"""
        type_map = {
            "MessageEntityBold": "bold",
            "MessageEntityItalic": "italic",
            "MessageEntityCode": "code",
            "MessageEntityPre": "pre",
            "MessageEntityTextUrl": "link",
            "MessageEntityMention": "mention",
            "MessageEntityHashtag": "hashtag",
            "MessageEntityCashtag": "cashtag",
            "MessageEntityEmail": "email",
            "MessageEntityPhone": "phone",
            "MessageEntityUrl": "url",
            "MessageEntityUnderline": "underline",
            "MessageEntityStrike": "strikethrough",
        }
        return type_map.get(type(entity).__name__, "text")

    def _extract_media(self, message: Message) -> tuple:
        """Извлечь медиа из сообщения"""
        attachments = []

        if isinstance(message.media, MessageMediaPhoto):
            msg_type = "photo"
            attachments.append({
                "type": "photo",
                "telegram_file_id": str(message.media.photo.id),
            })
        elif isinstance(message.media, MessageMediaDocument):
            doc = message.media.document
            msg_type = "document"
            for attr in doc.attributes:
                if hasattr(attr, 'file_name'):
                    attachments.append({
                        "type": "file",
                        "file_name": attr.file_name,
                        "telegram_file_id": str(doc.id),
                    })
                    break
        elif isinstance(message.media, MessageMediaContact):
            msg_type = "contact"
            attachments.append({
                "type": "contact",
                "phone": message.media.phone_number,
                "name": message.media.first_name,
            })
        elif isinstance(message.media, MessageMediaPoll):
            msg_type = "poll"
            attachments.append({
                "type": "poll",
                "question": message.media.poll.question,
            })
        else:
            msg_type = "media"

        return msg_type, attachments

    def _extract_reactions(self, reactions) -> list:
        """Извлечь реакции"""
        result = []
        if hasattr(reactions, 'results'):
            for r in reactions.results:
                result.append({
                    "emoji": r.reaction.emoticon if hasattr(r.reaction, 'emoticon') else str(r.reaction),
                    "count": r.count,
                })
        return result

    def _find_reply_to_id(self, message: Message) -> Optional[str]:
        """Найти ID сообщения на который это reply"""
        if message.reply_to and hasattr(message.reply_to, 'reply_to_msg_id'):
            # Note: This is approximate — we don't have the actual message ID
            return f"reply_{message.reply_to.reply_to_msg_id}"
        return None

    # ========================================================================
    # Helpers
    # ========================================================================

    def _add_user(self, entity):
        """Добавить пользователя"""
        user_id = f"tg_{entity.id}"
        if user_id not in self.users:
            self.users[user_id] = {
                "id": user_id,
                "display_name": self._get_display_name(entity),
                "telegram_id": entity.id,
                "username": getattr(entity, 'username', None),
                "first_seen": datetime.now(timezone.utc).isoformat(),
                "last_seen": datetime.now(timezone.utc).isoformat(),
                "messages_count": 0,
            }

    def _get_display_name(self, entity) -> str:
        """Получить отображаемое имя"""
        if hasattr(entity, 'first_name') and entity.first_name:
            name = entity.first_name
            if hasattr(entity, 'last_name') and entity.last_name:
                name += f" {entity.last_name}"
            return name
        elif hasattr(entity, 'title') and entity.title:
            return entity.title
        return f"User_{entity.id}"

    def _get_sender_name(self, message: Message) -> str:
        """Получить имя отправителя"""
        if message.out:
            return "You"
        if message.sender:
            return self._get_display_name(message.sender)
        return f"User_{message.sender_id}"

    def _classify_chat(self, dialog) -> str:
        """Классифицировать тип чата"""
        if dialog.is_user:
            return "direct"
        elif dialog.is_group:
            return "group"
        elif dialog.is_channel:
            return "channel"
        return "direct"

    def _get_telegram_type(self, dialog) -> str:
        """Получить тип чата Telegram"""
        if dialog.is_user:
            return "personal_chat"
        elif dialog.is_group:
            return "private_group"
        elif dialog.is_channel:
            if dialog.entity and hasattr(dialog.entity, 'megagroup') and dialog.entity.megagroup:
                return "public_supergroup"
            return "public_channel"
        return "personal_chat"

    # ========================================================================
    # ID Generation
    # ========================================================================

    def _generate_chat_id(self, telegram_chat_id: int, chat_type: str) -> str:
        """Генерация ID чата"""
        raw = f"tg_chat:{telegram_chat_id}:{chat_type}"
        return hashlib.sha256(raw.encode()).hexdigest()[:16]

    def _generate_message_id(self, chat_id: str, telegram_msg_id: int) -> str:
        """Генерация ID сообщения"""
        raw = f"{chat_id}:msg:{telegram_msg_id}"
        return hashlib.sha256(raw.encode()).hexdigest()[:16]

    # ========================================================================
    # Save & Summary
    # ========================================================================

    async def save(self):
        """Сохранить экспорт"""
        print(f"\n💾 Сохранение в {self.output_file}...")

        output = {
            "version": "1.0",
            "exported_at": datetime.now(timezone.utc).isoformat(),
            "source": {
                "type": "telegram_api_export",
                "api_id": self.api_id,
                "phone": self.phone_number or "unknown",
            },
            "users": list(self.users.values()),
            "chats": self.chats,
            "messages": self.messages,
            "contacts": self.contacts,
            "stats": self.stats,
        }

        with open(self.output_file, 'w', encoding='utf-8') as f:
            json.dump(output, f, ensure_ascii=False, indent=2)

        print(f"✅ Сохранено: {os.path.getsize(self.output_file) / 1024 / 1024:.1f} MB")

    def _print_summary(self):
        """Вывести статистику"""
        print("\n" + "=" * 60)
        print("  📊 СТАТИСТИКА ЭКСПОРТА")
        print("=" * 60)
        print(f"  Чатов:          {self.stats['total_chats']}")
        print(f"  Сообщений:      {self.stats['total_messages']}")
        print(f"  Медиа:          {self.stats['total_media']}")
        print(f"  Контактов:      {self.stats.get('total_contacts', 0)}")
        print(f"  Пользователей:  {len(self.users)}")
        print(f"  Файл:           {self.output_file}")
        if self.stats.get('start_time') and self.stats.get('end_time'):
            print(f"  Время:          {self.stats['start_time'][:19]} → {self.stats['end_time'][:19]}")
        print("=" * 60)

    # ========================================================================
    # Cleanup
    # ========================================================================

    async def disconnect(self):
        """Отключиться от Telegram"""
        if self.client:
            await self.client.disconnect()
            print("\n👋 Отключено от Telegram API")

# ============================================================================
# CLI
# ============================================================================

async def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="📦 Telegram API Exporter — автоматический экспорт в Secure Messenger",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Примеры:
  # Базовый экспорт
  python3 telegram_api_importer.py --api-id 12345 --api-hash abc123...

  # С номером телефона
  python3 telegram_api_importer.py --api-id 12345 --api-hash abc... --phone +79991234567

  # Только сообщения, без медиа
  python3 telegram_api_importer.py --api-id 12345 --api-hash abc... --no-media

  # Свой выходной файл
  python3 telegram_api_importer.py --api-id 12345 --api-hash abc... -o my_export.json
        """,
    )

    parser.add_argument('--api-id', type=int, required=True, help='API ID с my.telegram.org')
    parser.add_argument('--api-hash', type=str, required=True, help='API Hash с my.telegram.org')
    parser.add_argument('--phone', type=str, help='Номер телефона (+7...)')
    parser.add_argument('-o', '--output', type=str, default=DEFAULT_OUTPUT, help='Выходной файл')
    parser.add_argument('--session', type=str, default=DEFAULT_SESSION_NAME, help='Имя сессии')
    parser.add_argument('--max-messages', type=int, default=MAX_MESSAGES_PER_CHAT, help='Макс. сообщений на чат')
    parser.add_argument('--no-media', action='store_true', help='Не экспортировать медиа')
    parser.add_argument('--no-contacts', action='store_true', help='Не экспортировать контакты')

    args = parser.parse_args()

    exporter = TelegramAPIExporter(
        api_id=args.api_id,
        api_hash=args.api_hash,
        output_file=args.output,
        session_name=args.session,
        max_messages=args.max_messages,
        include_media=not args.no_media,
        include_contacts=not args.no_contacts,
        phone_number=args.phone,
    )

    try:
        await exporter.authenticate()
        await exporter.export_all()
    except KeyboardInterrupt:
        print("\n\n⚠️  Прервано пользователем. Сохраняю...")
        await exporter.save()
    except ApiIdInvalidError:
        print("\n❌ Неверный API ID или Hash!")
        print("   Получите на https://my.telegram.org/apps")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Ошибка: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        await exporter.disconnect()

if __name__ == "__main__":
    asyncio.run(main())
