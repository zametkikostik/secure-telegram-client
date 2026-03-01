"""API endpoints для генерации документов"""
from fastapi import APIRouter, Depends, HTTPException, Response, Query
from sqlalchemy.orm import Session
from datetime import datetime
from decimal import Decimal
from typing import List, Optional

from ..core.database import get_db, TransactionDB, User
from ..core.security import get_current_user
from ..utils.documents import doc_generator

router = APIRouter(prefix="/api/v1/documents", tags=["documents"])

@router.post("/invoice")
async def generate_invoice(
    buyer_name: str,
    buyer_inn: str,
    items: str,  # JSON string
    total: float,
    vat: str = "Без НДС",
    invoice_number: Optional[str] = None,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """
    Генерация счёта на оплату
    
    Args:
        items: JSON [{"name": "Услуга 1", "quantity": 1, "unit": "шт", "price": 100000}]
    """
    import json
    
    try:
        items_data = json.loads(items)
    except:
        raise HTTPException(status_code=400, detail="Неверный формат items")
    
    # Номер счёта
    if not invoice_number:
        count = db.query(TransactionDB).count()
        invoice_number = f"{count + 1:04d}"
    
    # Данные продавца (из профиля пользователя)
    seller_data = {
        "name": current_user.full_name or f"ИП {current_user.email}",
        "inn": current_user.inn or "Не указан",
        "bank": "Kaspi Bank",
        "bik": "123456789",
        "account": "1234567890123456"
    }
    
    # Данные покупателя
    buyer_data = {
        "name": buyer_name,
        "inn": buyer_inn
    }
    
    # Генерация PDF
    pdf_data = {
        "invoice_number": invoice_number,
        "invoice_date": datetime.now().strftime("%d.%m.%Y"),
        "seller": seller_data,
        "buyer": buyer_data,
        "items": items_data,
        "total": total,
        "vat": vat,
        "signature": current_user.full_name or "ИП"
    }
    
    pdf_bytes = doc_generator.generate_invoice(pdf_data)
    
    return Response(
        content=pdf_bytes,
        media_type="application/pdf",
        headers={
            "Content-Disposition": f"attachment; filename=invoice_{invoice_number}.pdf"
        }
    )

@router.post("/act")
async def generate_act(
    buyer_name: str,
    buyer_inn: str,
    services: str,  # JSON string
    total: float,
    period: str = None,
    act_number: Optional[str] = None,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Генерация акта выполненных работ"""
    import json
    
    try:
        services_data = json.loads(services)
    except:
        raise HTTPException(status_code=400, detail="Неверный формат services")
    
    if not act_number:
        count = db.query(TransactionDB).count()
        act_number = f"{count + 1:04d}"
    
    if not period:
        period = datetime.now().strftime("%B %Y")
    
    seller_data = {
        "name": current_user.full_name or f"ИП {current_user.email}",
        "inn": current_user.inn or "Не указан"
    }
    
    buyer_data = {
        "name": buyer_name,
        "inn": buyer_inn
    }
    
    pdf_data = {
        "act_number": act_number,
        "act_date": datetime.now().strftime("%d.%m.%Y"),
        "period": period,
        "seller": seller_data,
        "buyer": buyer_data,
        "services": services_data,
        "total": total,
        "signature": current_user.full_name or "ИП"
    }
    
    pdf_bytes = doc_generator.generate_act(pdf_data)
    
    return Response(
        content=pdf_bytes,
        media_type="application/pdf",
        headers={
            "Content-Disposition": f"attachment; filename=act_{act_number}.pdf"
        }
    )

@router.get("/templates")
async def get_document_templates():
    """Шаблоны документов"""
    return {
        "invoice": {
            "name": "Счёт на оплату",
            "fields": ["buyer_name", "buyer_inn", "items", "total", "vat"],
            "example": {
                "buyer_name": "ТОО «Ромашка»",
                "buyer_inn": "123456789012",
                "items": [{"name": "Услуга 1", "quantity": 1, "unit": "шт", "price": 100000}],
                "total": 100000,
                "vat": "Без НДС"
            }
        },
        "act": {
            "name": "Акт выполненных работ",
            "fields": ["buyer_name", "buyer_inn", "services", "total", "period"],
            "example": {
                "buyer_name": "ТОО «Ромашка»",
                "buyer_inn": "123456789012",
                "services": [{"name": "Консультационные услуги", "amount": 100000}],
                "total": 100000,
                "period": "Февраль 2026"
            }
        }
    }

@router.get("/invoice/{invoice_id}")
async def get_invoice(
    invoice_id: str,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Получение данных счёта (для предзаполнения)"""
    # TODO: Получить из БД
    return {
        "invoice_number": invoice_id,
        "invoice_date": datetime.now().strftime("%d.%m.%Y"),
        "seller": {
            "name": current_user.full_name,
            "inn": current_user.inn
        }
    }
