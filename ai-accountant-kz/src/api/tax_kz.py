"""API endpoints для расчёта налогов ИП Казахстан 2026"""
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from typing import Optional

from ..core.database import get_db
from ..models.tax_kz import (
    TaxCalculationRequest,
    TaxCalculationResponse,
    EmployeeData,
    TaxRateInfo
)
from ..services.tax_service_kz import tax_service_kz, KZTaxService2026

router = APIRouter(prefix="/api/v1/tax", tags=["tax-kz-2026"])


@router.post("/calculate-kz-2026", response_model=TaxCalculationResponse)
async def calculate_tax_kz_2026(
    request: TaxCalculationRequest,
    db: Session = Depends(get_db)
):
    """
    Расчёт налогов для ИП на упрощенке (Казахстан, 2026)
    
    **Константы 2026:**
    - МЗП: 85 000 ₸
    - МРП: 4 325 ₸
    - Упрощенка: 4% (ИПН 2% + СН 2%)
    
    **Платежи ИП за себя (в месяц):**
    - ОПВ 10%: 8 500 ₸
    - СО 5%: 4 250 ₸
    - ВОСМС 5% от 1.4 МЗП: 5 950 ₸
    - ОПВР 3.5%: 2 975 ₸
    - **Итого: 21 675 ₸/мес**
    
    **Сотрудники:**
    - Удержания: ИПН 10%, ОПВ 10%, ВОСМС 2%
    - Работодатель: СО 5%, ООСМС 3%, ОПВР 3.5%
    
    **Упрощенка (910.00):**
    - Ставка 4% от дохода
    - Соц.налог уменьшается на СО
    - Срок уплаты: 25 августа / 25 февраля
    - Срок сдачи: 15 августа / 15 февраля
    """
    try:
        result = tax_service_kz.calculate(request)
        return result
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.get("/rates-info-2026")
async def get_tax_rates_info_2026():
    """
    Информация о налоговых ставках 2026
    
    Возвращает актуальные константы и ставки:
    - МЗП, МРП
    - Ставки взносов ИП
    - Ставки взносов сотрудников
    - Ставки упрощенки
    - Лимиты (НДС, упрощенка)
    """
    return tax_service_kz.get_rates_info()


@router.post("/calculate-employee")
async def calculate_employee_tax(
    salary_net: float,
    has_children: bool = False,
    has_disability: bool = False,
    mzp: Optional[float] = None
):
    """
    Расчёт налогов для одного сотрудника
    
    - **salary_net**: Оклад на руки (тенге)
    - **has_children**: Есть ли дети (влияет на СИЗ)
    - **has_disability**: Есть ли инвалидность
    - **mzp**: МЗП (по умолчанию 85 000)
    
    Возвращает:
    - Оклад до удержания
    - Удержания (ИПН, ОПВ, ВОСМС)
    - Взносы работодателя (СО, ООСМС, ОПВР)
    """
    employee = EmployeeData(
        salary_net=salary_net,
        has_children=has_children,
        has_disability=has_disability
    )
    
    result = tax_service_kz.calculate_employee_contributions(employee, mzp)
    return {
        "salary_net": salary_net,
        "salary_gross": result.salary_gross,
        "deductions": result.deductions,
        "employer_contributions": result.employer,
        "total_deductions": result.total_deductions,
        "total_employer": result.total_employer,
        "kbk": tax_service_kz.KBK
    }


@router.post("/calculate-ie-own")
async def calculate_ie_own_tax(
    mzp: Optional[float] = None,
    use_mzp: bool = True
):
    """
    Расчёт взносов ИП за себя
    
    - **mzp**: МЗП для расчёта (по умолчанию 85 000)
    - **use_mzp**: Использовать МЗП как базу
    
    Возвращает:
    - ОПВ 10%
    - СО 5%
    - ВОСМС 5% от 1.4 МЗП
    - ОПВР 3.5%
    """
    result = tax_service_kz.calculate_ie_contributions(mzp, use_mzp)
    return {
        "contributions": {
            "opv": {"rate": "10%", "amount": result.opv},
            "so": {"rate": "5%", "amount": result.so},
            "vosms": {"rate": "5% от 1.4 МЗП", "amount": result.vosms},
            "opvr": {"rate": "3.5%", "amount": result.opvr}
        },
        "total": result.total,
        "kbk": {
            "opv": tax_service_kz.KBK["opv"],
            "so": tax_service_kz.KBK["so"],
            "vosms": tax_service_kz.KBK["vosms"],
            "opvr": tax_service_kz.KBK["opvr"]
        },
        "deadline": "до 25 числа следующего месяца"
    }


@router.get("/deadlines")
async def get_tax_deadlines():
    """
    Сроки уплаты налогов и сдачи отчётности
    """
    from datetime import datetime
    
    current_month = datetime.now().month
    next_month = current_month + 1 if current_month < 12 else 1
    
    months = [
        "", "января", "февраля", "марта", "апреля", "мая", "июня",
        "июля", "августа", "сентября", "октября", "ноября", "декабря"
    ]
    
    return {
        "monthly": {
            "opv_so_oms": f"до 25 {months[next_month]}",
            "ipn_salary": f"до 25 {months[next_month]}",
            "description": "Ежемесячные платежи за сотрудников и ИП за себя"
        },
        "uzd": {
            "first_half": {
                "declaration": "до 15 августа",
                "payment": "до 25 августа"
            },
            "second_half": {
                "declaration": "до 15 февраля",
                "payment": "до 25 февраля"
            }
        },
        "forms": {
            "uzd": "910.00",
            "opv": "Перечисление на счёт",
            "so": "Перечисление на счёт",
            "oms": "Перечисление на счёт"
        }
    }
