"""Расчёт зарплаты и налогов для сотрудников РК"""
from decimal import Decimal
from dataclasses import dataclass
from typing import Optional

@dataclass
class PayrollResult:
    """Результат расчёта зарплаты"""
    salary: Decimal  # Оклад
    bonus: Decimal  # Премия
    total_accrued: Decimal  # Всего начислено
    
    # Удержания
    opv: Decimal  # ОПВ 10%
    opv_ipn: Decimal  # ИПН с ОПВ
    social_tax: Decimal  # Соцналог
    osms: Decimal  # ОСМС 3.5%
    total_deductions: Decimal  # Всего удержано
    
    net_salary: Decimal  # На руки


def calculate_payroll(
    salary: float,
    bonus: float = 0,
    has_dependents: bool = False
) -> PayrollResult:
    """
    Расчёт зарплаты с налогами Казахстана (2026)
    
    Args:
        salary: Оклад (месячный)
        bonus: Премия
        has_dependents: Есть ли иждивенцы (влияет на ИПН)
    
    Returns:
        PayrollResult с расчётами
    """
    salary = Decimal(str(salary))
    bonus = Decimal(str(bonus))
    
    # Всего начислено
    total_accrued = salary + bonus
    
    # ===== Удержания =====
    
    # ОПВ (Обязательные пенсионные взносы) - 10%
    opv = total_accrued * Decimal("0.10")
    
    # ОСМС (Обязательное социальное медицинское страхование) - 3.5%
    osms = total_accrued * Decimal("0.035")
    
    # Налоговая база для ИПН
    taxable_income = total_accrued - opv
    
    # ИПН (Индивидуальный подоходный налог) - 10%
    # С учётом вычета (14 МРП в месяц = 54,936 ₸ в 2026)
    mrp = Decimal("3924")
    tax_deduction = mrp * Decimal("14")  # 54,936 ₸
    
    if has_dependents:
        # С иждивенцами вычет больше
        tax_deduction = tax_deduction * Decimal("1.5")
    
    ipn_base = max(0, taxable_income - tax_deduction)
    ipn = ipn_base * Decimal("0.10")
    
    # ИПН с ОПВ (возврат из ОПВ)
    opv_ipn = opv * Decimal("0.10")  # 10% от ОПВ
    
    # Социальный налог - 9.5% (рассчитывается от дохода до вычетов)
    # Уменьшается на соцвзнос (3.5%)
    social_tax_base = total_accrued
    social_tax = social_tax_base * Decimal("0.095")
    
    # Итого удержаний
    total_deductions = opv + osms + ipn
    
    # На руки
    net_salary = total_accrued - total_deductions
    
    return PayrollResult(
        salary=salary,
        bonus=bonus,
        total_accrued=total_accrued,
        opv=opv,
        opv_ipn=opv_ipn,
        social_tax=social_tax,
        osms=osms,
        total_deductions=total_deductions,
        net_salary=net_salary
    )


def calculate_employer_taxes(salary: float) -> dict:
    """
    Расчёт налогов работодателя сверх зарплаты
    
    Args:
        salary: Зарплата до вычетов
    
    Returns:
        Dict с налогами работодателя
    """
    salary = Decimal(str(salary))
    
    # Социальные отчисления (СО) - 3.5%
    social_contributions = salary * Decimal("0.035")
    
    # ОСМС работодатель - 3%
    osms_employer = salary * Decimal("0.03")
    
    # Социальный налог - 9.5%
    social_tax = salary * Decimal("0.095")
    
    # Итого налогов работодателя
    total_employer_taxes = social_contributions + osms_employer + social_tax
    
    # Итого стоимость сотрудника для компании
    total_cost = salary + total_employer_taxes
    
    return {
        "salary": float(salary),
        "social_contributions": float(social_contributions),
        "osms_employer": float(osms_employer),
        "social_tax": float(social_tax),
        "total_employer_taxes": float(total_employer_taxes),
        "total_cost": float(total_cost)
    }


def format_payroll_slip(result: PayrollResult, employee_name: str, period: str) -> str:
    """Форматирование расчётного листка"""
    return f"""
═══════════════════════════════════════════════════
         РАСЧЁТНЫЙ ЛИСТОК
═══════════════════════════════════════════════════
Сотрудник: {employee_name}
Период: {period}
───────────────────────────────────────────────────
НАЧИСЛЕНИЯ:
  Оклад:                    {result.salary:>15,.2f} ₸
  Премия:                   {result.bonus:>15,.2f} ₸
  ─────────────────────────────────────────────
  Всего начислено:          {result.total_accrued:>15,.2f} ₸

УДЕРЖАНИЯ:
  ОПВ (10%):                {result.opv:>15,.2f} ₸
  ИПН (10%):                {result.opv_ipn:>15,.2f} ₸
  ОСМС (3.5%):              {result.osms:>15,.2f} ₸
  ─────────────────────────────────────────────
  Всего удержано:           {result.total_deductions:>15,.2f} ₸

═══════════════════════════════════════════════════
К ВЫПЛАТЕ:                  {result.net_salary:>15,.2f} ₸
═══════════════════════════════════════════════════

НАЛОГИ РАБОТОДАТЕЛЯ (сверх зарплаты):
  Соцналог (9.5%):          {result.social_tax:>15,.2f} ₸
  СО (3.5%):                {result.osms:>15,.2f} ₸
  ─────────────────────────────────────────────
  Итого налогов:            {result.social_tax + result.osms:>15,.2f} ₸
"""
