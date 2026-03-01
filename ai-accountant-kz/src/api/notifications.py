"""API endpoints для Email/SMS уведомлений"""
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from pydantic import BaseModel, EmailStr
from typing import Optional

from ..core.database import get_db, User
from ..core.security import get_current_user
from ..utils.email_service import email_service
from ..utils.sms_service import sms_service

router = APIRouter(prefix="/api/v1/notifications", tags=["notifications"])

class SendEmailRequest(BaseModel):
    to: EmailStr
    subject: str
    message: str

class SendSMSRequest(BaseModel):
    phone: str
    message: str

class NotificationSettings(BaseModel):
    email_enabled: bool = True
    sms_enabled: bool = False
    telegram_enabled: bool = True
    tax_reminder: bool = True
    declaration_reminder: bool = True
    transaction_alerts: bool = False

@router.post("/send/email")
async def send_email(
    request: SendEmailRequest,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Отправка Email (для тестирования)"""
    success = await email_service.send_email(
        to=request.to,
        subject=request.subject,
        html=f"<p>{request.message}</p>"
    )
    
    if success:
        return {"status": "success", "message": "Email отправлен"}
    else:
        raise HTTPException(status_code=500, detail="Не удалось отправить Email")

@router.post("/send/sms")
async def send_sms(
    request: SendSMSRequest,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Отправка SMS (для тестирования)"""
    success = await sms_service.send_sms(
        phone=request.phone,
        message=request.message
    )
    
    if success:
        return {"status": "success", "message": "SMS отправлено"}
    else:
        raise HTTPException(status_code=500, detail="Не удалось отправить SMS")

@router.get("/settings")
async def get_notification_settings(
    current_user: User = Depends(get_current_user)
):
    """Настройки уведомлений пользователя"""
    # TODO: Сохранять настройки в БД
    return {
        "email_enabled": True,
        "sms_enabled": False,
        "telegram_enabled": True,
        "tax_reminder": True,
        "declaration_reminder": True,
        "transaction_alerts": False
    }

@router.post("/settings")
async def update_notification_settings(
    settings: NotificationSettings,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Обновление настроек уведомлений"""
    # TODO: Сохранить в БД
    return {"status": "success", "message": "Настройки сохранены"}

@router.post("/test/email")
async def test_email(
    current_user: User = Depends(get_current_user)
):
    """Тестовое Email уведомление"""
    success = await email_service.send_email(
        to=current_user.email,
        subject="🇰🇿 Тест от AI-Бухгалтер",
        html="""
        <html>
        <body>
            <h1>✅ Тест прошёл успешно!</h1>
            <p>Если вы видите это письмо - Email уведомления работают.</p>
        </body>
        </html>
        """
    )
    
    if success:
        return {"status": "success", "message": "Тестовое письмо отправлено"}
    else:
        raise HTTPException(status_code=500, detail="Email сервис не настроен")

@router.post("/test/sms")
async def test_sms(
    current_user: User = Depends(get_current_user)
):
    """Тестовое SMS уведомление"""
    if not current_user.phone:
        raise HTTPException(status_code=400, detail="Укажите номер телефона в профиле")
    
    success = await sms_service.send_sms(
        phone=current_user.phone,
        message="🇰🇿 AI-Бухгалтер: Тест SMS прошёл успешно!"
    )
    
    if success:
        return {"status": "success", "message": "Тестовое SMS отправлено"}
    else:
        raise HTTPException(status_code=500, detail="SMS сервис не настроен")
