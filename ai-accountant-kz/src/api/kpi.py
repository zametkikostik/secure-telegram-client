"""API endpoints для бизнес-аналитики и KPI"""
from fastapi import APIRouter, Depends, Query
from sqlalchemy.orm import Session
from sqlalchemy import func, extract, desc
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Optional

from ..core.database import get_db, TransactionDB, TransactionTypeDB, Employee, PayrollRecord, User
from ..core.security import get_current_user
from ..core.tax_rules import TAX_RULES

router = APIRouter(prefix="/api/v1/kpi", tags=["kpi"])

@router.get("/summary")
async def get_kpi_summary(
    period: str = "month",
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """
    Сводка KPI
    
    period: day, week, month, quarter, year
    """
    now = datetime.now()
    
    # Определяем период
    if period == "day":
        start_date = now - timedelta(days=1)
    elif period == "week":
        start_date = now - timedelta(weeks=1)
    elif period == "month":
        start_date = now - timedelta(days=30)
    elif period == "quarter":
        start_date = now - timedelta(days=90)
    else:  # year
        start_date = now - timedelta(days=365)
    
    # Доходы за период
    income_query = db.query(func.sum(TransactionDB.amount)).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == TransactionTypeDB.INCOME,
        TransactionDB.date >= start_date
    )
    total_income = income_query.scalar() or Decimal("0")
    
    # Расходы за период
    expense_query = db.query(func.sum(TransactionDB.amount)).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == TransactionTypeDB.EXPENSE,
        TransactionDB.date >= start_date
    )
    total_expense = expense_query.scalar() or Decimal("0")
    
    # Прибыль
    gross_profit = total_income - total_expense
    profit_margin = (gross_profit / total_income * 100) if total_income > 0 else Decimal("0")
    
    # Налог
    tax = total_income * TAX_RULES.RATE
    
    # Чистая прибыль
    net_profit = gross_profit - tax
    
    # Количество транзакций
    tx_count = db.query(TransactionDB).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.date >= start_date
    ).count()
    
    # Средний чек
    avg_check = (total_income / tx_count) if tx_count > 0 else Decimal("0")
    
    # Сотрудники
    employees_count = db.query(Employee).filter(
        Employee.user_id == current_user.id,
        Employee.is_active == True
    ).count()
    
    # Зарплатный фонд за период
    payroll_query = db.query(func.sum(PayrollRecord.net_salary)).filter(
        PayrollRecord.user_id == current_user.id,
        PayrollRecord.accrual_date >= start_date
    )
    payroll_fund = payroll_query.scalar() or Decimal("0")
    
    return {
        "period": period,
        "start_date": start_date.isoformat(),
        "end_date": now.isoformat(),
        "revenue": float(total_income),
        "expenses": float(total_expense),
        "gross_profit": float(gross_profit),
        "profit_margin": float(profit_margin),
        "tax": float(tax),
        "net_profit": float(net_profit),
        "transactions_count": tx_count,
        "avg_check": float(avg_check),
        "employees_count": employees_count,
        "payroll_fund": float(payroll_fund)
    }

@router.get("/revenue-trend")
async def get_revenue_trend(
    days: int = 30,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Тренд доходов по дням"""
    end_date = datetime.now()
    start_date = end_date - timedelta(days=days)
    
    # Группировка по дням
    from sqlalchemy import cast, Date
    
    trend_query = db.query(
        cast(TransactionDB.date, Date).label('date'),
        func.sum(TransactionDB.amount).filter(TransactionDB.type == TransactionTypeDB.INCOME).label('income'),
        func.sum(TransactionDB.amount).filter(TransactionDB.type == TransactionTypeDB.EXPENSE).label('expense')
    ).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.date >= start_date,
        TransactionDB.date <= end_date
    ).group_by(
        cast(TransactionDB.date, Date)
    ).order_by(
        cast(TransactionDB.date, Date)
    ).all()
    
    return [{
        "date": str(row.date),
        "income": float(row.income) if row.income else 0,
        "expense": float(row.expense) if row.expense else 0,
        "profit": float((row.income or 0) - (row.expense or 0))
    } for row in trend_query]

@router.get("/top-customers")
async def get_top_customers(
    limit: int = 10,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Топ клиентов по доходам"""
    query = db.query(
        TransactionDB.counterparty,
        func.sum(TransactionDB.amount).label('total'),
        func.count(TransactionDB.id).label('count')
    ).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == TransactionTypeDB.INCOME,
        TransactionDB.counterparty.isnot(None)
    ).group_by(
        TransactionDB.counterparty
    ).order_by(
        desc('total')
    ).limit(limit).all()
    
    return [{
        "name": row.counterparty,
        "total": float(row.total),
        "count": row.count
    } for row in query]

