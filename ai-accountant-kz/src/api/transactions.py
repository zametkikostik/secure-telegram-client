"""API endpoints для транзакций с авторизацией"""
from fastapi import APIRouter, Depends, HTTPException, Query, status
from sqlalchemy.orm import Session
from sqlalchemy import desc, func
from datetime import datetime
from decimal import Decimal
from typing import List, Optional
import uuid

from ..core.database import get_db, TransactionDB, TransactionTypeDB, TransactionSourceDB, AIClassificationStatusDB, User
from ..core.security import get_current_user
from ..core.tax_rules import TAX_RULES

router = APIRouter(prefix="/api/v1/transactions", tags=["transactions"])

@router.get("/")
async def get_transactions(
    skip: int = Query(0, ge=0),
    limit: int = Query(100, ge=1, le=1000),
    tx_type: Optional[str] = None,
    start_date: Optional[str] = None,
    end_date: Optional[str] = None,
    search: Optional[str] = None,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Получение списка транзакций пользователя"""
    
    query = db.query(TransactionDB).filter(TransactionDB.user_id == current_user.id)
    
    # Фильтры
    if tx_type:
        query = query.filter(TransactionDB.type == tx_type)
    
    if start_date:
        query = query.filter(TransactionDB.date >= datetime.fromisoformat(start_date))
    
    if end_date:
        query = query.filter(TransactionDB.date <= datetime.fromisoformat(end_date))
    
    if search:
        search_filter = f"%{search}%"
        query = query.filter(
            (TransactionDB.description.ilike(search_filter)) |
            (TransactionDB.counterparty.ilike(search_filter))
        )
    
    transactions = query.order_by(desc(TransactionDB.date)).offset(skip).limit(limit).all()
    
    return [{
        "id": tx.id,
        "date": tx.date.isoformat(),
        "amount": float(tx.amount),
        "currency": tx.currency,
        "type": tx.type.value,
        "counterparty": tx.counterparty,
        "counterparty_inn": tx.counterparty_inn,
        "description": tx.description,
        "source": tx.source.value,
        "bank_name": tx.bank_name,
        "ai_category": tx.ai_category,
        "ai_confidence": float(tx.ai_confidence) if tx.ai_confidence else None,
        "ai_status": tx.ai_status.value,
        "ai_reasoning": tx.ai_reasoning,
        "manual_category": tx.manual_category,
        "category_confirmed": tx.category_confirmed,
        "created_at": tx.created_at.isoformat()
    } for tx in transactions]

@router.get("/{transaction_id}")
async def get_transaction(
    transaction_id: str,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Получение одной транзакции"""
    
    tx = db.query(TransactionDB).filter(
        TransactionDB.id == transaction_id,
        TransactionDB.user_id == current_user.id
    ).first()
    
    if not tx:
        raise HTTPException(status_code=404, detail="Транзакция не найдена")
    
    return {
        "id": tx.id,
        "date": tx.date.isoformat(),
        "amount": float(tx.amount),
        "currency": tx.currency,
        "type": tx.type.value,
        "counterparty": tx.counterparty,
        "description": tx.description,
        "source": tx.source.value,
        "ai_category": tx.ai_category,
        "ai_confidence": float(tx.ai_confidence) if tx.ai_confidence else None,
        "ai_status": tx.ai_status.value,
        "ai_reasoning": tx.ai_reasoning,
        "manual_category": tx.manual_category
    }

@router.post("/")
async def create_transaction(
    date: str,
    amount: float,
    type: str,
    description: str,
    counterparty: Optional[str] = None,
    source: str = "manual",
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Создание новой транзакции"""
    
    tx_id = str(uuid.uuid4())
    
    try:
        tx_type = TransactionTypeDB(type)
    except ValueError:
        raise HTTPException(status_code=400, detail="Неверный тип транзакции")
    
    try:
        tx_source = TransactionSourceDB(source)
    except ValueError:
        raise HTTPException(status_code=400, detail="Неверный источник")
    
    transaction = TransactionDB(
        id=tx_id,
        user_id=current_user.id,
        date=datetime.fromisoformat(date.replace('Z', '+00:00')),
        amount=Decimal(str(amount)),
        type=tx_type,
        counterparty=counterparty,
        description=description,
        source=tx_source
    )
    
    db.add(transaction)
    db.commit()
    db.refresh(transaction)
    
    # TODO: Запустить AI классификацию в фоне
    # await classify_transaction(transaction)
    
    return {"id": tx_id, "status": "created"}

@router.delete("/{transaction_id}")
async def delete_transaction(
    transaction_id: str,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Удаление транзакции"""
    
    tx = db.query(TransactionDB).filter(
        TransactionDB.id == transaction_id,
        TransactionDB.user_id == current_user.id
    ).first()
    
    if not tx:
        raise HTTPException(status_code=404, detail="Транзакция не найдена")
    
    db.delete(tx)
    db.commit()
    
    return {"status": "deleted"}

@router.put("/{transaction_id}/category")
async def update_category(
    transaction_id: str,
    category: str,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Обновление категории транзакции"""
    
    tx = db.query(TransactionDB).filter(
        TransactionDB.id == transaction_id,
        TransactionDB.user_id == current_user.id
    ).first()
    
    if not tx:
        raise HTTPException(status_code=404, detail="Транзакция не найдена")
    
    tx.manual_category = category
    tx.category_confirmed = True
    tx.ai_status = AIClassificationStatusDB.REJECTED  # Пользователь переопределил AI
    db.commit()
    
    return {"status": "updated", "category": category}

@router.post("/{transaction_id}/classify")
async def classify_single(
    transaction_id: str,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """AI-классификация одной транзакции"""
    
    from ..ai.classifier import classifier
    
    tx = db.query(TransactionDB).filter(
        TransactionDB.id == transaction_id,
        TransactionDB.user_id == current_user.id
    ).first()
    
    if not tx:
        raise HTTPException(status_code=404, detail="Транзакция не найдена")
    
    # Классифицируем
    result = await classifier.classify(
        description=tx.description,
        amount=float(tx.amount),
        counterparty=tx.counterparty,
        tx_type=tx.type.value
    )
    
    # Сохраняем
    tx.ai_category = result.get("category")
    tx.ai_confidence = Decimal(str(result.get("confidence", 0)))
    tx.ai_reasoning = result.get("reasoning")
    tx.ai_status = AIClassificationStatusDB.NEEDS_REVIEW if result.get("needs_review") else AIClassificationStatusDB.CONFIRMED
    tx.ai_model = result.get("model", "qwen-plus")
    
    db.commit()
    
    return {
        "transaction_id": transaction_id,
        "category": result.get("category"),
        "confidence": result.get("confidence"),
        "needs_review": result.get("needs_review")
    }
