"""Telegram уведомления"""
import httpx
from loguru import logger
from typing import Optional

from ..core.config import settings

class TelegramNotifier:
    """Класс для отправки уведомлений в Telegram"""
    
    def __init__(self):
        self.bot_token = settings.TELEGRAM_BOT_TOKEN
        self.chat_id = settings.TELEGRAM_CHAT_ID
        self.enabled = settings.TELEGRAM_ENABLED and bool(self.bot_token) and bool(self.chat_id)
        self.base_url = f"https://api.telegram.org/bot{self.bot_token}"
    
    async def send_message(self, message: str, parse_mode: str = "HTML") -> bool:
        """Отправка сообщения"""
        if not self.enabled:
            logger.debug("Telegram уведомления отключены")
            return False
        
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                response = await client.post(
                    f"{self.base_url}/sendMessage",
                    json={
                        "chat_id": self.chat_id,
                        "text": message,
                        "parse_mode": parse_mode
                    }
                )
                
                if response.status_code == 200:
                    logger.info("Telegram сообщение отправлено")
                    return True
                else:
                    logger.error(f"Telegram API error: {response.text}")
                    return False
                    
        except Exception as e:
            logger.error(f"Ошибка отправки Telegram: {e}")
            return False
    
    async def send_tax_reminder(self, tax_amount: float, deadline: str) -> bool:
        """Напоминание об уплате налога"""
        message = f"""
🧾 <b>Напоминание об уплате налога</b>

💰 Сумма: <b>{tax_amount:,.2f} ₸</b>
📅 Срок до: <b>{deadline}</b>

Не забудьте оплатить вовремя! ⏰
        """.strip()
        
        return await self.send_message(message)
    
    async def send_declaration_reminder(self, deadline: str) -> bool:
        """Напоминание о сдаче декларации"""
        message = f"""
📋 <b>Напоминание о декларации</b>

📅 Срок сдачи: <b>{deadline}</b>

Пора сдать декларацию по упрощёнке! 📝
        """.strip()
        
        return await self.send_message(message)
    
    async def send_limit_warning(self, current_income: float, limit: float) -> bool:
        """Предупреждение о приближении к лимиту"""
        percent = (current_income / limit) * 100
        
        message = f"""
⚠️ <b>Внимание: приближение к лимиту дохода</b>

Текущий доход: <b>{current_income:,.2f} ₸</b>
Лимит: <b>{limit:,.2f} ₸</b>
Использовано: <b>{percent:.1f}%</b>

При превышении лимита вы потеряете право на упрощёнку! 🚨
        """.strip()
        
        return await self.send_message(message)
    
    async def send_transaction_summary(self, income: float, expense: float, count: int) -> bool:
        """Ежедневная сводка по транзакциям"""
        message = f"""
📊 <b>Ежедневная сводка</b>

✅ Транзакций: {count}
📈 Доходы: {income:,.2f} ₸
📉 Расходы: {expense:,.2f} ₸

Хорошего дня! 🌟
        """.strip()
        
        return await self.send_message(message)

# Глобальный экземпляр
notifier = TelegramNotifier()
