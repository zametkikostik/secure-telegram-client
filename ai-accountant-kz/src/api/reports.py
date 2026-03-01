"""API endpoints для отчётов"""
from fastapi import APIRouter, Depends, Response, Query
from sqlalchemy.orm import Session
from sqlalchemy import func, extract
from datetime import datetime
from decimal import Decimal
import csv
import io

from ..core.database import get_db, TransactionDB, TransactionTypeDB, User
from ..core.security import get_current_user
from ..core.tax_rules import TAX_RULES

router = APIRouter(prefix="/api/v1", tags=["reports"])

@router.get("/summary")
async def get_summary(
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Общая сводка пользователя"""
    
    # Доходы
    income_query = db.query(func.sum(TransactionDB.amount)).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == TransactionTypeDB.INCOME
    )
    total_income = income_query.scalar() or Decimal("0")
    
    # Расходы
    expense_query = db.query(func.sum(TransactionDB.amount)).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == TransactionTypeDB.EXPENSE
    )
    total_expense = expense_query.scalar() or Decimal("0")
    
    tax = total_income * TAX_RULES.RATE
    
    return {
        "total_income": float(total_income),
        "total_expense": float(total_expense),
        "net_profit": float(total_income - total_expense),
        "tax_amount": float(tax),
        "tax_rate": TAX_RULES.rate_percent,
        "transactions_count": db.query(TransactionDB).filter(
            TransactionDB.user_id == current_user.id
        ).count()
    }

@router.get("/tax/calculate")
async def calculate_tax(
    period: str = Query(default="2026", description="Год или период (2026 или 2026-1)"),
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Расчёт налога за период"""
    
    year = int(period.split("-")[0])
    
    # Доходы за год
    income_query = db.query(func.sum(TransactionDB.amount)).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == TransactionTypeDB.INCOME,
        extract('year', TransactionDB.date) == year
    )
    total_income = income_query.scalar() or Decimal("0")
    
    tax_amount = total_income * TAX_RULES.RATE
    is_limit_exceeded = total_income > TAX_RULES.max_income
    
    return {
        "period": period,
        "year": year,
        "total_income": float(total_income),
        "tax_rate": str(TAX_RULES.RATE),
        "tax_percent": TAX_RULES.rate_percent,
        "tax_amount": float(tax_amount),
        "is_limit_exceeded": is_limit_exceeded,
        "limit_amount": TAX_RULES.max_income,
        "remaining_limit": float(max(0, TAX_RULES.max_income - total_income))
    }

@router.get("/export/csv")
async def export_csv(
    tx_type: str = None,
    start_date: str = None,
    end_date: str = None,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Экспорт транзакций в CSV"""
    
    query = db.query(TransactionDB).filter(
        TransactionDB.user_id == current_user.id
    ).order_by(TransactionDB.date.desc())
    
    if tx_type:
        query = query.filter(TransactionDB.type == tx_type)
    
    if start_date:
        query = query.filter(TransactionDB.date >= datetime.fromisoformat(start_date))
    
    if end_date:
        query = query.filter(TransactionDB.date <= datetime.fromisoformat(end_date))
    
    transactions = query.all()
    
    output = io.StringIO()
    writer = csv.writer(output)
    
    # Заголовки
    writer.writerow([
        "Дата", "Тип", "Сумма", "Валюта", "Контрагент", 
        "Описание", "Категория AI", "Статус", "Источник"
    ])
    
    # Данные
    for tx in transactions:
        writer.writerow([
            tx.date.strftime("%d.%m.%Y"),
            "Доход" if tx.type == TransactionTypeDB.INCOME else "Расход",
            float(tx.amount),
            tx.currency,
            tx.counterparty or "",
            tx.description,
            tx.ai_category or "",
            tx.ai_status.value,
            tx.source.value
        ])
    
    filename = f"transactions_{current_user.inn or current_user.email}_{datetime.now().strftime('%Y%m%d')}.csv"
    
    return Response(
        content=output.getvalue(),
        media_type="text/csv",
        headers={"Content-Disposition": f"attachment; filename={filename}"}
    )

@router.get("/stats/monthly")
async def get_monthly_stats(
    year: int = Query(default=None),
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user)
):
    """Месячная статистика"""
    
    if year is None:
        year = datetime.now().year
    
    # Группировка по месяцам
    from sqlalchemy import extract
    
    monthly_data = db.query(
        extract('month', TransactionDB.date).label('month'),
        func.sum(TransactionDB.amount).filter(TransactionDB.type == TransactionTypeDB.INCOME).label('income'),
        func.sum(TransactionDB.amount).filter(TransactionDB.type == TransactionTypeDB.EXPENSE).label('expense')
    ).filter(
        TransactionDB.user_id == current_user.id,
        extract('year', TransactionDB.date) == year
    ).group_by(
        extract('month', TransactionDB.date)
    ).all()
    
    result = []
    for row in monthly_data:
        result.append({
            "month": int(row.month),
            "income": float(row.income) if row.income else 0,
            "expense": float(row.expense) if row.expense else 0
        })
    
    return {"year": year, "data": result}
