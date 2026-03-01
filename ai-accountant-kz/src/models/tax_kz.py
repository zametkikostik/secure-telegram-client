"""Модели для расчёта налогов ИП Казахстан 2026"""
from pydantic import BaseModel, Field
from typing import Optional, List
from datetime import date


class EmployeeData(BaseModel):
    """Данные о сотруднике"""
    salary_net: float = Field(..., description="Оклад на руки (тенге)")
    has_disability: bool = Field(False, description="Есть ли инвалидность")
    has_children: bool = Field(False, description="Есть ли дети (для СИЗ)")


class TaxCalculationRequest(BaseModel):
    """Запрос на расчёт налогов"""
    income: float = Field(..., description="Доход ИП за период (тенге)")
    period_months: int = Field(1, description="Количество месяцев (1-6)")
    employees: List[EmployeeData] = Field(default_factory=list, description="Сотрудники")
    ie_own_contributions: bool = Field(True, description="Платить взносы ИП за себя")
    mzp: float = Field(85000, description="МЗП (тенге)")
    mrp: float = Field(4325, description="МРП (тенге)")


class IESocialContributions(BaseModel):
    """Взносы ИП за себя"""
    opv: float = Field(..., description="ОПВ 10%")
    so: float = Field(..., description="СО 5%")
    vosms: float = Field(..., description="ВОСМС 5% от 1.4 МЗП")
    opvr: float = Field(..., description="ОПВР 3.5%")
    total: float = Field(..., description="Итого за ИП")


class EmployeeContributions(BaseModel):
    """Взносы за сотрудника"""
    salary_gross: float = Field(..., description="Оклад до удержания")
    deductions: dict = Field(..., description="Удержания из зарплаты")
    employer: dict = Field(..., description="Взносы работодателя")
    total_deductions: float = Field(..., description="Всего удержано")
    total_employer: float = Field(..., description="Всего работодатель")


class UZDTax(BaseModel):
    """Налог по упрощенке"""
    income_total: float = Field(..., description="Общий доход")
    rate_percent: float = Field(4.0, description="Ставка %")
    tax_total: float = Field(..., description="Налог 4%")
    ipn: float = Field(..., description="ИПН (2%)")
    social_tax: float = Field(..., description="Соц.налог (2%)")
    social_tax_after_so: float = Field(..., description="Соц.налог после уменьшения на СО")
    total_to_pay: float = Field(..., description="Всего к уплате")


class TaxCalculationResponse(BaseModel):
    """Результат расчёта налогов"""
    period: str = Field(..., description="Период расчёта")
    constants: dict = Field(..., description="Константы (МЗП, МРП)")
    
    ie_own: Optional[IESocialContributions] = Field(None, description="Взносы ИП за себя")
    
    employees: List[EmployeeContributions] = Field(default_factory=list, description="Сотрудники")
    
    uzd: Optional[UZDTax] = Field(None, description="Налог по упрощенке")
    
    totals: dict = Field(..., description="Итоговые суммы")
    
    deadlines: dict = Field(..., description="Сроки уплаты")
    
    kbk: dict = Field(..., description="КБК для оплаты")
    
    warnings: List[str] = Field(default_factory=list, description="Предупреждения")


class TaxRateInfo(BaseModel):
    """Информация о налоговых ставках"""
    mzp: float
    mrp: float
    ie_contributions: dict
    employee_contributions: dict
    uzd_rate: float
    vat_threshold: float
    uzd_income_limit: float
