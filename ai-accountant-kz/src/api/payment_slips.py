"""API endpoints для генерации платёжных квитанций"""
from fastapi import APIRouter, Depends, HTTPException
from fastapi.responses import StreamingResponse
from sqlalchemy.orm import Session
from typing import Optional
import io
import json

from ..core.database import get_db
from ..models.payment_slips import (
    PaymentSlipRequest,
    PaymentSlipsResponse,
    PaymentSlip
)
from ..services.payment_slips import payment_slips_generator

router = APIRouter(prefix="/api/v1/tax", tags=["tax-payment-slips"])


@router.post("/payment-slips", response_model=PaymentSlipsResponse)
async def generate_payment_slips(
    request: PaymentSlipRequest,
    db: Session = Depends(get_db)
):
    """
    Генерация платёжных квитанций
    
    **Возвращает:**
    - Список платёжек с КБК и суммами
    - общую сумму
    - сроки уплаты
    
    **Пример запроса:**
    ```json
    {
        "taxpayer_name": "Иванов Иван Иванович",
        "taxpayer_inn": "123456789012",
        "taxpayer_address": "г. Алматы, ул. Абая 10",
        "bank_name": "Kaspi Bank",
        "bank_bik": "KASPKZKA",
        "account_number": "KZ123456789012345678",
        "ie_payments": true,
        "employee_payments": false,
        "period": "02/2026"
    }
    ```
    """
    try:
        result = payment_slips_generator.generate_slips(request)
        return result
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/payment-slips/pdf")
async def generate_payment_slips_pdf(
    request: PaymentSlipRequest,
    db: Session = Depends(get_db)
):
    """
    Генерация PDF с платёжными квитанциями
    
    Возвращает PDF файл для скачивания
    """
    try:
        # Генерация платёжек
        slips_response = payment_slips_generator.generate_slips(request)
        
        # Генерация PDF
        pdf_bytes = payment_slips_generator.generate_pdf(slips_response)
        
        # Возврат файла
        return StreamingResponse(
            io.BytesIO(pdf_bytes),
            media_type="application/pdf",
            headers={
                "Content-Disposition": f"attachment; filename=payment_slips_{request.period.replace('/', '_')}.pdf"
            }
        )
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get("/payment-slips/kbk-info")
async def get_kbks_info():
    """
    Информация о КБК для платежей
    
    Возвращает справочник КБК с названиями
    """
    return {
        "kbk": payment_slips_generator.KBK_NAMES,
        "recipient": {
            "name": payment_slips_generator.RECIPIENT_NAME,
            "bik": payment_slips_generator.RECIPIENT_BIK,
            "kbe": payment_slips_generator.RECIPIENT_KBE
        },
        "ie_contributions": {
            "opv": {"kbk": "183102", "rate": "10%", "base": "1 МЗП"},
            "so": {"kbk": "183101", "rate": "5%", "base": "1 МЗП"},
            "vosms": {"kbk": "122101", "rate": "5%", "base": "1.4 МЗП"},
            "opvr": {"kbk": "183110", "rate": "3.5%", "base": "1 МЗП"}
        },
        "employee_contributions": {
            "deductions": {
                "ipn": {"kbk": "101201", "rate": "10%"},
                "opv": {"kbk": "183102", "rate": "10%"},
                "vosms": {"kbk": "122101", "rate": "2%"}
            },
            "employer": {
                "so": {"kbk": "183101", "rate": "5%"},
                "oosms": {"kbk": "122101", "rate": "3%"},
                "opvr": {"kbk": "183110", "rate": "2.5%"}
            }
        },
        "uzd": {
            "ipn": {"kbk": "101202", "rate": "2%"},
            "social_tax": {"kbk": "103101", "rate": "2%"}
        }
    }


@router.post("/payment-slips/quick")
async def generate_quick_payment_slip(
    taxpayer_name: str,
    taxpayer_inn: str,
    period: str,
    include_ie: bool = True,
    include_employee: bool = False,
    employee_salary_gross: float = 0,
    db: Session = Depends(get_db)
):
    """
    Быстрая генерация платёжки для ИП
    
    - **taxpayer_name**: ФИО ИП
    - **taxpayer_inn**: ИИН ИП
    - **period**: Период (месяц/год)
    - **include_ie**: Включить платежи ИП за себя
    - **include_employee**: Включить платежи за сотрудников
    - **employee_salary_gross**: Оклад сотрудника (грязными)
    """
    request = PaymentSlipRequest(
        taxpayer_name=taxpayer_name,
        taxpayer_inn=taxpayer_inn,
        taxpayer_address="Не указан",
        bank_name="Не указан",
        bank_bik="Не указан",
        account_number="Не указан",
        ie_payments=include_ie,
        employee_payments=include_employee,
        employee_salary_gross=employee_salary_gross,
        period=period
    )
    
    result = payment_slips_generator.generate_slips(request)
    return result
