"""AI API endpoints"""
from fastapi import APIRouter, Depends, HTTPException, BackgroundTasks
from sqlalchemy.orm import Session
from decimal import Decimal
from typing import List

from ..core.database import get_db, TransactionDB, AIClassificationStatusDB, User
from ..core.security import get_current_user
from ..ai.classifier import classifier

router = APIRouter(prefix="/api/v1/ai", tags=["ai"])

@router.post("/classify/{transaction_id}")
async def classify_transaction(
    transaction_id: str,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """AI-классификация одной транзакции"""
    
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
        "reasoning": result.get("reasoning"),
        "needs_review": result.get("needs_review"),
        "is_exempt": result.get("is_exempt", False)
    }

@router.post("/classify-all")
async def classify_all_pending(
    background_tasks: BackgroundTasks,
    limit: int = 50,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Классифицировать все транзакции без категории"""
    
    pending = db.query(TransactionDB).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.ai_category.is_(None)
    ).limit(limit).all()
    
    results = []
    for tx in pending:
        try:
            result = await classifier.classify(
                description=tx.description,
                amount=float(tx.amount),
                counterparty=tx.counterparty,
                tx_type=tx.type.value
            )
            
            tx.ai_category = result.get("category")
            tx.ai_confidence = Decimal(str(result.get("confidence", 0)))
            tx.ai_reasoning = result.get("reasoning")
            tx.ai_status = AIClassificationStatusDB.NEEDS_REVIEW if result.get("needs_review") else AIClassificationStatusDB.CONFIRMED
            tx.ai_model = result.get("model", "qwen-plus")
            
            results.append({
                "id": tx.id,
                "category": result.get("category"),
                "confidence": result.get("confidence"),
                "needs_review": result.get("needs_review")
            })
        except Exception as e:
            results.append({"id": tx.id, "error": str(e)})
    
    db.commit()
    
    return {"classified": len(results), "results": results}

async def classify_transaction_background(tx_id: str, user_id: str):
    """Фоновая классификация (для новых транзакций)"""
    db = SessionLocal()
    try:
        tx = db.query(TransactionDB).filter(
            TransactionDB.id == tx_id,
            TransactionDB.user_id == user_id
        ).first()
        
        if tx and not tx.ai_category:
            result = await classifier.classify(
                description=tx.description,
                amount=float(tx.amount),
                counterparty=tx.counterparty,
                tx_type=tx.type.value
            )
            
            tx.ai_category = result.get("category")
            tx.ai_confidence = Decimal(str(result.get("confidence", 0)))
            tx.ai_reasoning = result.get("reasoning")
            tx.ai_status = AIClassificationStatusDB.NEEDS_REVIEW if result.get("needs_review") else AIClassificationStatusDB.CONFIRMED
            
            db.commit()
    finally:
        db.close()
