"""Модели для платёжных квитанций"""
from pydantic import BaseModel, Field
from typing import List, Optional
from datetime import date


class PaymentItem(BaseModel):
    """Один платёж в квитанции"""
    kbk: str = Field(..., description="КБК")
    name: str = Field(..., description="Название платежа")
    amount: float = Field(..., description="Сумма (тенге)")
    period: str = Field(..., description="Период (месяц/год)")


class PaymentSlipRequest(BaseModel):
    """Запрос на формирование платёжек"""
    taxpayer_name: str = Field(..., description="ФИО налогоплательщика (ИП)")
    taxpayer_inn: str = Field(..., description="ИИН/БИН")
    taxpayer_address: str = Field(..., description="Адрес")
    bank_name: str = Field(..., description="Название банка")
    bank_bik: str = Field(..., description="БИК банка")
    account_number: str = Field(..., description="Расчётный счёт")
    
    ie_payments: bool = Field(True, description="Платежи ИП за себя")
    employee_payments: bool = Field(False, description="Платежи за сотрудников")
    
    income: float = Field(0, description="Доход (для расчёта)")
    employee_count: int = Field(0, description="Количество сотрудников")
    employee_salary_gross: float = Field(0, description="Оклад сотрудника (грязными)")
    
    period: str = Field(..., description="Период (месяц/год)")


class PaymentSlip(BaseModel):
    """Платёжная квитанция"""
    slip_type: str = Field(..., description="Тип квитанции")
    payer_name: str = Field(..., description="Плательщик")
    payer_inn: str = Field(..., description="ИИН/БИН")
    payer_address: str = Field(..., description="Адрес")
    
    recipient_name: str = Field(..., description="Получатель")
    recipient_bik: str = Field(..., description="БИК получателя")
    recipient_kbe: str = Field("16", description="КБЕ получателя")
    
    account_number: str = Field(..., description="Счёт плательщика")
    bank_name: str = Field(..., description="Банк плательщика")
    bank_bik: str = Field(..., description="БИК банка")
    
    payments: List[PaymentItem] = Field(default_factory=list, description="Платежи")
    total_amount: float = Field(..., description="Общая сумма")
    
    date: str = Field(..., description="Дата формирования")
    deadline: str = Field(..., description="Срок уплаты")


class PaymentSlipsResponse(BaseModel):
    """Ответ с платёжками"""
    slips: List[PaymentSlip] = Field(default_factory=list, description="Список квитанций")
    total_amount: float = Field(..., description="Общая сумма всех платежей")
    period: str = Field(..., description="Период")
    generated_at: str = Field(..., description="Дата формирования")
