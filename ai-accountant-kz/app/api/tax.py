"""
API endpoints для налогового калькулятора Казахстана 2026

Endpoints:
- POST /api/v1/calculate — основной расчет
- GET /api/v1/health — проверка состояния сервиса
- GET /api/v1/config — текущие ставки МРП/МЗП из ядра
"""
from fastapi import APIRouter, Depends, HTTPException, Header, status
from typing import Optional
import logging
from datetime import datetime, timezone

from ..core.tax_core import tax_engine, TaxEngineKZ2026
from ..core.config import settings
from ..core.logging_config import get_trace_id, set_trace_id
from ..schemas.tax import (
    CalculationRequest,
    CalculationResponse,
    HealthResponse,
    ConfigResponse,
    ErrorResponse,
    EmployeeDeduction,
    EmployerContribution
)

logger = logging.getLogger("ai-accountant")

router = APIRouter(prefix="/api/v1", tags=["tax-calculator"])


def verify_api_key(x_api_key: Optional[str] = Header(None, alias="X-API-Key")) -> str:
    """
    API Key Authentication
    
    Проверяет наличие и валидность API Key в заголовке X-API-Key
    
    Returns:
        str: Валидный API Key
    
    Raises:
        HTTPException: 401 Unauthorized если ключ отсутствует или неверен
    """
    if not x_api_key:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=ErrorResponse(
                success=False,
                error="unauthorized",
                message="API Key отсутствует. Добавьте заголовок X-API-Key",
                trace_id=get_trace_id(),
                timestamp=datetime.now(timezone.utc).isoformat()
            ).model_dump()
        )
    
    # Проверка API Key
    if x_api_key != settings.API_KEY:
        logger.warning(f"Неверный API Key попытка: {x_api_key[:8]}...")
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=ErrorResponse(
                success=False,
                error="unauthorized",
                message="Неверный API Key",
                trace_id=get_trace_id(),
                timestamp=datetime.now(timezone.utc).isoformat()
            ).model_dump()
        )
    
    return x_api_key


@router.get(
    "/health",
    response_model=HealthResponse,
    summary="Health Check",
    description="Проверка состояния сервиса",
    tags=["health"]
)
async def health_check():
    """
    Health check endpoint для мониторинга доступности сервиса
    
    Returns:
        HealthResponse: Статус сервиса, версия, окружение
    """
    return HealthResponse(
        status="healthy",
        version="2026.03.01",
        environment=settings.ENVIRONMENT,
        timestamp=datetime.now(timezone.utc).isoformat()
    )


@router.get(
    "/config",
    response_model=ConfigResponse,
    summary="Конфигурация ядра",
    description="Получение текущих ставок МРП/МЗП из ядра TaxEngine",
    tags=["config"]
)
async def get_config(x_api_key: str = Depends(verify_api_key)):
    """
    Получение текущих ставок МРП/МЗП из ядра
    
    Returns:
        ConfigResponse: Константы 2026 (МРП, МЗП, ОПВР, КБК)
    """
    rates_info = tax_engine.get_rates_info()
    
    return ConfigResponse(
        version=rates_info["version"],
        mrp=TaxEngineKZ2026.MRP,
        mzp=TaxEngineKZ2026.MZP,
        opvr_rate=TaxEngineKZ2026.OPVR_RATE,
        deduction_14mrp=TaxEngineKZ2026.DEDUCTION_14MRP,
        rates=rates_info["rates"],
        kbk=rates_info["kbk"]
    )


@router.post(
    "/calculate",
    response_model=CalculationResponse,
    summary="Основной расчет",
    description="Расчет налогов для ИП Казахстан 2026",
    responses={
        200: {"model": CalculationResponse, "description": "Успешный расчет"},
        401: {"model": ErrorResponse, "description": "Unauthorized"},
        422: {"model": ErrorResponse, "description": "Ошибка валидации данных"}
    }
)
async def calculate(
    request: CalculationRequest,
    x_api_key: str = Depends(verify_api_key)
):
    """
    Основной endpoint для расчета налогов
    
    **Константы 2026:**
    - МРП: 4325 ₸
    - МЗП: 85000 ₸
    - ОПВР: 2.5%
    - КБК ОПВР: 183110
    
    **Удержания сотрудника:**
    - ОПВ: 10%
    - ВОСМС: 2%
    - ИПН: 10% (с учетом вычета 14 МРП = 60 550 ₸)
    
    **Взносы работодателя:**
    - СО: 5%
    - ООСМС: 3%
    - ОПВР: 2.5%
    
    Args:
        request: CalculationRequest с параметрами расчета
        x_api_key: API Key для аутентификации
    
    Returns:
        CalculationResponse: Результаты расчета с КБК
    """
    trace_id = get_trace_id()
    timestamp = datetime.now(timezone.utc).isoformat()
    
    logger.info(
        f"Расчет: type={request.calculation_type}, amount={request.amount}",
        extra={"extra_data": {"trace_id": trace_id}}
    )
    
    try:
        # Выполняем расчет через TaxEngine
        if request.calculation_type == "gross":
            result = tax_engine.calculate_from_gross(
                salary_gross=request.amount,
                user_id="api_user",
                trace_id=trace_id
            )
        else:
            result = tax_engine.calculate_from_net(
                salary_net=request.amount,
                user_id="api_user",
                trace_id=trace_id
            )
        
        # Формируем ответ в соответствии со схемой
        return CalculationResponse(
            success=True,
            trace_id=trace_id,
            timestamp=timestamp,
            calculation_type=result["calculation_type"],
            input_amount=result.get("requested_net", result.get("salary_gross", request.amount)),
            constants={
                "MRP": result["mrp_used"],
                "MZP": result["mzp_used"],
                "DEDUCTION_14MRP": result["deduction_14mrp"],
                "OPVR_RATE": int(result["opvr_rate"] * 100)
            },
            salary_gross=result["salary_gross"],
            salary_net=result["salary_net"],
            employee_deductions={
                k: EmployeeDeduction(**v) 
                for k, v in result["employee_deductions"].items()
            },
            employer_contributions={
                k: EmployerContribution(**v) 
                for k, v in result["employer_contributions"].items()
            },
            total_employee_deductions=result["total_employee_deductions"],
            total_employer_cost=result["total_employer_cost"],
            total_budget=result["total_budget"]
        )
    
    except Exception as e:
        logger.error(f"Ошибка расчета: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=ErrorResponse(
                success=False,
                error="calculation_error",
                message=str(e),
                trace_id=trace_id,
                timestamp=timestamp
            ).model_dump()
        )
