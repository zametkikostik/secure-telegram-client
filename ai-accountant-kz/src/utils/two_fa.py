"""2FA (TOTP) для двухфакторной аутентификации"""
import pyotp
import base64
import qrcode
from io import BytesIO
from typing import Optional, Tuple
from loguru import logger

from ..core.config import settings


class TwoFactorAuth:
    """2FA на основе TOTP (Time-based One-Time Password)"""
    
    def __init__(self):
        self.issuer = settings.APP_NAME
        self.algorithm = "SHA1"
        self.digits = 6
        self.period = 30
    
    def generate_secret(self) -> str:
        """Генерация нового секрета"""
        secret = pyotp.random_base32()
        logger.info("Generated new 2FA secret")
        return secret
    
    def get_provisioning_uri(self, email: str, secret: str) -> str:
        """Генерация URI для QR-кода"""
        totp = pyotp.TOTP(
            secret,
            issuer=self.issuer,
            algorithm=self.algorithm,
            digits=self.digits,
            period=self.period
        )
        return totp.provisioning_uri(name=email)
    
    def generate_qr_code(self, email: str, secret: str) -> str:
        """
        Генерация QR-кода для настройки 2FA
        
        Returns:
            base64 encoded PNG image
        """
        uri = self.get_provisioning_uri(email, secret)
        
        img = qrcode.make(uri)
        buffered = BytesIO()
        img.save(buffered, format="PNG")
        
        img_base64 = base64.b64encode(buffered.getvalue()).decode()
        logger.info(f"Generated QR code for {email}")
        
        return f"data:image/png;base64,{img_base64}"
    
    def verify_code(self, secret: str, code: str) -> bool:
        """
        Проверка 2FA кода
        
        Args:
            secret: Секрет пользователя
            code: 6-значный код из приложения
        
        Returns:
            True если код верный
        """
        totp = pyotp.TOTP(
            secret,
            algorithm=self.algorithm,
            digits=self.digits,
            period=self.period
        )
        
        # Проверяем код с допуском ±1 периода (для синхронизации времени)
        is_valid = totp.verify(code, valid_window=1)
        
        if is_valid:
            logger.info("2FA code verified successfully")
        else:
            logger.warning("2FA code verification failed")
        
        return is_valid
    
    def get_backup_codes(self, count: int = 10) -> list:
        """Генерация резервных кодов"""
        import secrets
        import string
        
        codes = []
        for _ in range(count):
            # Генерируем 8-значный код из цифр и букв
            code = ''.join(secrets.choice(string.ascii_uppercase + string.digits) for _ in range(8))
            codes.append(code)
        
        logger.info(f"Generated {count} backup codes")
        return codes


# Глобальный экземпляр
totp_auth = TwoFactorAuth()
