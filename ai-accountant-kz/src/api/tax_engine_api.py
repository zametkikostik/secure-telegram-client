"""API endpoints для налогового движка РК 2026"""
from fastapi import APIRouter, Depends, HTTPException, Query
from fastapi.responses import FileResponse
from sqlalchemy.orm import Session
from typing import Optional
import json
from pathlib import Path

from ..core.database import get_db
from ..core.tax_core import tax_engine, KazakhstanTaxEngine2026, AuditLogger

router = APIRouter(prefix="/api/v1/tax-engine", tags=["tax-engine-2026"])


@router.get("/sync")
async def sync_tax_rates(
    config_url: Optional[str] = Query(None, description="URL удалённого конфига"),
    force_local: bool = Query(False, description="Только локальный файл")
):
    """
    Синхронизация налоговых ставок
    
    - **config_url**: URL JSON файла с константами
    - **force_local**: Использовать только локальный файл
    
    Возвращает статус обновления
    """
    result = tax_engine.sync_rates(config_url, force_local)
    return {
        "status": "success",
        "message": result,
        "current_rates": tax_engine.get_rates_info()
    }


@router.get("/rates")
async def get_current_rates():
    """
    Получение текущих налоговых ставок и КБК
    """
    return tax_engine.get_rates_info()


@router.post("/calculate/employee")
async def calculate_employee_tax(
    salary_gross: float = Query(..., description="Оклад до вычетов (грязными)"),
    user_id: str = Query("Anonymous", description="ID пользователя для аудита"),
    db: Session = Depends(get_db)
):
    """
    Расчёт налогов для сотрудника
    
    - **salary_gross**: Оклад до удержания (грязными)
    - **user_id**: ID пользователя для журнала аудита
    
    Возвращает:
    - Удержания сотрудника (ОПВ, ВОСМС, ИПН)
    - Взносы работодателя (СО, ООСМС, ОПВР)
    - Зарплата на руки
    - verification_hash для аудита
    """
    result = tax_engine.calculate_employee(salary_gross, user_id)
    return result


@router.get("/constants")
async def get_constants():
    """
    Получение констант (МЗП, МРП, вычеты)
    """
    return {
        "MRP": tax_engine.MRP,
        "MZP": tax_engine.MZP,
        "DEDUCTION_14MRP": tax_engine.DEDUCTION_14MRP,
        "DEDUCTION_14MRP_MULTIPLIER": tax_engine.DEDUCTION_14MRP // tax_engine.MRP,
        "OPVR_RATE": tax_engine.OPVR_RATE,
        "config_version": tax_engine.CONFIG_VERSION
    }


@router.get("/kbk")
async def get_kbks():
    """
    Справочник КБК
    """
    return tax_engine.KBK_MAP


@router.get("/audit-log")
async def get_audit_log(
    limit: int = Query(100, description="Количество записей"),
    user_id: Optional[str] = Query(None, description="Фильтр по пользователю")
):
    """
    Получение журнала аудита
    
    - **limit**: Количество записей (макс. 1000)
    - **user_id**: Фильтр по ID пользователя
    """
    audit_file = Path(AuditLogger.AUDIT_FILE)
    
    if not audit_file.exists():
        return {"status": "no_logs", "message": "Журнал аудита пуст"}
    
    logs = []
    with open(audit_file, 'r', encoding='utf-8') as f:
        for line in f:
            try:
                log_entry = json.loads(line.strip())
                if user_id is None or log_entry.get('user_id') == user_id:
                    logs.append(log_entry)
            except:
                continue
    
    # Сортировка по времени (новые сверху)
    logs.sort(key=lambda x: x.get('timestamp', ''), reverse=True)
    
    return {
        "status": "success",
        "total_entries": len(logs),
        "entries": logs[:limit]
    }


@router.get("/audit-log/download")
async def download_audit_log():
    """
    Скачать полный журнал аудита (JSON)
    """
    audit_file = Path(AuditLogger.AUDIT_FILE)
    
    if not audit_file.exists():
        raise HTTPException(status_code=404, detail="Журнал аудита не найден")
    
    return FileResponse(
        audit_file,
        media_type="application/json",
        filename="audit_trail.json"
    )


@router.post("/calculate/net-to-gross")
async def calculate_net_to_gross(
    salary_net: float = Query(..., description="Оклад на руки (чистыми)"),
    db: Session = Depends(get_db)
):
    """
    Расчёт gross из net (обратный расчёт)
    
    Формула: gross ≈ net / 0.78 (итеративно)
    """
    # Итеративный расчёт
    gross = salary_net / 0.78
    
    for _ in range(5):  # 5 итераций для точности
        opv = gross * tax_engine.RATES['opv_employee']
        vosms = gross * tax_engine.RATES['vosms_employee']
        taxable_base = gross - opv - vosms - tax_engine.DEDUCTION_14MRP
        ipn = max(0, taxable_base * tax_engine.RATES['ipn_employee'])
        
        calculated_net = gross - opv - vosms - ipn
        if abs(calculated_net - salary_net) < 1:
            break
        
        gross = gross + (salary_net - calculated_net) * 0.5
    
    # Финальный расчёт
    result = tax_engine.calculate_employee(round(gross))
    result["requested_net"] = salary_net
    result["calculated_gross"] = round(gross)
    
    return result