@router.get("/top-expenses")
async def get_top_expenses(
    limit: int = 10,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Топ расходов по категориям"""
    query = db.query(
        TransactionDB.ai_category,
        func.sum(TransactionDB.amount).label('total'),
        func.count(TransactionDB.id).label('count')
    ).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == TransactionTypeDB.EXPENSE,
        TransactionDB.ai_category.isnot(None)
    ).group_by(
        TransactionDB.ai_category
    ).order_by(
        desc('total')
    ).limit(limit).all()
    
    return [{
        "category": row.ai_category,
        "total": float(row.total),
        "count": row.count
    } for row in query]

@router.get("/limits")
async def get_limits_status(
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Статус лимитов (налоговый лимит и т.д.)"""
    now = datetime.now()
    year_start = datetime(now.year, 1, 1)
    
    # Доход за год
    income_query = db.query(func.sum(TransactionDB.amount)).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.type == TransactionTypeDB.INCOME,
        TransactionDB.date >= year_start
    )
    total_income = income_query.scalar() or Decimal("0")
    
    # Лимит для упрощенки
    limit = TAX_RULES.max_income
    used_percent = (total_income / limit * 100) if limit > 0 else Decimal("0")
    
    # Остаток
    remaining = max(0, limit - total_income)
    
    # Статус
    if used_percent >= 90:
        status = "critical"
    elif used_percent >= 75:
        status = "warning"
    else:
        status = "normal"
    
    return {
        "year": now.year,
        "total_income": float(total_income),
        "limit": limit,
        "used_percent": float(used_percent),
        "remaining": float(remaining),
        "status": status,
        "tax_rate": TAX_RULES.rate_percent
    }

@router.get("/forecast")
async def get_forecast(
    months: int = 3,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Прогноз доходов на основе истории"""
    # Простая линейная регрессия для прогноза
    now = datetime.now()
    
    # Получаем данные за последние 6 месяцев
    start_date = now - timedelta(days=180)
    
    monthly_query = db.query(
        extract('year', TransactionDB.date).label('year'),
        extract('month', TransactionDB.date).label('month'),
        func.sum(TransactionDB.amount).filter(TransactionDB.type == TransactionTypeDB.INCOME).label('income')
    ).filter(
        TransactionDB.user_id == current_user.id,
        TransactionDB.date >= start_date,
        TransactionDB.type == TransactionTypeDB.INCOME
    ).group_by(
        extract('year', TransactionDB.date),
        extract('month', TransactionDB.date)
    ).order_by(
        'year', 'month'
    ).all()
    
    if len(monthly_query) < 2:
        return {"forecast": [], "message": "Недостаточно данных для прогноза"}
    
    # Простой прогноз (средний рост)
    incomes = [float(row.income) if row.income else 0 for row in monthly_query]
    
    # Рассчитываем средний месячный рост
    if len(incomes) > 1:
        growth_rates = [(incomes[i] - incomes[i-1]) / incomes[i-1] if incomes[i-1] > 0 else 0 
                       for i in range(1, len(incomes))]
        avg_growth = sum(growth_rates) / len(growth_rates) if growth_rates else 0
    else:
        avg_growth = 0
    
    # Прогноз
    last_income = incomes[-1] if incomes else 0
    forecast = []
    
    for i in range(months):
        forecast_date = datetime(now.year, now.month + i + 1, 1)
        forecasted_income = last_income * (1 + avg_growth)
        
        forecast.append({
            "month": forecast_date.strftime("%B %Y"),
            "forecasted_income": round(forecasted_income, 2),
            "confidence": max(0.5, 0.9 - (i * 0.1))  # Уверенность падает с каждым месяцем
        })
        last_income = forecasted_income
    
    return {
        "forecast": forecast,
        "avg_monthly_growth": round(avg_growth * 100, 2),
        "base_income": last_income
    }
