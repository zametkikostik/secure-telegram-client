"""База данных и модели SQLAlchemy для продакшена"""
from sqlalchemy import create_engine, Column, String, Integer, DateTime, Numeric, Enum, Text, Boolean, ForeignKey, Index
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker, relationship
from sqlalchemy.pool import StaticPool, NullPool
from datetime import datetime
from decimal import Decimal
import enum
import os

from .config import settings, DATA_DIR

# Выбор движка в зависимости от DATABASE_URL
if settings.DATABASE_URL:
    # PostgreSQL для продакшена
    engine = create_engine(
        settings.DATABASE_URL,
        pool_size=20,
        max_overflow=40,
        pool_recycle=3600,
        pool_pre_ping=True,
        echo=settings.DEBUG
    )
else:
    # SQLite для разработки
    engine = create_engine(
        settings.DATABASE_URL_SQLITE,
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
        echo=settings.DEBUG
    )

SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
Base = declarative_base()

class TransactionTypeDB(str, enum.Enum):
    INCOME = "income"
    EXPENSE = "expense"
    TRANSFER = "transfer"

class TransactionSourceDB(str, enum.Enum):
    KASPI = "kaspi"
    HALYK = "halyk"
    MANUAL = "manual"
    CSV = "csv"
    API = "api"

class AIClassificationStatusDB(str, enum.Enum):
    PENDING = "pending"
    CONFIRMED = "confirmed"
    REJECTED = "rejected"
    NEEDS_REVIEW = "needs_review"
    PROCESSING = "processing"

class User(Base):
    """Пользователи системы"""
    __tablename__ = "users"
    
    id = Column(String, primary_key=True, index=True)
    email = Column(String, unique=True, index=True, nullable=False)
    hashed_password = Column(String, nullable=False)
    full_name = Column(String, nullable=True)
    inn = Column(String, unique=True, index=True, nullable=True)  # ИИН/БИН
    phone = Column(String, nullable=True)
    
    # Настройки
    is_active = Column(Boolean, default=True)
    is_superuser = Column(Boolean, default=False)
    telegram_chat_id = Column(String, nullable=True)
    language = Column(String, default="ru")
    
    # 2FA
    two_factor_secret = Column(String, nullable=True)
    two_factor_enabled = Column(Boolean, default=False)
    backup_codes = Column(String, nullable=True)  # JSON список резервных кодов
    
    # Временные метки
    created_at = Column(DateTime, default=datetime.now)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)
    last_login = Column(DateTime, nullable=True)
    
    # Связи
    transactions = relationship("TransactionDB", back_populates="user", cascade="all, delete-orphan")
    employees = relationship("Employee", back_populates="user", cascade="all, delete-orphan")

class TransactionDB(Base):
    """Транзакции"""
    __tablename__ = "transactions"
    
    id = Column(String, primary_key=True, index=True)
    user_id = Column(String, ForeignKey("users.id"), nullable=False, index=True)
    
    date = Column(DateTime, nullable=False, index=True)
    amount = Column(Numeric(15, 2), nullable=False)
    currency = Column(String, default="KZT")
    type = Column(Enum(TransactionTypeDB), nullable=False)
    
    # Контрагент
    counterparty = Column(String, nullable=True, index=True)
    counterparty_inn = Column(String, nullable=True)
    
    # Описание
    description = Column(Text, nullable=False)
    source = Column(Enum(TransactionSourceDB), default=TransactionSourceDB.MANUAL)
    
    # Банк
    bank_transaction_id = Column(String, nullable=True, unique=True)  # ID из банка
    bank_name = Column(String, nullable=True)  # kaspi, halyk
    
    # AI классификация
    ai_category = Column(String, nullable=True)
    ai_confidence = Column(Numeric(5, 4), nullable=True)
    ai_status = Column(Enum(AIClassificationStatusDB), default=AIClassificationStatusDB.PENDING)
    ai_reasoning = Column(Text, nullable=True)
    ai_model = Column(String, nullable=True)  # Какая модель использовалась
    
    # Ручная категория
    manual_category = Column(String, nullable=True)
    category_confirmed = Column(Boolean, default=False)  # Пользователь подтвердил категорию
    
    # Файлы
    attachment_path = Column(String, nullable=True)  # Путь к чеку/документу
    
    # Временные метки
    created_at = Column(DateTime, default=datetime.now, index=True)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)
    
    # Связи
    user = relationship("User", back_populates="transactions")
    
    __table_args__ = (
        Index('ix_transactions_user_date', 'user_id', 'date'),
        Index('ix_transactions_type', 'type'),
        Index('ix_transactions_amount', 'amount'),
    )

