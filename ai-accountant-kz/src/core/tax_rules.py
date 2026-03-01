"""Налоговые правила Казахстана для ИП на упрощенке (2026)"""
from dataclasses import dataclass
from decimal import Decimal

@dataclass
class TaxRules2026:
    # Ставка упрощенки
    RATE: Decimal = Decimal("0.04")  # 4%
    
    # Константы 2026
    MZP: int = 85000  # Минимальная зарплата
    MRP: int = 4325   # Месячный расчётный показатель
    
    # Лимиты
    LIMIT_MRP: int = 24038  # 24 038 МРП
    VAT_THRESHOLD_MRP: int = 10000  # 10 000 МРП для НДС
    
    # Сроки
    DECLARATION_DEADLINES: list = None
    PAYMENT_DEADLINES: list = None
    
    # Взносы ИП за себя (в месяц)
    IE_OPV: int = 8500    # 10% от МЗП
    IE_SO: int = 4250     # 5% от МЗП
    IE_VOSMS: int = 5950  # 5% от 1.4 МЗП
    IE_OPVR: int = 2975   # 3.5% от МЗП
    IE_TOTAL: int = 21675 # Итого

    def __post_init__(self):
        self.DECLARATION_DEADLINES = ["15.02", "15.08"]  # Упрощенка
        self.PAYMENT_DEADLINES = ["25.02", "25.08"]      # Упрощенка

    @property
    def max_income(self) -> int:
        return self.LIMIT_MRP * self.MRP
    
    @property
    def vat_threshold(self) -> int:
        return self.VAT_THRESHOLD_MRP * self.MRP  # 43 250 000 ₸

    @property
    def rate_percent(self) -> str:
        return f"{int(self.RATE * 100)}%"
    
    @property
    def ie_contributions(self) -> dict:
        return {
            "opv": self.IE_OPV,
            "so": self.IE_SO,
            "vosms": self.IE_VOSMS,
            "opvr": self.IE_OPVR,
            "total": self.IE_TOTAL
        }

INCOME_CATEGORIES = {
    "sales": "Реализация товаров/услуг",
    "services": "Оплата услуг",
    "advance": "Авансы полученные",
    "other_income": "Прочие доходы",
}

EXPENSE_CATEGORIES = {
    "suppliers": "Оплата поставщикам",
    "salary": "Зарплата",
    "rent": "Аренда",
    "utilities": "Коммунальные услуги",
    "taxes": "Налоги и сборы",
    "bank_fees": "Банковские комиссии",
    "other_expense": "Прочие расходы",
}

EXEMPT_SOURCES = {
    "loan_received": "Получение кредита",
    "loan_repaid": "Возврат кредита",
    "owner_contribution": "Взнос собственника",
    "transfer_own": "Перевод между своими счетами",
    "refund": "Возврат от поставщика",
}

TAX_RULES = TaxRules2026()
