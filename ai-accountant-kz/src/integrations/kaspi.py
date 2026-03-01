"""Kaspi Bank API интеграция"""
import httpx
from datetime import datetime, timedelta
from decimal import Decimal
from typing import List, Optional, Dict
from loguru import logger

from ..core.config import settings


class KaspiBankClient:
    """Клиент для Kaspi Pay API"""
    
    def __init__(self):
        self.api_key = settings.KASPI_API_KEY
        self.merchant_id = settings.KASPI_MERCHANT_ID
        self.base_url = "https://api.kaspi.kz"
        self.enabled = bool(self.api_key) and bool(self.merchant_id)
    
    async def get_transactions(
        self,
        start_date: datetime,
        end_date: datetime,
        page: int = 1,
        limit: int = 100
    ) -> Optional[Dict]:
        """Получение транзакций из Kaspi"""
        if not self.enabled:
            logger.warning("Kaspi API не настроен")
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.base_url}/v1/merchant/{self.merchant_id}/transactions",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    },
                    params={
                        "startDate": start_date.isoformat(),
                        "endDate": end_date.isoformat(),
                        "page": page,
                        "limit": limit
                    }
                )
                
                if response.status_code == 200:
                    data = response.json()
                    logger.info(f"Kaspi: получено {len(data.get('transactions', []))} транзакций")
                    return data
                else:
                    logger.error(f"Kaspi API error: {response.status_code} - {response.text}")
                    return None
                    
        except Exception as e:
            logger.error(f"Kaspi API exception: {e}")
            return None
    
    async def get_balance(self) -> Optional[Dict]:
        """Получение баланса счета"""
        if not self.enabled:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.base_url}/v1/merchant/{self.merchant_id}/balance",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    }
                )
                
                if response.status_code == 200:
                    return response.json()
                else:
                    logger.error(f"Kaspi balance error: {response.status_code}")
                    return None
                    
        except Exception as e:
            logger.error(f"Kaspi balance exception: {e}")
            return None
    
    async def get_payment_details(self, payment_id: str) -> Optional[Dict]:
        """Детали платежа"""
        if not self.enabled:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.base_url}/v1/payments/{payment_id}",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    }
                )
                
                if response.status_code == 200:
                    return response.json()
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"Kaspi payment details exception: {e}")
            return None
    
    def parse_transaction(self, tx: Dict) -> Dict:
        """Парсинг транзакции Kaspi в формат приложения"""
        amount = Decimal(str(tx.get('amount', 0)))
        tx_type = 'income' if tx.get('type') == 'credit' else 'expense'
        
        return {
            'bank_transaction_id': tx.get('id'),
            'bank_name': 'kaspi',
            'date': datetime.fromisoformat(tx.get('date')),
            'amount': amount,
            'currency': tx.get('currency', 'KZT'),
            'type': tx_type,
            'counterparty': tx.get('counterpartyName'),
            'counterparty_inn': tx.get('counterpartyIin'),
            'description': tx.get('description', ''),
            'source': 'kaspi',
            'kaspi_data': tx  # Сохраняем оригинальные данные
        }


class KaspiWebhookHandler:
    """Обработчик вебхуков от Kaspi"""
    
    def __init__(self):
        self.client = KaspiBankClient()
    
    async def handle_notification(self, payload: Dict, signature: str) -> bool:
        """Обработка уведомления от Kaspi"""
        # Проверка подписи
        if not self._verify_signature(payload, signature):
            logger.error("Invalid Kaspi webhook signature")
            return False
        
        event_type = payload.get('event')
        data = payload.get('data', {})
        
        logger.info(f"Kaspi webhook event: {event_type}")
        
        if event_type == 'payment.received':
            await self._handle_payment_received(data)
        elif event_type == 'payment.refunded':
            await self._handle_payment_refunded(data)
        
        return True
    
    def _verify_signature(self, payload: Dict, signature: str) -> bool:
        """Проверка подписи вебхука"""
        # TODO: Реализовать проверку подписи
        return True
    
    async def _handle_payment_received(self, data: Dict):
        """Обработка входящего платежа"""
        logger.info(f"Kaspi payment received: {data.get('id')}")
        # TODO: Создать транзакцию в БД
    
    async def _handle_payment_refunded(self, data: Dict):
        """Обработка возврата платежа"""
        logger.info(f"Kaspi payment refunded: {data.get('id')}")
        # TODO: Обновить транзакцию в БД


# Глобальный экземпляр
kaspi_client = KaspiBankClient()
kaspi_webhook = KaspiWebhookHandler()
