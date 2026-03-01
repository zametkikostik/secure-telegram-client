"""Сервис расчёта налогов для ИП Казахстан 2026"""
from datetime import datetime, date
from typing import List, Optional
from loguru import logger

from ..models.tax_kz import (
    TaxCalculationRequest,
    TaxCalculationResponse,
    IESocialContributions,
    EmployeeContributions,
    UZDTax,
    EmployeeData
)


class KZTaxService2026:
    """
    Сервис расчёта налогов ИП на упрощенке (Казахстан, 2026)
    
    Константы:
    - МЗП: 85 000 ₸
    - МРП: 4 325 ₸
    - Упрощенка: 4% (ИПН 2% + СН 2%)
    - ОПВР: 2.5% (с 2026 года)
    """
    
    # Константы 2026
    MZP = 85000
    MRP = 4325
    DEDUCTION_14MRP = 14 * 4325  # 60 550 ₸
    
    # Лимиты
    VAT_THRESHOLD = 10000 * MRP  # 43 250 000 ₸
    UZD_INCOME_LIMIT = 20000 * MZP  # 1 700 000 000 ₸
    
    # Ставки ИП за себя
    IE_OPV_RATE = 0.10
    IE_SO_RATE = 0.05
    IE_VOSMS_RATE = 0.05  # от 1.4 МЗП
    IE_OPVR_RATE = 0.035
    
    # Ставки сотрудник
    EMP_OPV_RATE = 0.10
    EMP_VOSMS_RATE = 0.02
    EMP_OOSMS_RATE = 0.03
    EMP_SO_RATE = 0.05
    EMP_OPVR_RATE = 0.025  # 2.5% на 2026 год!
    EMP_IPN_RATE = 0.10
    
    # КБК 2026 (по требованиям пользователя)
    KBK = {
        "opv": "183102",
        "so": "183101",
        "vosms": "122101",
        "oosms": "122101",
        "opvr": "183110",
        "ipn": "101201",
        "uzd_ipn": "101202",
        "uzd_social": "103101"
    }
    
    def calculate_ie_contributions(
        self,
        mzp: float = None,
        use_mzp: bool = True
    ) -> IESocialContributions:
        """Расчёт взносов ИП за себя"""
        base = mzp if mzp else self.MZP
        
        # От 1 МЗП
        opv = base * self.IE_OPV_RATE
        so = base * self.IE_SO_RATE
        
        # ВОСМС от 1.4 МЗП
        vosms_base = base * 1.4
        vosms = vosms_base * self.IE_VOSMS_RATE
        
        # ОПВР от 1 МЗП
        opvr = base * self.IE_OPVR_RATE
        
        return IESocialContributions(
            opv=opv,
            so=so,
            vosms=vosms,
            opvr=opvr,
            total=opv + so + vosms + opvr
        )
    
    def calculate_employee_contributions(
        self,
        employee: EmployeeData,
        mzp: float = None
    ) -> EmployeeContributions:
        """Расчёт взносов за сотрудника"""
        base_mzp = mzp if mzp else self.MZP
        
        # Расчёт зарплаты до удержания
        # На руки = salary_net
        # ИПН 10% после вычета СИЗ
        # СИЗ = 14 МРП если есть дети
        
        siz = 14 * self.MRP if employee.has_children else 0
        
        # salary_net = gross - IPN(10%) - OPV(10%) - VOSMS(2%)
        # salary_net = gross - 0.22*gross + adjustments
        # gross = salary_net / 0.78 (примерно)
        
        # Точный расчёт:
        # taxable_income = gross - OPV(10%) - SIZ
        # IPN = taxable_income * 10%
        # net = gross - OPV - VOSMS - IPN
        
        # Итеративный расчёт
        gross = employee.salary_net / 0.78
        
        # Удержания
        opv = gross * self.EMP_OPV_RATE
        vosms = gross * self.EMP_VOSMS_RATE
        
        taxable_income = gross - opv - siz
        ipn = max(0, taxable_income * self.EMP_IPN_RATE)
        
        # Корректировка gross для точного net
        actual_net = gross - opv - vosms - ipn
        if abs(actual_net - employee.salary_net) > 1:
            # Точная подгонка
            gross = employee.salary_net + opv + vosms + ipn
        
        # Пересчитываем с точным gross
        opv = round(gross * self.EMP_OPV_RATE)
        vosms = round(gross * self.EMP_VOSMS_RATE)
        taxable_income = gross - opv - vosms - self.DEDUCTION_14MRP  # 14 МРП вычет
        ipn = round(max(0, taxable_income * self.EMP_IPN_RATE))

        # Взносы работодателя
        so = round((gross - opv) * self.EMP_SO_RATE)
        oosms = round(gross * self.EMP_OOSMS_RATE)
        opvr = round(gross * self.EMP_OPVR_RATE)  # 2.5%

        return EmployeeContributions(
            salary_gross=round(gross),
            deductions={
                "ipn": {"rate": "10%", "amount": ipn, "kbk": self.KBK["ipn"]},
                "opv": {"rate": "10%", "amount": opv, "kbk": self.KBK["opv"]},
                "vosms": {"rate": "2%", "amount": vosms, "kbk": self.KBK["vosms"]}
            },
            employer={
                "so": {"rate": "5%", "amount": so, "kbk": self.KBK["so"]},
                "oosms": {"rate": "3%", "amount": oosms, "kbk": self.KBK["oosms"]},
                "opvr": {"rate": "2.5%", "amount": opvr, "kbk": self.KBK["opvr"]}
            },
            total_deductions=ipn + opv + vosms,
            total_employer=so + oosms + opvr
        )
    
    def calculate_uzd_tax(
        self,
        income: float,
        period_months: int,
        total_so: float
    ) -> UZDTax:
        """Расчёт налога по упрощенке"""
        # Ставка 4%
        tax_total = income * 0.04
        
        # ИПН 50% + СН 50%
        ipn = tax_total * 0.5
        social_tax = tax_total * 0.5
        
        # Уменьшение СН на СО
        social_tax_after_so = max(0, social_tax - total_so)
        
        # Всего к уплате
        total_to_pay = ipn + social_tax_after_so
        
        return UZDTax(
            income_total=income,
            rate_percent=4.0,
            tax_total=round(tax_total, 2),
            ipn=round(ipn, 2),
            social_tax=round(social_tax, 2),
            social_tax_after_so=round(social_tax_after_so, 2),
            total_to_pay=round(total_to_pay, 2)
        )
    
    def calculate(
        self,
        request: TaxCalculationRequest
    ) -> TaxCalculationResponse:
        """Полный расчёт налогов"""
        warnings = []
        
        # Проверка лимитов
        if request.income > self.VAT_THRESHOLD:
            warnings.append(
                f"⚠️ Превышен порог НДС: {request.income:,.0f} ₸ > {self.VAT_THRESHOLD:,.0f} ₸"
            )
        
        if request.income > self.UZD_INCOME_LIMIT:
            warnings.append(
                f"⚠️ Превышен лимит упрощенки: {request.income:,.0f} ₸ > {self.UZD_INCOME_LIMIT:,.0f} ₸"
            )
        
        # Расчёт ИП за себя
        ie_own = None
        if request.ie_own_contributions:
            ie_own = self.calculate_ie_contributions(request.mzp)
        
        # Расчёт сотрудников
        employees: List[EmployeeContributions] = []
        total_so = ie_own.so if ie_own else 0
        total_employee_deductions = 0
        total_employee_employer = 0
        
        for emp in request.employees:
            emp_calc = self.calculate_employee_contributions(emp, request.mzp)
            employees.append(emp_calc)
            total_so += emp_calc.employer["so"]["amount"]
            total_employee_deductions += emp_calc.total_deductions
            total_employee_employer += emp_calc.total_employer
        
        # Расчёт упрощенки
        uzd = None
        if request.income > 0:
            uzd = self.calculate_uzd_tax(
                request.income,
                request.period_months,
                total_so
            )
        
        # Итоговые суммы
        totals = {
            "ie_own_monthly": ie_own.total if ie_own else 0,
            "employees_total_deductions": total_employee_deductions,
            "employees_total_employer": total_employee_employer,
            "uzd_tax": uzd.total_to_pay if uzd else 0,
            "grand_total_monthly": (
                (ie_own.total if ie_own else 0) +
                total_employee_deductions +
                total_employee_employer
            ),
            "uzd_annual_estimate": uzd.tax_total if uzd else 0
        }
        
        # Сроки
        current_month = datetime.now().month
        next_month = current_month + 1 if current_month < 12 else 1
        
        deadlines = {
            "monthly": {
                "opv_so_oms": f"до 25 {self._get_month_name(next_month)}",
                "ipn_salary": f"до 25 {self._get_month_name(next_month)}"
            },
            "uzd": {
                "payment_1half": "до 25 августа",
                "declaration_1half": "до 15 августа",
                "payment_2half": "до 25 февраля",
                "declaration_2half": "до 15 февраля"
            }
        }
        
        # КБК
        kbk = {
            "ie_own": {
                "opv": self.KBK["opv"],
                "so": self.KBK["so"],
                "vosms": self.KBK["vosms"],
                "opvr": self.KBK["opvr"]
            },
            "employee": {
                "opv": self.KBK["opv"],
                "vosms": self.KBK["vosms"],
                "oosms": self.KBK["oosms"],
                "so": self.KBK["so"],
                "opvr": self.KBK["opvr"],
                "ipn": self.KBK["ipn"]
            },
            "uzd": {
                "ipn": self.KBK["uzd_ipn"],
                "social_tax": self.KBK["uzd_social"]
            }
        }
        
        return TaxCalculationResponse(
            period=f"{self._get_month_name(current_month)} {datetime.now().year}",
            constants={
                "mzp": self.MZP,
                "mrp": self.MRP,
                "vat_threshold": self.VAT_THRESHOLD,
                "uzd_income_limit": self.UZD_INCOME_LIMIT
            },
            ie_own=ie_own,
            employees=employees,
            uzd=uzd,
            totals=totals,
            deadlines=deadlines,
            kbk=kbk,
            warnings=warnings
        )
    
    def _get_month_name(self, month: int) -> str:
        """Название месяца"""
        months = [
            "", "января", "февраля", "марта", "апреля", "мая", "июня",
            "июля", "августа", "сентября", "октября", "ноября", "декабря"
        ]
        return months[month]
    
    def get_rates_info(self) -> dict:
        """Информация о текущих ставках"""
        return {
            "year": 2026,
            "mzp": self.MZP,
            "mrp": self.MRP,
            "deduction_14mrp": self.DEDUCTION_14MRP,
            "ie_contributions": {
                "opv": {"rate": "10%", "base": "1 МЗП", "amount": self.MZP * 0.10, "kbk": self.KBK["opv"]},
                "so": {"rate": "5%", "base": "1 МЗП", "amount": self.MZP * 0.05, "kbk": self.KBK["so"]},
                "vosms": {"rate": "5%", "base": "1.4 МЗП", "amount": self.MZP * 1.4 * 0.05, "kbk": self.KBK["vosms"]},
                "opvr": {"rate": "3.5%", "base": "1 МЗП", "amount": self.MZP * 0.035, "kbk": self.KBK["opvr"]}
            },
            "employee_contributions": {
                "deductions": {
                    "ipn": "10% (после вычета 14 МРП)",
                    "opv": "10%",
                    "vosms": "2%"
                },
                "employer": {
                    "so": "5%",
                    "oosms": "3%",
                    "opvr": "2.5% (на 2026 год)"
                },
                "kbk": {
                    "ipn": self.KBK["ipn"],
                    "opv": self.KBK["opv"],
                    "vosms": self.KBK["vosms"],
                    "so": self.KBK["so"],
                    "oosms": self.KBK["oosms"],
                    "opvr": self.KBK["opvr"]
                }
            },
            "uzd": {
                "rate": "4%",
                "ipn": "2%",
                "social_tax": "2%",
                "deadlines": {
                    "payment": "25 августа / 25 февраля",
                    "declaration": "15 августа / 15 февраля"
                },
                "kbk": {
                    "ipn": self.KBK["uzd_ipn"],
                    "social": self.KBK["uzd_social"]
                }
            },
            "limits": {
                "vat_threshold": f"{self.VAT_THRESHOLD:,.0f} ₸ (10 000 МРП)",
                "uzd_income": f"{self.UZD_INCOME_LIMIT:,.0f} ₸ (20 000 МЗП)"
            }
        }


# Глобальный экземпляр
tax_service_kz = KZTaxService2026()
