"""Telegram Bot для AI-Бухгалтера"""
import asyncio
import logging
from datetime import datetime
from decimal import Decimal
from typing import Optional
from aiogram import Bot, Dispatcher, types
from aiogram.filters import Command, CommandStart
from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton
from loguru import logger

from ..core.config import settings

# Настройка логирования
logging.basicConfig(level=logging.INFO)

class AccountantBot:
    """Telegram бот для AI-Бухгалтера"""
    
    def __init__(self):
        self.token = settings.TELEGRAM_BOT_TOKEN
        self.enabled = bool(self.token)
        
        if self.enabled:
            self.bot = Bot(token=self.token)
            self.dp = Dispatcher()
            self._setup_handlers()
            logger.info("Telegram Bot инициализирован")
        else:
            self.bot = None
            self.dp = None
            logger.warning("Telegram Bot отключен (нет токена)")
    
    def _setup_handlers(self):
        """Настройка обработчиков команд"""
        self.dp.message(CommandStart())(self.cmd_start)
        self.dp.message(Command("help"))(self.cmd_help)
        self.dp.message(Command("balance"))(self.cmd_balance)
        self.dp.message(Command("add"))(self.cmd_add)
        self.dp.message(Command("report"))(self.cmd_report)
        self.dp.message(Command("settings"))(self.cmd_settings)
    
    async def cmd_start(self, message: types.Message):
        """Команда /start"""
        await message.answer(
            "🇰🇿 <b>AI-Бухгалтер для ИП</b>\n\n"
            "Я ваш персональный бухгалтерский помощник!\n\n"
            "📋 <b>Доступные команды:</b>\n"
            "/help - Помощь\n"
            "/balance - Баланс и сводка\n"
            "/add - Добавить транзакцию\n"
            "/report - Отчёт за период\n"
            "/settings - Настройки\n\n"
            "🌐 <b>Веб-версия:</b> ai-accountant.kz"
        )
    
    async def cmd_help(self, message: types.Message):
        """Команда /help"""
        await message.answer(
            "📖 <b>Помощь</b>\n\n"
            "💰 <b>Транзакции:</b>\n"
            "  /add income 100000 Оплата от клиента\n"
            "  /add expense 50000 Аренда офиса\n\n"
            "📊 <b>Отчёты:</b>\n"
            "  /balance - Текущая сводка\n"
            "  /report 2026-1 - За квартал\n\n"
            "⚙️ <b>Настройки:</b>\n"
            "  /settings - Изменить настройки"
        )
    
    async def cmd_balance(self, message: types.Message):
        """Команда /balance"""
        # TODO: Получить данные из БД по user_id
        await message.answer(
            "📊 <b>Ваша сводка</b>\n\n"
            "💰 Доходы: 0 ₸\n"
            "💸 Расходы: 0 ₸\n"
            "📈 Прибыль: 0 ₸\n"
            "📑 Налог (4%): 0 ₸\n\n"
            "_Данные обновлены_"
        )
    
    async def cmd_add(self, message: types.Message):
        """Команда /add"""
        # Парсинг: /add income 100000 Описание
        args = message.text.split(maxsplit=3)
        
        if len(args) < 4:
            await message.answer(
                "❌ <b>Ошибка формата</b>\n\n"
                "Используйте:\n"
                "/add income|expense СУММА ОПИСАНИЕ\n\n"
                "Пример:\n"
                "/add income 100000 Оплата от клиента"
            )
            return
        
        tx_type = args[1]  # income/expense
        amount = args[2]   # сумма
        description = args[3]  # описание
        
        if tx_type not in ['income', 'expense']:
            await message.answer("❌ Тип должен быть income или expense")
            return
        
        # TODO: Сохранить транзакцию в БД
        await message.answer(
            f"✅ <b>Транзакция добавлена</b>\n\n"
            f"Тип: {'💰 Доход' if tx_type == 'income' else '💸 Расход'}\n"
            f"Сумма: {amount} ₸\n"
            f"Описание: {description}"
        )
    
    async def cmd_report(self, message: types.Message):
        """Команда /report"""
        args = message.text.split()
        period = args[1] if len(args) > 1 else "2026"
        
        # TODO: Получить отчёт из БД
        await message.answer(
            f"📈 <b>Отчёт за {period}</b>\n\n"
            f"Доходы: 0 ₸\n"
            f"Расходы: 0 ₸\n"
            f"Налог: 0 ₸"
        )
    
    async def cmd_settings(self, message: types.Message):
        """Команда /settings"""
        keyboard = InlineKeyboardMarkup(inline_keyboard=[
            [InlineKeyboardButton(text="🔔 Уведомления", callback_data="settings_notifications")],
            [InlineKeyboardButton(text="🏦 Банки", callback_data="settings_banks")],
            [InlineKeyboardButton(text="📑 Налоговая", callback_data="settings_tax")],
        ])
        
        await message.answer("⚙️ <b>Настройки</b>", reply_markup=keyboard)
    
    async def send_notification(self, chat_id: str, message: str, parse_mode: str = "HTML"):
        """Отправка уведомления пользователю"""
        if not self.enabled:
            return False
        
        try:
            await self.bot.send_message(
                chat_id=chat_id,
                text=message,
                parse_mode=parse_mode
            )
            logger.info(f"Telegram notification sent to {chat_id}")
            return True
        except Exception as e:
            logger.error(f"Telegram send error: {e}")
            return False
    
    async def send_tax_reminder(self, chat_id: str, amount: float, deadline: str):
        """Напоминание об уплате налога"""
        message = (
            f"🧾 <b>Напоминание об уплате налога</b>\n\n"
            f"💰 Сумма: <b>{amount:,.2f} ₸</b>\n"
            f"📅 Срок до: <b>{deadline}</b>\n\n"
            f"Не забудьте оплатить вовремя! ⏰"
        )
        return await self.send_notification(chat_id, message)
    
    async def send_declaration_reminder(self, chat_id: str, deadline: str):
        """Напоминание о сдаче декларации"""
        message = (
            f"📋 <b>Напоминание о декларации</b>\n\n"
            f"📅 Срок сдачи: <b>{deadline}</b>\n\n"
            f"Пора сдать декларацию по упрощёнке! 📝"
        )
        return await self.send_notification(chat_id, message)
    
    async def send_transaction_alert(self, chat_id: str, tx_type: str, amount: float, description: str):
        """Уведомление о новой транзакции"""
        emoji = "💰" if tx_type == "income" else "💸"
        type_name = "Доход" if tx_type == "income" else "Расход"
        
        message = (
            f"{emoji} <b>Новая транзакция</b>\n\n"
            f"Тип: {type_name}\n"
            f"Сумма: {amount:,.2f} ₸\n"
            f"Описание: {description}"
        )
        return await self.send_notification(chat_id, message)
    
    async def start_polling(self):
        """Запуск бота (для отдельного процесса)"""
        if not self.enabled:
            return
        
        logger.info("Telegram Bot starting polling...")
        await self.dp.start_polling(self.bot)
    
    async def stop(self):
        """Остановка бота"""
        if self.bot:
            await self.bot.session.close()


# Глобальный экземпляр
bot = AccountantBot()

# Функции для использования в других модулях
async def send_notification(chat_id: str, message: str):
    return await bot.send_notification(chat_id, message)

async def send_tax_reminder(chat_id: str, amount: float, deadline: str):
    return await bot.send_tax_reminder(chat_id, amount, deadline)

async def send_declaration_reminder(chat_id: str, deadline: str):
    return await bot.send_declaration_reminder(chat_id, deadline)
