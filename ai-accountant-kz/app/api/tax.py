"""API endpoints для налогового калькулятора Казахстана 2026"""
from fastapi import APIRouter, Depends, HTTPException, Header
from typing import Optional
import logging
from datetime import datetime

from ..core.tax_core import tax_engine, TaxEngineKZ2026
from ..core.config import settings
from ..core.logging_config import get_trace_id, set_trace_id
from ..schemas.tax import (
    TaxCalculationResponse,
    HealthResponse,
    RatesInfoResponse,
    ErrorResponse
)

logger = logging.getLogger("ai-accountant")

router = APIRouter(prefix="/v1", tags=["tax-calculator"])


def verify_api_key(x_api_key: Optional[str] = Header(None)) -> str:
    """
    Проверка API Key для аутентификации запросов
    
    Args:
        x_api_key: API Key из заголовка запроса
    
    Returns:
        str: API Key если валиден
    
    Raises:
        HTTPException: Если API Key отсутствует или неверен
    """
    if not x_api_key:
        raise HTTPException(
            status_code=401,
            detail={
                "error": "unauthorized",
                "message": "API Key отсутствует. Добавьте заголовок X-API-Key",
                "trace_id": get_trace_id()
            }
        )
    
    # Проверка API Key (сравнение с настройками)
    if x_api_key != settings.API_KEY:
        logger.warning(f"Неверный API Key попытка: {x_api_key[:8]}...")
        raise HTTPException(
            status_code=403,
            detail={
                "error": "forbidden",
                "message": "Неверный API Key",
                "trace_id": get_trace_id()
            }
        )
    
    return x_api_key


@router.get(
    "/health",
    response_model=HealthResponse,
    summary="Health Check",
    description="Проверка статуса системы"
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
        timestamp=datetime.utcnow().isoformat() + "Z"
    )


@router.post(
    "/calculate/gross",
    response_model=TaxCalculationResponse,
    summary="Расчет от оклада",
    description="Расчет налогов от суммы до вычетов (gross)",
    responses={
        200: {"description": "Успешный расчет"},
        401: {"model": ErrorResponse, "description": "Отсутствует API Key"},
        403: {"model": ErrorResponse, "description": "Неверный API Key"},
        422: {"model": ErrorResponse, "description": "Ошибка валидации данных"}
    }
)
async def calculate_from_gross(
    amount: float,
    has_children: bool = False,
    has_disability: bool = False,
    x_api_key: str = Depends(verify_api_key)
):
    """
    Расчет налогов от оклада (суммы до вычетов)
    
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
        amount: Оклад до вычетов (грязными) в тенге
        has_children: Есть ли дети (влияет на вычеты)
        has_disability: Есть ли инвалидность
        x_api_key: API Key для аутентификации
    
    Returns:
        TaxCalculationResponse: Результаты расчета с КБК
    """
    trace_id = get_trace_id()
    logger.info(f"Расчет gross: amount={amount}, trace_id={trace_id}")
    
    try:
        result = tax_engine.calculate_from_gross(
            salary_gross=amount,
            user_id="api_user",
            trace_id=trace_id
        )
        
        return TaxCalculationResponse(**result)
    
    except Exception as e:
        logger.error(f"Ошибка расчета gross: {e}", exc_info=True)
        raise HTTPException(
            status_code=500,
            detail={
                "error": "calculation_error",
                "message": str(e),
                "trace_id": trace_id
            }
        )


@router.post(
    "/calculate/net",
    response_model=TaxCalculationResponse,
    summary="Расчет от суммы на руки",
    description="Обратный расчет: из суммы на руки (net) в оклад (gross)",
    responses={
        200: {"description": "Успешный расчет"},
        401: {"model": ErrorResponse, "description": "Отсутствует API Key"},
        403: {"model": ErrorResponse, "description": "Неверный API Key"},
        422: {"model": ErrorResponse, "description": "Ошибка валидации данных"}
    }
)
async def calculate_from_net(
    amount: float,
    has_children: bool = False,
    has_disability: bool = False,
    x_api_key: str = Depends(verify_api_key)
):
    """
    Обратный расчет: из желаемой суммы на руки в оклад до вычетов
    
    Использует итеративный метод для точного определения gross.
    
    **Алгоритм:**
    1. Начальное приближение: gross ≈ net / 0.78
    2. Итеративная коррекция (до 5 итераций)
    3. Финальный расчет через calculate_from_gross
    
    Args:
        amount: Желаемая сумма на руки (чистыми) в тенге
        has_children: Есть ли дети (влияет на вычеты)
        has_disability: Есть ли инвалидность
        x_api_key: API Key для аутентификации
    
    Returns:
        TaxCalculationResponse: Результаты расчета с КБК
    """
    trace_id = get_trace_id()
    logger.info(f"Расчет net: amount={amount}, trace_id={trace_id}")
    
    try:
        result = tax_engine.calculate_from_net(
            salary_net=amount,
            user_id="api_user",
            trace_id=trace_id
        )
        
        return TaxCalculationResponse(**result)
    
    except Exception as e:
        logger.error(f"Ошибка расчета net: {e}", exc_info=True)
        raise HTTPException(
            status_code=500,
            detail={
                "error": "calculation_error",
                "message": str(e),
                "trace_id": trace_id
            }
        )


@router.get(
    "/rates",
    response_model=RatesInfoResponse,
    summary="Налоговые ставки",
    description="Получение информации о текущих налоговых ставках и КБК"
)
async def get_rates_info():
    """
    Получение актуальных налоговых ставок и констант
    
    Returns:
        RatesInfoResponse: Ставки, константы, КБК
    """
    rates = tax_engine.get_rates_info()
    return RatesInfoResponse(**rates)
