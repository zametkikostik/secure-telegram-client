"""API endpoints для 2FA"""
from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from pydantic import BaseModel
import json

from ..core.database import get_db, User
from ..core.security import get_current_user, verify_password
from ..utils.two_fa import totp_auth

router = APIRouter(prefix="/api/v1/2fa", tags=["2fa"])

class Enable2FARequest(BaseModel):
    password: str

class Verify2FARequest(BaseModel):
    code: str

class Disable2FARequest(BaseModel):
    password: str
    code: str

class BackupCodesResponse(BaseModel):
    backup_codes: list

@router.post("/enable")
async def enable_2fa(
    request: Enable2FARequest,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """
    Включение 2FA (шаг 1)
    
    Генерирует секрет и QR-код для настройки
    """
    # Проверка пароля
    if not verify_password(request.password, current_user.hashed_password):
        raise HTTPException(status_code=400, detail="Неверный пароль")
    
    # Генерация секрета
    secret = totp_auth.generate_secret()
    
    # Генерация QR-кода
    qr_code = totp_auth.generate_qr_code(current_user.email, secret)
    
    # Генерация резервных кодов
    backup_codes = totp_auth.get_backup_codes()
    
    # Сохраняем в сессии (пока не подтверждено)
    # В продакшене использовать Redis
    current_user.two_factor_secret = secret
    current_user.backup_codes = json.dumps(backup_codes)
    db.commit()
    
    return {
        "secret": secret,
        "qr_code": qr_code,
        "backup_codes": backup_codes,
        "message": "Отсканируйте QR-код и введите код из приложения"
    }

@router.post("/verify")
async def verify_2fa(
    request: Verify2FARequest,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """
    Подтверждение включения 2FA (шаг 2)
    
    Проверяет код из приложения и активирует 2FA
    """
    if not current_user.two_factor_secret:
        raise HTTPException(status_code=400, detail="2FA не настроена")
    
    # Проверка кода
    is_valid = totp_auth.verify_code(current_user.two_factor_secret, request.code)
    
    if not is_valid:
        raise HTTPException(status_code=400, detail="Неверный код")
    
    # Активируем 2FA
    current_user.two_factor_enabled = True
    db.commit()
    
    return {
        "status": "success",
        "message": "2FA успешно включена"
    }

@router.post("/disable")
async def disable_2fa(
    request: Disable2FARequest,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Отключение 2FA"""
    # Проверка пароля
    if not verify_password(request.password, current_user.hashed_password):
        raise HTTPException(status_code=400, detail="Неверный пароль")
    
    # Проверка 2FA кода (или резервного)
    if current_user.two_factor_enabled:
        is_valid = totp_auth.verify_code(current_user.two_factor_secret, request.code)
        
        # Проверка резервного кода
        if not is_valid and current_user.backup_codes:
            backup_codes = json.loads(current_user.backup_codes)
            if request.code in backup_codes:
                backup_codes.remove(request.code)
                current_user.backup_codes = json.dumps(backup_codes)
                is_valid = True
        
        if not is_valid:
            raise HTTPException(status_code=400, detail="Неверный код")
    
    # Отключаем 2FA
    current_user.two_factor_enabled = False
    current_user.two_factor_secret = None
    current_user.backup_codes = None
    db.commit()
    
    return {"status": "success", "message": "2FA отключена"}

@router.get("/status")
async def get_2fa_status(
    current_user: User = Depends(get_current_user)
):
    """Статус 2FA пользователя"""
    return {
        "enabled": current_user.two_factor_enabled,
        "configured": current_user.two_factor_secret is not None
    }

@router.post("/regenerate-backup-codes")
async def regenerate_backup_codes(
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Перегенерация резервных кодов"""
    if not current_user.two_factor_enabled:
        raise HTTPException(status_code=400, detail="2FA не включена")
    
    # Генерация новых кодов
    backup_codes = totp_auth.get_backup_codes()
    current_user.backup_codes = json.dumps(backup_codes)
    db.commit()
    
    return {"backup_codes": backup_codes}
