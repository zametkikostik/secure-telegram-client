"""Модели данных для транзакций"""
from pydantic import BaseModel, Field
from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Optional

class TransactionType(str, Enum):
    INCOME = "income"
    EXPENSE = "expense"
    TRANSFER = "transfer"

class TransactionSource(str, Enum):
    KASPI = "kaspi"
    HALYK = "halyk"
    MANUAL = "manual"
    CSV = "csv"

class AIClassificationStatus(str, Enum):
    PENDING = "pending"
    CONFIRMED = "confirmed"
    REJECTED = "rejected"
    NEEDS_REVIEW = "needs_review"

class Transaction(BaseModel):
    id: str
    date: datetime
    amount: Decimal
    currency: str = "KZT"
    type: TransactionType
    counterparty: Optional[str] = None
    description: str
    source: TransactionSource
    ai_category: Optional[str] = None
    ai_confidence: Optional[float] = None
    ai_status: AIClassificationStatus = AIClassificationStatus.PENDING
    manual_category: Optional[str] = None
    created_at: datetime = Field(default_factory=datetime.now)
    updated_at: datetime = Field(default_factory=datetime.now)

class TransactionCreate(BaseModel):
    date: datetime
    amount: Decimal
    type: TransactionType
    counterparty: Optional[str] = None
    description: str
    source: TransactionSource = TransactionSource.MANUAL

class TaxCalculation(BaseModel):
    period: str
    total_income: Decimal
    taxable_income: Decimal
    tax_rate: Decimal
    tax_amount: Decimal
    is_limit_exceeded: bool = False

class AIClassificationRequest(BaseModel):
    description: str
    amount: Decimal
    counterparty: Optional[str] = None
    transaction_type: TransactionType

class AIClassificationResponse(BaseModel):
    category: str
    confidence: float
    reasoning: str
    needs_review: bool = False
    warning: Optional[str] = None