class AuditLog(Base):
    """Лог аудита всех действий"""
    __tablename__ = "audit_logs"
    
    id = Column(String, primary_key=True, index=True)
    user_id = Column(String, ForeignKey("users.id"), index=True)
    
    action = Column(String, nullable=False)  # create, update, delete, login, etc.
    resource = Column(String, nullable=False)  # transaction, user, etc.
    resource_id = Column(String, nullable=True)
    
    old_value = Column(Text, nullable=True)  # JSON
    new_value = Column(Text, nullable=True)  # JSON
    
    ip_address = Column(String, nullable=True)
    user_agent = Column(String, nullable=True)
    
    created_at = Column(DateTime, default=datetime.now, index=True)

class TaxDeclaration(Base):
    """Налоговые декларации"""
    __tablename__ = "tax_declarations"
    
    id = Column(String, primary_key=True, index=True)
    user_id = Column(String, ForeignKey("users.id"), nullable=False)
    
    period = Column(String, nullable=False)  # 2026-1 (год-квартал)
    year = Column(Integer, nullable=False)
    quarter = Column(Integer, nullable=False)
    
    total_income = Column(Numeric(15, 2), nullable=False)
    taxable_income = Column(Numeric(15, 2), nullable=False)
    tax_rate = Column(Numeric(5, 4), nullable=False)
    tax_amount = Column(Numeric(15, 2), nullable=False)
    
    status = Column(String, default="draft")  # draft, submitted, paid
    submitted_at = Column(DateTime, nullable=True)
    paid_at = Column(DateTime, nullable=True)
    
    declaration_file = Column(String, nullable=True)  # Путь к файлу декларации

    created_at = Column(DateTime, default=datetime.now)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)

class Employee(Base):
    """Сотрудники"""
    __tablename__ = "employees"
    
    id = Column(String, primary_key=True, index=True)
    user_id = Column(String, ForeignKey("users.id"), nullable=False)
    
    full_name = Column(String, nullable=False)
    inn = Column(String, unique=True, index=True, nullable=False)
    position = Column(String, nullable=True)
    phone = Column(String, nullable=True)
    email = Column(String, nullable=True)
    
    salary = Column(Numeric(15, 2), nullable=False)
    is_active = Column(Boolean, default=True)
    
    hired_at = Column(DateTime, default=datetime.now)
    fired_at = Column(DateTime, nullable=True)
    
    created_at = Column(DateTime, default=datetime.now)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)
    
    # Связи
    user = relationship("User", back_populates="employees")
    payroll_records = relationship("PayrollRecord", back_populates="employee")

class PayrollRecord(Base):
    """Записи о зарплате"""
    __tablename__ = "payroll_records"
    
    id = Column(String, primary_key=True, index=True)
    user_id = Column(String, ForeignKey("users.id"), nullable=False)
    employee_id = Column(String, ForeignKey("employees.id"), nullable=False)
    
    period = Column(String, nullable=False)
    accrual_date = Column(DateTime, default=datetime.now)
    
    salary = Column(Numeric(15, 2), nullable=False)
    bonus = Column(Numeric(15, 2), default=0)
    total_accrued = Column(Numeric(15, 2), nullable=False)
    
    opv = Column(Numeric(15, 2), default=0)
    opv_ipn = Column(Numeric(15, 2), default=0)
    social_tax = Column(Numeric(15, 2), default=0)
    osms = Column(Numeric(15, 2), default=0)
    total_deductions = Column(Numeric(15, 2), default=0)
    
    net_salary = Column(Numeric(15, 2), nullable=False)
    
    status = Column(String, default="draft")
    paid_at = Column(DateTime, nullable=True)
    
    created_at = Column(DateTime, default=datetime.now)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)
    
    # Связи
    employee = relationship("Employee", back_populates="payroll_records")

def get_db():
    """Зависимость для получения сессии БД"""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

def init_db():
    """Инициализация БД"""
    Base.metadata.create_all(bind=engine)

def drop_db():
    """Удаление всех таблиц (для тестов)"""
    Base.metadata.drop_all(bind=engine)
