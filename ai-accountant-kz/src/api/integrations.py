"""API endpoints для интеграций с банками и налоговой"""
from fastapi import APIRouter, Depends, HTTPException, BackgroundTasks, Request
from sqlalchemy.orm import Session
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Optional, Dict
import uuid

from ..core.database import get_db, User, TransactionDB, TransactionSourceDB, TaxDeclaration
from ..core.security import get_current_user
from ..core.tax_rules import TAX_RULES
from ..integrations.kaspi import kaspi_client, kaspi_webhook
from ..integrations.halyk import halyk_client, halyk_webhook
from ..integrations.tax_service import tax_service, declaration_generator
from ..tasks import sync_kaspi_transactions, sync_halyk_transactions, submit_tax_declaration

router = APIRouter(prefix="/api/v1/integrations", tags=["integrations"])

# ===== Kaspi Bank =====

@router.post("/kaspi/sync")
async def sync_kaspi(
    days: int = 7,
    background_tasks: BackgroundTasks = None,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Синхронизация транзакций Kaspi"""
    if not current_user.inn:
        raise HTTPException(status_code=400, detail="Укажите ИНН в профиле")
    
    # Запуск фоновой задачи
    task = sync_kaspi_transactions.delay(current_user.id, days)
    
    return {
        "status": "started",
        "task_id": task.id,
        "message": f"Синхронизация за {days} дней запущена"
    }

@router.get("/kaspi/balance")
async def get_kaspi_balance(
    current_user: User = Depends(get_current_user)
):
    """Получение баланса Kaspi"""
    balance = await kaspi_client.get_balance()
    if not balance:
        raise HTTPException(status_code=503, detail="Не удалось получить баланс")
    
    return balance

@router.post("/kaspi/webhook")
async def kaspi_webhook_endpoint(
    request: Request,
    background_tasks: BackgroundTasks
):
    """Webhook для уведомлений от Kaspi"""
    payload = await request.json()
    signature = request.headers.get("X-Kaspi-Signature", "")
    
    success = await kaspi_webhook.handle_notification(payload, signature)
    
    return {"status": "ok" if success else "error"}

# ===== Halyk Bank =====

@router.post("/halyk/sync")
async def sync_halyk(
    days: int = 7,
    background_tasks: BackgroundTasks = None,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Синхронизация транзакций Halyk"""
    if not current_user.inn:
        raise HTTPException(status_code=400, detail="Укажите ИНН в профиле")
    
    task = sync_halyk_transactions.delay(current_user.id, days)
    
    return {
        "status": "started",
        "task_id": task.id,
        "message": f"Синхронизация за {days} дней запущена"
    }

@router.get("/halyk/accounts")
async def get_halyk_accounts(
    current_user: User = Depends(get_current_user)
):
    """Получение счетов Halyk"""
    accounts = await halyk_client.get_accounts()
    if accounts is None:
        raise HTTPException(status_code=503, detail="Не удалось получить счета")
    
    return {"accounts": accounts}

@router.post("/halyk/webhook")
async def halyk_webhook_endpoint(
    request: Request,
    background_tasks: BackgroundTasks
):
    """Webhook для уведомлений от Halyk"""
    payload = await request.json()
    signature = request.headers.get("X-Halyk-Signature", "")
    
    success = await halyk_webhook.handle_notification(payload, signature)
    
    return {"status": "ok" if success else "error"}

# ===== Tax Service =====

@router.post("/tax/submit-declaration")
async def submit_declaration(
    period: str,
    background_tasks: BackgroundTasks = None,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Отправка налоговой декларации 101.02"""
    if not current_user.inn:
        raise HTTPException(status_code=400, detail="Укажите ИНН в профиле")
    
    # Получаем данные за период
    year = int(period.split("-")[0]) if "-" in period else int(period)
    
    from sqlalchemy import func, extract
    income_query = db.query(func.sum(TransactionDB.amount)).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == "income",
        extract('year', TransactionDB.date) == year
    )
    expense_query = db.query(func.sum(TransactionDB.amount)).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == "expense",
        extract('year', TransactionDB.date) == year
    )
    
    total_income = income_query.scalar() or Decimal("0")
    total_expense = expense_query.scalar() or Decimal("0")
    tax_amount = total_income * TAX_RULES.RATE
    
    # Запуск фоновой задачи
    task = submit_tax_declaration.delay(current_user.id, period)
    
    # Сохраняем декларацию
    declaration = TaxDeclaration(
        id=str(uuid.uuid4()),
        user_id=current_user.id,
        period=period,
        year=year,
        quarter=1,
        total_income=total_income,
        taxable_income=total_income,
        tax_rate=TAX_RULES.RATE,
        tax_amount=tax_amount,
        status="pending"
    )
    db.add(declaration)
    db.commit()
    
    return {
        "status": "started",
        "task_id": task.id,
        "declaration_id": declaration.id,
        "total_income": float(total_income),
        "total_expense": float(total_expense),
        "tax_amount": float(tax_amount)
    }

@router.get("/tax/declarations")
async def get_declarations(
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Список деклараций пользователя"""
    declarations = db.query(TaxDeclaration).filter(
        TaxDeclaration.user_id == current_user.id
    ).order_by(TaxDeclaration.year.desc(), TaxDeclaration.quarter.desc()).all()
    
    return [{
        "id": d.id,
        "period": d.period,
        "year": d.year,
        "quarter": d.quarter,
        "total_income": float(d.total_income),
        "tax_amount": float(d.tax_amount),
        "status": d.status,
        "submitted_at": d.submitted_at.isoformat() if d.submitted_at else None,
        "paid_at": d.paid_at.isoformat() if d.paid_at else None
    } for d in declarations]

@router.get("/tax/declaration/{declaration_id}")
async def get_declaration(
    declaration_id: str,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Детали декларации"""
    declaration = db.query(TaxDeclaration).filter(
        TaxDeclaration.id == declaration_id,
        TaxDeclaration.user_id == current_user.id
    ).first()
    
    if not declaration:
        raise HTTPException(status_code=404, detail="Декларация не найдена")
    
    return {
        "id": declaration.id,
        "period": declaration.period,
        "year": declaration.year,
        "quarter": declaration.quarter,
        "total_income": float(declaration.total_income),
        "taxable_income": float(declaration.taxable_income),
        "tax_rate": str(declaration.tax_rate),
        "tax_amount": float(declaration.tax_amount),
        "status": declaration.status,
        "submitted_at": declaration.submitted_at.isoformat() if declaration.submitted_at else None,
        "paid_at": declaration.paid_at.isoformat() if declaration.paid_at else None,
        "declaration_file": declaration.declaration_file
    }

@router.post("/tax/declaration/{declaration_id}/download")
async def download_declaration(
    declaration_id: str,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Скачать декларацию (XML/PDF)"""
    declaration = db.query(TaxDeclaration).filter(
        TaxDeclaration.id == declaration_id,
        TaxDeclaration.user_id == current_user.id
    ).first()
    
    if not declaration:
        raise HTTPException(status_code=404, detail="Декларация не найдена")
    
    # Генерация XML
    xml_content = declaration_generator.generate_xml(
        inn=current_user.inn,
        period=declaration.period,
        income=declaration.total_income,
        expense=declaration.total_income - declaration.tax_amount,
        tax_amount=declaration.tax_amount,
        taxpayer_name=current_user.full_name or current_user.email
    )
    
    return {
        "format": "xml",
        "content": xml_content,
        "filename": f"declaration_101.02_{declaration.period}.xml"
    }

# ===== Counterparty Check =====

@router.get("/counterparty/{inn}")
async def check_counterparty(
    inn: str,
    current_user: User = Depends(get_current_user)
):
    """Проверка контрагента по ИНН"""
    result = await tax_service.check_counterparty(inn)
    
    if not result:
        raise HTTPException(status_code=404, detail="Контрагент не найден")
    
    return {
        "inn": inn,
        "name": result.get("name"),
        "status": result.get("status"),
        "is_active": result.get("isActive", False),
        "registration_date": result.get("registrationDate"),
        "address": result.get("address")
    }
