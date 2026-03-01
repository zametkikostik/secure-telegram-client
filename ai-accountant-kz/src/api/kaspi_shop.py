"""API endpoints для Kaspi Магазин интеграции"""
from fastapi import APIRouter, Depends, HTTPException, Query, BackgroundTasks
from sqlalchemy.orm import Session
from datetime import datetime, timedelta
from typing import Optional

from ..core.database import get_db, TransactionDB, TransactionTypeDB, TransactionSourceDB, User
from ..core.security import get_current_user
from ..integrations.kaspi_shop import kaspi_shop, kaspi_webhook

router = APIRouter(prefix="/api/v1/integrations/kaspi-shop", tags=["kaspi-shop"])

@router.get("/orders")
async def get_kaspi_orders(
    status: Optional[str] = None,
    days: int = Query(7, ge=1, le=90),
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Получение заказов из Kaspi Магазин"""
    end_date = datetime.now()
    start_date = end_date - timedelta(days=days)
    
    orders = await kaspi_shop.get_orders(
        status=status,
        start_date=start_date,
        end_date=end_date
    )
    
    if orders is None:
        raise HTTPException(status_code=503, detail="Kaspi Shop API не настроен")
    
    return orders

@router.get("/orders/{order_id}")
async def get_kaspi_order(
    order_id: str,
    current_user: User = Depends(get_current_user)
):
    """Детали заказа"""
    order = await kaspi_shop.get_order_details(order_id)
    
    if not order:
        raise HTTPException(status_code=404, detail="Заказ не найден")
    
    return order

@router.post("/orders/{order_id}/status")
async def update_order_status(
    order_id: str,
    status: str,
    tracking_number: Optional[str] = None,
    current_user: User = Depends(get_current_user)
):
    """Обновление статуса заказа"""
    success = await kaspi_shop.update_order_status(
        order_id,
        status,
        tracking_number
    )
    
    if success:
        return {"status": "success", "message": f"Статус обновлён на {status}"}
    else:
        raise HTTPException(status_code=500, detail="Не удалось обновить статус")

@router.get("/products")
async def get_kaspi_products(
    page: int = 1,
    limit: int = 100,
    current_user: User = Depends(get_current_user)
):
    """Получение списка товаров"""
    products = await kaspi_shop.get_products(page, limit)
    
    if products is None:
        raise HTTPException(status_code=503, detail="Kaspi Shop API не настроен")
    
    return products

@router.put("/products/{product_id}/price")
async def update_product_price(
    product_id: str,
    price: float,
    current_user: User = Depends(get_current_user)
):
    """Обновление цены товара"""
    success = await kaspi_shop.update_product_price(product_id, price)
    
    if success:
        return {"status": "success", "message": f"Цена обновлена на {price} ₸"}
    else:
        raise HTTPException(status_code=500, detail="Не удалось обновить цену")

@router.get("/balance")
async def get_kaspi_balance(
    current_user: User = Depends(get_current_user)
):
    """Баланс продавца Kaspi"""
    balance = await kaspi_shop.get_balance()
    
    if balance is None:
        raise HTTPException(status_code=503, detail="Kaspi Shop API не настроен")
    
    return balance

@router.post("/sync")
async def sync_kaspi_orders(
    days: int = Query(7, ge=1, le=90),
    background_tasks: BackgroundTasks = None,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """
    Синхронизация заказов Kaspi Магазин
    
    Создаёт транзакции для всех новых заказов
    """
    end_date = datetime.now()
    start_date = end_date - timedelta(days=days)
    
    orders = await kaspi_shop.get_orders(
        start_date=start_date,
        end_date=end_date
    )
    
    if orders is None:
        raise HTTPException(status_code=503, detail="Kaspi Shop API не настроен")
    
    imported = 0
    for order in orders.get('orders', []):
        try:
            # Проверка на дубликат
            existing = db.query(TransactionDB).filter(
                TransactionDB.bank_transaction_id == order.get('id')
            ).first()
            
            if existing:
                continue
            
            # Конвертация в транзакцию
            tx_data = kaspi_shop.parse_order_to_transaction(order)
            
            transaction = TransactionDB(
                id=tx_data['bank_transaction_id'],
                user_id=current_user.id,
                date=tx_data['date'],
                amount=tx_data['amount'],
                currency=tx_data['currency'],
                type=tx_data['type'],
                counterparty=tx_data['counterparty'],
                counterparty_inn=tx_data['counterparty_inn'],
                description=tx_data['description'],
                source=TransactionSourceDB.KASPI
            )
            
            db.add(transaction)
            imported += 1
            
        except Exception as e:
            continue
    
    db.commit()
    
    return {
        "status": "success",
        "imported": imported,
        "total": len(orders.get('orders', []))
    }

@router.post("/webhook")
async def kaspi_shop_webhook(
    request: dict,
    background_tasks: BackgroundTasks
):
    """Webhook для уведомлений от Kaspi Магазин"""
    event_type = request.get('event')
    order_data = request.get('data', {})
    
    if event_type == 'order.created':
        await kaspi_webhook.handle_order_created(order_data)
    elif event_type == 'order.cancelled':
        await kaspi_webhook.handle_order_cancelled(order_data)
    elif event_type == 'order.status_changed':
        await kaspi_webhook.handle_status_changed(order_data)
    
    return {"status": "ok"}
