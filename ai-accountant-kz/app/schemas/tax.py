"""Pydantic схемы для налогового API"""
from pydantic import BaseModel, Field
from typing import Dict, Optional
from datetime import datetime


class TaxCalculationRequest(BaseModel):
    """Запрос на расчет налога"""
    amount: float = Field(..., description="Сумма (оклад или на руки)", gt=0)
    calculation_type: str = Field(..., description="Тип расчета", pattern="^(gross|net)$")
    has_children: bool = Field(False, description="Есть ли дети для вычета")
    has_disability: bool = Field(False, description="Есть ли инвалидность")


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


class TaxCalculationResponse(BaseModel):
    """Ответ с расчетом налогов"""
    status: str = Field(..., description="Статус расчета")
    config_version: str = Field(..., description="Версия конфигурации")
    calculation_type: str = Field(..., description="Тип расчета (gross/net)")
    
    # Константы использованные в расчете
    mrp_used: int = Field(..., description="МРП использованный в расчете")
    mzp_used: int = Field(..., description="МЗП использованный в расчете")
    deduction_14mrp: int = Field(..., description="Вычет 14 МРП")
    opvr_rate: float = Field(..., description="Ставка ОПВР")
    
    # Входные данные
    salary_gross: float = Field(..., description="Оклад до вычетов")
    salary_net: float = Field(..., description="Оклад на руки")
    
    # Удержания сотрудника
    employee_deductions: Dict[str, EmployeeDeduction] = Field(
        ..., 
        description="Удержания сотрудника (ОПВ, ВОСМС, ИПН)"
    )
    
    # Взносы работодателя
    employer_contributions: Dict[str, EmployerContribution] = Field(
        ..., 
        description="Взносы работодателя (СО, ООСМС, ОПВР)"
    )
    
    # Итоги
    total_employee_deductions: float = Field(..., description="Всего удержано")
    total_employer_cost: float = Field(..., description="Всего расходов работодателя")
    total_budget: float = Field(..., description="Всего в бюджет")
    
    # Аудит
    trace_id: str = Field(..., description="ID треисинга запроса")
    timestamp: str = Field(..., description="Время расчета")


class HealthResponse(BaseModel):
    """Ответ health check"""
    status: str = Field(..., description="Статус сервиса")
    version: str = Field(..., description="Версия API")
    environment: str = Field(..., description="Окружение")
    timestamp: str = Field(..., description="Время проверки")


class RatesInfoResponse(BaseModel):
    """Информация о налоговых ставках"""
    version: str = Field(..., description="Версия конфигурации")
    constants: Dict[str, int] = Field(..., description="Константы (МРП, МЗП, вычеты)")
    rates: Dict[str, Dict[str, str]] = Field(..., description="Ставки налогов")
    kbk: Dict[str, str] = Field(..., description="КБК")
    last_updated: str = Field(..., description="Дата обновления")


class ErrorResponse(BaseModel):
    """Стандартный ответ об ошибке"""
    error: str = Field(..., description="Код ошибки")
    message: str = Field(..., description="Описание ошибки")
    trace_id: str = Field(..., description="ID треисинга")
    timestamp: str = Field(..., description="Время ошибки")
