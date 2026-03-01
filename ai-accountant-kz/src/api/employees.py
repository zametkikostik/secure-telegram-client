"""API endpoints для управления сотрудниками и зарплатой"""
from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy.orm import Session
from datetime import datetime
from decimal import Decimal
from typing import List, Optional
import uuid

from ..core.database import get_db, Employee, PayrollRecord, User
from ..core.security import get_current_user
from ..utils.payroll import calculate_payroll, calculate_employer_taxes, format_payroll_slip

router = APIRouter(prefix="/api/v1/employees", tags=["employees"])

@router.get("/")
async def get_employees(
    is_active: bool = True,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Список сотрудников"""
    query = db.query(Employee).filter(
        Employee.user_id == current_user.id,
        Employee.is_active == is_active
    )
    
    employees = query.all()
    
    return [{
        "id": emp.id,
        "full_name": emp.full_name,
        "inn": emp.inn,
        "position": emp.position,
        "phone": emp.phone,
        "email": emp.email,
        "salary": float(emp.salary),
        "hired_at": emp.hired_at.isoformat() if emp.hired_at else None,
        "is_active": emp.is_active
    } for emp in employees]

@router.post("/")
async def create_employee(
    full_name: str,
    inn: str,
    salary: float,
    position: Optional[str] = None,
    phone: Optional[str] = None,
    email: Optional[str] = None,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Добавление сотрудника"""
    # Проверка ИНН
    existing = db.query(Employee).filter(
        Employee.inn == inn,
        Employee.user_id == current_user.id
    ).first()
    
    if existing:
        raise HTTPException(status_code=400, detail="Сотрудник с таким ИНН уже существует")
    
    employee = Employee(
        id=str(uuid.uuid4()),
        user_id=current_user.id,
        full_name=full_name,
        inn=inn,
        position=position,
        phone=phone,
        email=email,
        salary=Decimal(str(salary))
    )
    
    db.add(employee)
    db.commit()
    db.refresh(employee)
    
    return {"id": employee.id, "status": "created"}

@router.delete("/{employee_id}")
async def delete_employee(
    employee_id: str,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Удаление сотрудника"""
    employee = db.query(Employee).filter(
        Employee.id == employee_id,
        Employee.user_id == current_user.id
    ).first()
    
    if not employee:
        raise HTTPException(status_code=404, detail="Сотрудник не найден")
    
    employee.is_active = False
    employee.fired_at = datetime.now()
    db.commit()
    
    return {"status": "deleted"}

@router.get("/{employee_id}/payroll")
async def get_employee_payroll(
    employee_id: str,
    period: Optional[str] = None,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Зарплата сотрудника"""
    employee = db.query(Employee).filter(
        Employee.id == employee_id,
        Employee.user_id == current_user.id
    ).first()
    
    if not employee:
        raise HTTPException(status_code=404, detail="Сотрудник не найден")
    
    query = db.query(PayrollRecord).filter(
        PayrollRecord.employee_id == employee_id,
        PayrollRecord.user_id == current_user.id
    )
    
    if period:
        query = query.filter(PayrollRecord.period == period)
    
    records = query.order_by(PayrollRecord.period.desc()).all()
    
    return [{
        "id": rec.id,
        "period": rec.period,
        "salary": float(rec.salary),
        "bonus": float(rec.bonus),
        "total_accrued": float(rec.total_accrued),
        "opv": float(rec.opv),
        "osms": float(rec.osms),
        "ipn": float(rec.opv_ipn),
        "social_tax": float(rec.social_tax),
        "total_deductions": float(rec.total_deductions),
        "net_salary": float(rec.net_salary),
        "status": rec.status,
        "paid_at": rec.paid_at.isoformat() if rec.paid_at else None
    } for rec in records]

@router.post("/{employee_id}/payroll/calculate")
async def calculate_employee_payroll(
    employee_id: str,
    period: str,
    bonus: float = 0,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Расчёт зарплаты сотрудника"""
    employee = db.query(Employee).filter(
        Employee.id == employee_id,
        Employee.user_id == current_user.id
    ).first()
    
    if not employee:
        raise HTTPException(status_code=404, detail="Сотрудник не найден")
    
    # Расчёт
    result = calculate_payroll(
        salary=float(employee.salary),
        bonus=bonus
    )
    
    return {
        "employee_id": employee_id,
        "employee_name": employee.full_name,
        "period": period,
        "salary": float(result.salary),
        "bonus": float(result.bonus),
        "total_accrued": float(result.total_accrued),
        "deductions": {
            "opv": float(result.opv),
            "opv_ipn": float(result.opv_ipn),
            "osms": float(result.osms),
            "social_tax": float(result.social_tax),
            "total": float(result.total_deductions)
        },
        "net_salary": float(result.net_salary),
        "employer_taxes": calculate_employer_taxes(float(employee.salary))
    }

@router.post("/{employee_id}/payroll/accrue")
async def accrue_payroll(
    employee_id: str,
    period: str,
    bonus: float = 0,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Начисление зарплаты"""
    employee = db.query(Employee).filter(
        Employee.id == employee_id,
        Employee.user_id == current_user.id
    ).first()
    
    if not employee:
        raise HTTPException(status_code=404, detail="Сотрудник не найден")
    
    # Расчёт
    result = calculate_payroll(
        salary=float(employee.salary),
        bonus=bonus
    )
    
    # Создание записи
    record = PayrollRecord(
        id=str(uuid.uuid4()),
        user_id=current_user.id,
        employee_id=employee_id,
        period=period,
        salary=result.salary,
        bonus=result.bonus,
        total_accrued=result.total_accrued,
        opv=result.opv,
        opv_ipn=result.opv_ipn,
        social_tax=result.social_tax,
        osms=result.osms,
        total_deductions=result.total_deductions,
        net_salary=result.net_salary,
        status="accrued"
    )
    
    db.add(record)
    db.commit()
    db.refresh(record)
    
    return {
        "id": record.id,
        "status": "accrued",
        "net_salary": float(result.net_salary)
    }

@router.get("/payroll/report")
async def get_payroll_report(
    period: str,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Отчёт по зарплате за период"""
    records = db.query(PayrollRecord).filter(
        PayrollRecord.user_id == current_user.id,
        PayrollRecord.period == period
    ).all()
    
    total_salary = sum(r.salary for r in records)
    total_bonus = sum(r.bonus for r in records)
    total_accrued = sum(r.total_accrued for r in records)
    total_deductions = sum(r.total_deductions for r in records)
    total_net = sum(r.net_salary for r in records)
    total_opv = sum(r.opv for r in records)
    total_osms = sum(r.osms for r in records)
    total_social_tax = sum(r.social_tax for r in records)
    
    return {
        "period": period,
        "employees_count": len(records),
        "total_salary": float(total_salary),
        "total_bonus": float(total_bonus),
        "total_accrued": float(total_accrued),
        "total_deductions": {
            "opv": float(total_opv),
            "osms": float(total_osms),
            "social_tax": float(total_social_tax),
            "total": float(total_deductions)
        },
        "total_net_salary": float(total_net)
    }
