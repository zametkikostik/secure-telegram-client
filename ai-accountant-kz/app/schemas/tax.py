"""Pydantic схемы для налогового API"""
from pydantic import BaseModel, Field
from typing import Dict, Optional, Literal
from datetime import datetime


class CalculationRequest(BaseModel):
    """
    Запрос на расчет налога (единая схема)
    
    Args:
        amount: Сумма в тенге (оклад или желаемая на руки)
        calculation_type: Тип расчета - 'gross' (от оклада) или 'net' (от суммы на руки)
        has_children: Есть ли дети для вычета
        has_disability: Есть ли инвалидность
    """
    amount: float = Field(..., description="Сумма в тенге", gt=0, le=100000000)
    calculation_type: Literal["gross", "net"] = Field(..., description="Тип расчета: gross (от оклада) или net (от суммы на руки)")
    has_children: bool = Field(False, description="Есть ли дети для вычета")
    has_disability: bool = Field(False, description="Есть ли инвалидность")
    
    class Config:
        json_schema_extra = {
            "example": {
                "amount": 300000,
                "calculation_type": "gross",
                "has_children": False,
                "has_disability": False
            }
        }


class EmployeeDeduction(BaseModel):
    """Удержание сотрудника"""
    amount: float = Field(..., description="Сумма удержания")
    kbk: str = Field(..., description="КБК платежа")
    rate: str = Field(..., description="Ставка в процентах")


class EmployerContribution(BaseModel):
    """Взнос работодателя"""
    amount: float = Field(..., description="Сумма взноса")
    kbk: str = Field(..., description="КБК платежа")
    rate: str = Field(..., description="Ставка в процентах")


class CalculationResponse(BaseModel):
    """
    Ответ с расчетом налогов (единая схема)
    
    Structured Output с строгой валидацией типов
    """
    success: bool = Field(True, description="Статус выполнения")
    trace_id: str = Field(..., description="ID трассировки запроса")
    timestamp: str = Field(..., description="Время расчета в ISO 8601")
    
    # Параметры расчета
    calculation_type: str = Field(..., description="Тип расчета (gross/net)")
    input_amount: float = Field(..., description="Входная сумма")
    
    # Константы 2026
    constants: Dict[str, int] = Field(..., description="Константы (МРП, МЗП, вычеты)")
    
    # Результаты
    salary_gross: float = Field(..., description="Оклад до вычетов")
    salary_net: float = Field(..., description="Оклад на руки")
    
    # Удержания и взносы
    employee_deductions: Dict[str, EmployeeDeduction] = Field(
        ..., 
        description="Удержания сотрудника (ОПВ, ВОСМС, ИПН)"
    )
    employer_contributions: Dict[str, EmployerContribution] = Field(
        ..., 
        description="Взносы работодателя (СО, ООСМС, ОПВР)"
    )
    
    # Итоги
    total_employee_deductions: float = Field(..., description="Всего удержано с сотрудника")
    total_employer_cost: float = Field(..., description="Всего расходов работодателя")
    total_budget: float = Field(..., description="Всего в бюджет")
    
    class Config:
        json_schema_extra = {
            "example": {
                "success": True,
                "trace_id": "550e8400-e29b-41d4-a716-446655440000",
                "timestamp": "2026-03-01T12:00:00Z",
                "calculation_type": "gross",
                "input_amount": 300000,
                "salary_gross": 300000,
                "salary_net": 243655,
                "total_employee_deductions": 56345,
                "total_employer_cost": 22500,
                "total_budget": 78845
            }
        }


class HealthResponse(BaseModel):
    """Ответ health check"""
    status: str = Field(..., description="Статус сервиса")
    version: str = Field(..., description="Версия API")
    environment: str = Field(..., description="Окружение")
    timestamp: str = Field(..., description="Время проверки")


class ConfigResponse(BaseModel):
    """Ответ с текущими ставками МРП/МЗП из ядра"""
    version: str = Field(..., description="Версия конфигурации")
    mrp: int = Field(..., description="МРП (Месячный расчётный показатель)")
    mzp: int = Field(..., description="МЗП (Минимальная зарплата)")
    opvr_rate: float = Field(..., description="Ставка ОПВР")
    deduction_14mrp: int = Field(..., description="Вычет 14 МРП")
    rates: Dict[str, Dict[str, str]] = Field(..., description="Ставки налогов")
    kbk: Dict[str, str] = Field(..., description="КБК")


class ErrorResponse(BaseModel):
    """Стандартный ответ об ошибке"""
    success: bool = Field(False, description="Статус ошибки")
    error: str = Field(..., description="Код ошибки")
    message: str = Field(..., description="Описание ошибки")
    trace_id: str = Field(..., description="ID трассировки")
    timestamp: str = Field(..., description="Время ошибки")
