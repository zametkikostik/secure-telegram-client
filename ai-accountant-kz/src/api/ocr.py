"""API endpoints для OCR и загрузки файлов"""
from fastapi import APIRouter, Depends, HTTPException, UploadFile, File, Form
from sqlalchemy.orm import Session
from datetime import datetime
from decimal import Decimal
import uuid
import base64

from ..core.database import get_db, TransactionDB, TransactionTypeDB, TransactionSourceDB, User
from ..core.security import get_current_user
from ..utils.ocr import receipt_processor

router = APIRouter(prefix="/api/v1/ocr", tags=["ocr"])

@router.post("/upload")
async def upload_receipt(
    file: UploadFile = File(...),
    create_transaction: bool = Form(default=False),
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """
    Загрузка чека для распознавания
    
    Returns:
        {
            "success": true,
            "transaction_data": {...},
            "raw_text": "...",
            "confidence": 0.9
        }
    """
    # Проверка типа файла
    if not file.content_type.startswith('image/'):
        raise HTTPException(status_code=400, detail="Только изображения")
    
    # Чтение файла
    image_data = await file.read()
    
    # Обработка
    result = await receipt_processor.process_receipt(image_data)
    
    # Если нужно создать транзакцию автоматически
    if create_transaction and result['success'] and result['transaction_data']:
        tx_data = result['transaction_data']
        
        transaction = TransactionDB(
            id=str(uuid.uuid4()),
            user_id=current_user.id,
            date=datetime.fromisoformat(tx_data['date']) if tx_data.get('date') else datetime.now(),
            amount=Decimal(str(tx_data['amount'])),
            type=TransactionTypeDB(tx_data['type']),
            counterparty=tx_data.get('counterparty'),
            counterparty_inn=tx_data.get('counterparty_inn'),
            description=tx_data.get('description', 'Чек'),
            source=TransactionSourceDB.MANUAL,  # OCR как источник
            ai_category=None,
            ai_confidence=Decimal(str(result['confidence'])),
            ai_reasoning=f"OCR распознал: {result['raw_text'][:200]}"
        )
        
        db.add(transaction)
        db.commit()
        db.refresh(transaction)
        
        result['transaction_id'] = transaction.id
    
    return result

@router.post("/recognize-base64")
async def recognize_base64(
    image_base64: str,
    current_user: User = Depends(get_current_user)
):
    """Распознавание чека из base64"""
    try:
        # Декодирование base64
        image_data = base64.b64decode(image_base64.split(',')[1] if ',' in image_base64 else image_base64)
        
        result = await receipt_processor.process_receipt(image_data)
        return result
        
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.get("/supported-formats")
async def get_supported_formats():
    """Поддерживаемые форматы чеков"""
    return {
        "formats": ["JPEG", "PNG", "WEBP", "HEIC"],
        "max_size_mb": 10,
        "features": [
            "Распознавание суммы",
            "Распознавание даты",
            "Распознавание магазина",
            "Распознавание ИНН",
            "Список товаров",
            "Тип оплаты"
        ]
    }
