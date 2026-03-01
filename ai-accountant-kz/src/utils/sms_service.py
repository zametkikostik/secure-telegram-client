"""SMS сервис для уведомлений (Казахстан)"""
import httpx
from typing import Optional
from loguru import logger

from ..core.config import settings


class SMSService:
    """
    SMS сервис для Казахстана
    
    Поддерживаемые провайдеры:
    - SMS.kz (https://sms.kz)
    - Oxygencity (https://sms.oxygencity.com)
    - Twilio (международный)
    """
    
    def __init__(self):
        self.provider = getattr(settings, 'SMS_PROVIDER', 'sms_kz')
        self.api_key = getattr(settings, 'SMS_API_KEY', '')
        self.sender_id = getattr(settings, 'SMS_SENDER_ID', 'AI-Buhgalter')
        self.enabled = bool(self.api_key)
        
        # Настройки провайдеров
        self.providers = {
            'sms_kz': {
                'url': 'https://sms.kz/api/send',
                'params': self._sms_kz_params
            },
            'oxygencity': {
                'url': 'https://sms.oxygencity.com/ashulie/send',
                'params': self._oxygencity_params
            },
            'twilio': {
                'url': 'https://api.twilio.com/2010-04-01/Accounts/{sid}/Messages.json',
                'params': self._twilio_params
            }
        }
    
    def _sms_kz_params(self, phone: str, message: str) -> dict:
        """Параметры для SMS.kz"""
        return {
            'api_key': self.api_key,
            'phone': phone,
            'message': message,
            'sender': self.sender_id
        }
    
    def _oxygencity_params(self, phone: str, message: str) -> dict:
        """Параметры для Oxygencity"""
        return {
            'apiKey': self.api_key,
            'mobile': phone,
            'message': message,
            'from': self.sender_id
        }
    
    def _twilio_params(self, phone: str, message: str) -> dict:
        """Параметры для Twilio"""
        return {
            'To': phone,
            'From': self.sender_id,
            'Body': message
        }
    
    async def send_sms(self, phone: str, message: str) -> bool:
        """
        Отправка SMS
        
        Args:
            phone: Номер телефона (+77012345678)
            message: Текст сообщения
        
        Returns:
            True если отправлено успешно
        """
        if not self.enabled:
            logger.warning("SMS service not configured")
            return False
        
        try:
            provider_config = self.providers.get(self.provider, self.providers['sms_kz'])
            
            async with httpx.AsyncClient(timeout=30.0) as client:
                if self.provider == 'twilio':
                    # Twilio использует Basic Auth
                    response = await client.post(
                        provider_config['url'],
                        auth=(self.api_key.split(':')[0], self.api_key.split(':')[1]),
                        data=provider_config['params'](phone, message)
                    )
                else:
                    # Остальные используют query params
                    response = await client.get(
                        provider_config['url'],
                        params=provider_config['params'](phone, message)
                    )
                
                if response.status_code in [200, 201]:
                    logger.info(f"SMS sent to {phone}")
                    return True
                else:
                    logger.error(f"SMS send error: {response.status_code} - {response.text}")
                    return False
                    
        except Exception as e:
            logger.error(f"SMS exception: {e}")
            return False
    
    async def send_verification_code(self, phone: str, code: str) -> bool:
        """Отправка кода верификации"""
        message = f"Ваш код подтверждения: {code}. Действует 5 минут."
        return await self.send_sms(phone, message)
    
    async def send_tax_reminder(self, phone: str, amount: float, deadline: str) -> bool:
        """Напоминание об уплате налога"""
        message = f"🧾 AI-Бухгалтер: Напоминание об уплате налога {amount:,.0f}₸ до {deadline}. Не забудьте оплатить!"
        return await self.send_sms(phone, message)
    
    async def send_declaration_reminder(self, phone: str, deadline: str) -> bool:
        """Напоминание о сдаче декларации"""
        message = f"📋 AI-Бухгалтер: Напоминание! Срок сдачи декларации до {deadline}. Сдайте вовремя!"
        return await self.send_sms(phone, message)
    
    async def send_transaction_alert(self, phone: str, tx_type: str, amount: float) -> bool:
        """Уведомление о крупной транзакции"""
        emoji = "💰" if tx_type == "income" else "💸"
        type_name = "Доход" if tx_type == "income" else "Расход"
        message = f"{emoji} AI-Бухгалтер: {type_name} {amount:,.0f}₸. Баланс обновлён."
        return await self.send_sms(phone, message)


# Глобальный экземпляр
sms_service = SMSService()
