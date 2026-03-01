"""Halyk Bank Open Banking API интеграция"""
import httpx
from datetime import datetime, timedelta
from decimal import Decimal
from typing import List, Optional, Dict
from loguru import logger

from ..core.config import settings


class HalykBankClient:
    """Клиент для Halyk Bank Open Banking API"""
    
    def __init__(self):
        self.api_key = settings.HALYK_API_KEY
        self.client_id = settings.HALYK_CLIENT_ID
        self.base_url = "https://api.halykbank.kz"
        self.auth_url = "https://auth.halykbank.kz"
        self.enabled = bool(self.api_key) and bool(self.client_id)
        self._access_token: Optional[str] = None
        self._token_expires: Optional[datetime] = None
    
    async def _get_access_token(self) -> Optional[str]:
        """Получение access токена"""
        if self._access_token and self._token_expires:
            if datetime.now() < self._token_expires - timedelta(minutes=5):
                return self._access_token
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(
                    f"{self.auth_url}/oauth/token",
                    data={
                        "grant_type": "client_credentials",
                        "client_id": self.client_id,
                        "client_secret": self.api_key,
                        "scope": "transactions accounts"
                    }
                )
                
                if response.status_code == 200:
                    data = response.json()
                    self._access_token = data['access_token']
                    expires_in = data.get('expires_in', 3600)
                    self._token_expires = datetime.now() + timedelta(seconds=expires_in)
                    logger.info("Halyk: получен новый access token")
                    return self._access_token
                else:
                    logger.error(f"Halyk auth error: {response.status_code}")
                    return None
                    
        except Exception as e:
            logger.error(f"Halyk auth exception: {e}")
            return None
    
    async def get_accounts(self) -> Optional[List[Dict]]:
        """Получение списка счетов"""
        token = await self._get_access_token()
        if not token:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.base_url}/v1/accounts",
                    headers={
                        "Authorization": f"Bearer {token}",
                        "Content-Type": "application/json"
                    }
                )
                
                if response.status_code == 200:
                    data = response.json()
                    return data.get('accounts', [])
                else:
                    logger.error(f"Halyk accounts error: {response.status_code}")
                    return None
                    
        except Exception as e:
            logger.error(f"Halyk accounts exception: {e}")
            return None
    
    async def get_transactions(
        self,
        account_id: str,
        start_date: datetime,
        end_date: datetime,
        limit: int = 100
    ) -> Optional[List[Dict]]:
        """Получение транзакций по счету"""
        token = await self._get_access_token()
        if not token:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.base_url}/v1/accounts/{account_id}/transactions",
                    headers={
                        "Authorization": f"Bearer {token}",
                        "Content-Type": "application/json"
                    },
                    params={
                        "fromDate": start_date.strftime("%Y-%m-%d"),
                        "toDate": end_date.strftime("%Y-%m-%d"),
                        "limit": limit
                    }
                )
                
                if response.status_code == 200:
                    data = response.json()
                    transactions = data.get('transactions', [])
                    logger.info(f"Halyk: получено {len(transactions)} транзакций")
                    return transactions
                else:
                    logger.error(f"Halyk transactions error: {response.status_code}")
                    return None
                    
        except Exception as e:
            logger.error(f"Halyk transactions exception: {e}")
            return None
    
    async def get_account_balance(self, account_id: str) -> Optional[Dict]:
        """Получение баланса счета"""
        token = await self._get_access_token()
        if not token:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.base_url}/v1/accounts/{account_id}/balance",
                    headers={
                        "Authorization": f"Bearer {token}",
                        "Content-Type": "application/json"
                    }
                )
                
                if response.status_code == 200:
                    return response.json()
                else:
                    logger.error(f"Halyk balance error: {response.status_code}")
                    return None
                    
        except Exception as e:
            logger.error(f"Halyk balance exception: {e}")
            return None
    
    def parse_transaction(self, tx: Dict, account_id: str) -> Dict:
        """Парсинг транзакции Halyk в формат приложения"""
        amount = Decimal(str(tx.get('amount', 0)))
        tx_type = 'income' if tx.get('type') == 'CREDIT' else 'expense'
        
        return {
            'bank_transaction_id': tx.get('transactionId'),
            'bank_name': 'halyk',
            'date': datetime.fromisoformat(tx.get('valueDate')),
            'amount': abs(amount),
            'currency': tx.get('currency', 'KZT'),
            'type': tx_type,
            'counterparty': tx.get('counterpartyName'),
            'counterparty_inn': tx.get('counterpartyIin'),
            'description': tx.get('description', ''),
            'source': 'halyk',
            'halyk_account_id': account_id,
            'halyk_data': tx  # Сохраняем оригинальные данные
        }


class HalykWebhookHandler:
    """Обработчик вебхуков от Halyk Bank"""
    
    def __init__(self):
        self.client = HalykBankClient()
    
    async def handle_notification(self, payload: Dict, signature: str) -> bool:
        """Обработка уведомления от Halyk"""
        if not self._verify_signature(payload, signature):
            logger.error("Invalid Halyk webhook signature")
            return False
        
        event_type = payload.get('eventType')
        data = payload.get('data', {})
        
        logger.info(f"Halyk webhook event: {event_type}")
        
        if event_type == 'transaction.created':
            await self._handle_transaction_created(data)
        elif event_type == 'account.balance.changed':
            await self._handle_balance_changed(data)
        
        return True
    
    def _verify_signature(self, payload: Dict, signature: str) -> bool:
        """Проверка подписи вебхука"""
        # TODO: Реализовать проверку подписи
        return True
    
    async def _handle_transaction_created(self, data: Dict):
        """Обработка новой транзакции"""
        logger.info(f"Halyk transaction created: {data.get('transactionId')}")
    
    async def _handle_balance_changed(self, data: Dict):
        """Обработка изменения баланса"""
        logger.info(f"Halyk balance changed: {data.get('accountId')}")


# Глобальный экземпляр
halyk_client = HalykBankClient()
halyk_webhook = HalykWebhookHandler()
