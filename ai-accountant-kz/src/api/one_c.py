"""API endpoints для 1С интеграции"""
from fastapi import APIRouter, Depends, HTTPException, Response, Query
from sqlalchemy.orm import Session
from sqlalchemy import func, extract
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Optional

from ..core.database import get_db, TransactionDB, TransactionTypeDB, Employee, User
from ..core.security import get_current_user
from ..utils.one_c_integration import exporter, importer

router = APIRouter(prefix="/api/v1/1c", tags=["1c"])

@router.get("/export/transactions")
async def export_transactions(
    start_date: str,
    end_date: str,
    format: str = Query("xml", regex="^(xml|json)$"),
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """
    Экспорт транзакций для 1С
    
    Args:
        start_date: YYYY-MM-DD
        end_date: YYYY-MM-DD
        format: xml или json
    """
    # Получение транзакций
    transactions = db.query(TransactionDB).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.date >= datetime.fromisoformat(start_date),
        TransactionDB.date <= datetime.fromisoformat(end_date)
    ).all()
    
    tx_list = [{
        "id": tx.id,
        "date": tx.date.isoformat(),
        "amount": float(tx.amount),
        "currency": tx.currency,
        "type": tx.type.value,
        "counterparty": tx.counterparty,
        "counterparty_inn": tx.counterparty_inn,
        "category": tx.ai_category or tx.manual_category,
        "description": tx.description,
        "document_number": None,
        "document_date": None
    } for tx in transactions]
    
    if format == "xml":
        xml_content = exporter.export_transactions_to_xml(tx_list)
        return Response(
            content=xml_content,
            media_type="application/xml",
            headers={
                "Content-Disposition": f"attachment; filename=transactions_{start_date}_{end_date}.xml"
            }
        )
    else:
        # JSON формат
        exchange_data = {
            "company": {
                "name": current_user.full_name or current_user.email,
                "inn": current_user.inn
            },
            "period": {
                "start": start_date,
                "end": end_date
            },
            "transactions": tx_list
        }
        
        import json
        return Response(
            content=json.dumps(exchange_data, ensure_ascii=False, indent=2),
            media_type="application/json",
            headers={
                "Content-Disposition": f"attachment; filename=transactions_{start_date}_{end_date}.json"
            }
        )

@router.post("/import/transactions")
async def import_transactions(
    file_content: str,
    format: str = Query("xml", regex="^(xml|json)$"),
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """
    Импорт транзакций из 1С
    
    Args:
        file_content: Содержимое файла
        format: xml или json
    """
    import json
    
    if format == "xml":
        data = importer.parse_1c_export(file_content)
    else:
        data = json.loads(file_content)
    
    transactions = data.get("transactions", [])
    
    imported = 0
    errors = []
    
    for tx_data in transactions:
        try:
            # Проверка на дубликат
            existing = db.query(TransactionDB).filter(
                TransactionDB.id == tx_data.get('id')
            ).first()
            
            if existing:
                errors.append(f"Duplicate: {tx_data.get('id')}")
                continue
            
            # Создание транзакции
            tx = TransactionDB(
                id=tx_data.get('id'),
                user_id=current_user.id,
                date=datetime.fromisoformat(tx_data.get('date')),
                amount=Decimal(str(tx_data.get('amount', 0))),
                type=TransactionTypeDB(tx_data.get('type', 'expense')),
                counterparty=tx_data.get('counterparty'),
                counterparty_inn=tx_data.get('inn') or tx_data.get('counterparty_inn'),
                description=tx_data.get('description', ''),
                ai_category=tx_data.get('category'),
                source="1c"
            )
            
            db.add(tx)
            imported += 1
            
        except Exception as e:
            errors.append(f"Error importing {tx_data.get('id')}: {str(e)}")
    
    db.commit()
    
    return {
        "imported": imported,
        "errors": errors,
        "total": len(transactions)
    }

@router.get("/export/employees")
async def export_employees(
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Экспорт сотрудников для 1С"""
    employees = db.query(Employee).filter(
        Employee.user_id == current_user.id,
        Employee.is_active == True
    ).all()
    
    emp_list = [{
        "id": emp.id,
        "full_name": emp.full_name,
        "inn": emp.inn,
        "position": emp.position,
        "salary": float(emp.salary)
    } for emp in employees]
    
    import json
    return {
        "company": {
            "name": current_user.full_name or current_user.email,
            "inn": current_user.inn
        },
        "employees": emp_list
    }

@router.get("/1c-exchange-script")
async def get_1c_exchange_script(
    current_user: User = Depends(get_current_user)
):
    """Получить скрипт для импорта в 1С"""
    script = exporter.generate_1c_import_script(
        f"https://ai-accountant.kz/api/v1/1c/export/transactions"
    )
    
    return Response(
        content=script,
        media_type="text/plain",
        headers={
            "Content-Disposition": "attachment; filename=import_to_1c.bsl"
        }
    )

@router.get("/config")
async def get_1c_config(
    current_user: User = Depends(get_current_user)
):
    """Конфигурация для подключения 1С"""
    return {
        "api_url": "https://ai-accountant.kz/api/v1/1c",
        "auth_type": "bearer",
        "supported_formats": ["xml", "json"],
        "endpoints": {
            "export_transactions": "/api/v1/1c/export/transactions",
            "import_transactions": "/api/v1/1c/import/transactions",
            "export_employees": "/api/v1/1c/export/employees"
        },
        "instructions": {
            "1c_accounting": "https://docs.ai-accountant.kz/1c/accounting",
            "1c_enterprise": "https://docs.ai-accountant.kz/1c/enterprise"
        }
    }
