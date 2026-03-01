"""Kaspi Магазин интеграция для e-commerce"""
import httpx
from datetime import datetime, timedelta
from decimal import Decimal
from typing import List, Dict, Optional
from loguru import logger

from ..core.config import settings


class KaspiShopClient:
    """
    Клиент для Kaspi Магазин API
    
    Документация: https://kaspi.kz/merchant/manual
    """
    
    def __init__(self):
        self.api_key = settings.KASPI_API_KEY
        self.merchant_id = settings.KASPI_MERCHANT_ID
        self.base_url = "https://kaspi.kz"
        self.api_url = "https://api.kaspi.kz"
        self.enabled = bool(self.api_key and self.merchant_id)
    
    async def get_orders(
        self,
        status: Optional[str] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        page: int = 1,
        limit: int = 50
    ) -> Optional[Dict]:
        """
        Получение заказов из Kaspi Магазин
        
        Args:
            status: Статус заказа (new, confirmed, shipped, delivered, cancelled)
            start_date: Начало периода
            end_date: Конец периода
            page: Номер страницы
            limit: Количество на странице
        
        Returns:
            Dict с заказами
        """
        if not self.enabled:
            logger.warning("Kaspi Shop API not configured")
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                params = {
                    "merchantId": self.merchant_id,
                    "page": page,
                    "size": limit
                }
                
                if status:
                    params["status"] = status
                
                if start_date:
                    params["startDate"] = start_date.isoformat()
                
                if end_date:
                    params["endDate"] = end_date.isoformat()
                
                response = await client.get(
                    f"{self.api_url}/shop/v1/orders",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    },
                    params=params
                )
                
                if response.status_code == 200:
                    data = response.json()
                    logger.info(f"Kaspi Shop: получено {len(data.get('orders', []))} заказов")
                    return data
                else:
                    logger.error(f"Kaspi Shop API error: {response.status_code} - {response.text}")
                    return None
                    
        except Exception as e:
            logger.error(f"Kaspi Shop exception: {e}")
            return None
    
    async def get_order_details(self, order_id: str) -> Optional[Dict]:
        """Детали заказа"""
        if not self.enabled:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.api_url}/shop/v1/orders/{order_id}",
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
            logger.error(f"Kaspi order details exception: {e}")
            return None
    
    async def update_order_status(
        self,
        order_id: str,
        status: str,
        tracking_number: Optional[str] = None
    ) -> bool:
        """
        Обновление статуса заказа
        
        Args:
            order_id: ID заказа
            status: Новый статус
            tracking_number: Трек-номер для отправленных
        
        Returns:
            True если успешно
        """
        if not self.enabled:
            return False
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                data = {
                    "status": status,
                    "merchantId": self.merchant_id
                }
                
                if tracking_number:
                    data["trackingNumber"] = tracking_number
                
                response = await client.put(
                    f"{self.api_url}/shop/v1/orders/{order_id}/status",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    },
                    json=data
                )
                
                if response.status_code == 200:
                    logger.info(f"Kaspi order {order_id} status updated to {status}")
                    return True
                else:
                    logger.error(f"Kaspi status update error: {response.status_code}")
                    return False
                    
        except Exception as e:
            logger.error(f"Kaspi status update exception: {e}")
            return False
    
    async def get_products(self, page: int = 1, limit: int = 100) -> Optional[Dict]:
        """Получение списка товаров"""
        if not self.enabled:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.api_url}/shop/v1/products",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    },
                    params={
                        "merchantId": self.merchant_id,
                        "page": page,
                        "size": limit
                    }
                )
                
                if response.status_code == 200:
                    return response.json()
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"Kaspi products exception: {e}")
            return None
    
    async def update_product_price(self, product_id: str, price: float) -> bool:
        """Обновление цены товара"""
        if not self.enabled:
            return False
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.put(
                    f"{self.api_url}/shop/v1/products/{product_id}/price",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    },
                    json={
                        "price": price,
                        "currency": "KZT"
                    }
                )
                
                if response.status_code == 200:
                    logger.info(f"Kaspi product {product_id} price updated to {price}")
                    return True
                else:
                    return False
                    
        except Exception as e:
            logger.error(f"Kaspi price update exception: {e}")
            return False
    
    async def get_balance(self) -> Optional[Dict]:
        """Получение баланса продавца"""
        if not self.enabled:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.api_url}/shop/v1/merchant/{self.merchant_id}/balance",
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
            logger.error(f"Kaspi balance exception: {e}")
            return None
    
    def parse_order_to_transaction(self, order: Dict) -> Dict:
        """
        Конвертация заказа Kaspi в транзакцию
        
        Args:
            order: Данные заказа от Kaspi
        
        Returns:
            Dict для создания транзакции
        """
        items = order.get('items', [])
        total_amount = sum(item.get('price', 0) * item.get('quantity', 1) for item in items)
        
        return {
            'bank_transaction_id': order.get('id'),
            'bank_name': 'kaspi_shop',
            'date': datetime.fromisoformat(order.get('createdAt')),
            'amount': Decimal(str(total_amount)),
            'currency': 'KZT',
            'type': 'income',
            'counterparty': f"Kaspi Магазин (Заказ {order.get('number')})",
            'counterparty_inn': '881102061363',  # ИНН Kaspi
            'description': f"Заказ {order.get('number')} из Kaspi Магазин",
            'source': 'kaspi_shop',
            'kaspi_order_data': order  # Сохраняем оригинальные данные
        }


class KaspiWebhookHandler:
    """Обработчик вебхуков от Kaspi Магазин"""
    
    def __init__(self):
        self.client = KaspiShopClient()
    
    async def handle_order_created(self, order_data: Dict) -> bool:
        """Обработка нового заказа"""
        logger.info(f"Kaspi new order: {order_data.get('number')}")
        # TODO: Создать транзакцию в БД
        # TODO: Отправить уведомление
        return True
    
    async def handle_order_cancelled(self, order_data: Dict) -> bool:
        """Обработка отмены заказа"""
        logger.info(f"Kaspi order cancelled: {order_data.get('number')}")
        # TODO: Обновить транзакцию
        return True
    
    async def handle_status_changed(self, order_data: Dict) -> bool:
        """Обработка изменения статуса"""
        logger.info(f"Kaspi order status changed: {order_data.get('number')} -> {order_data.get('status')}")
        # TODO: Обновить статус в БД
        return True


# Глобальный экземпляр
kaspi_shop = KaspiShopClient()
kaspi_webhook = KaspiWebhookHandler()
