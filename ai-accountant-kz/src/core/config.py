"""Конфигурация приложения для продакшена"""
from pydantic_settings import BaseSettings
from pathlib import Path
from functools import lru_cache
import os

class Settings(BaseSettings):
    # ===== Приложение =====
    APP_NAME: str = "AI-Бухгалтер для ИП"
    DEBUG: bool = False
    ENVIRONMENT: str = "production"  # development, staging, production
    
    # ===== База данных =====
    DATABASE_URL: str = ""  # PostgreSQL: postgresql://user:pass@host:5432/db
    DATABASE_URL_SQLITE: str = "sqlite:///./data/accountant.db"
    
    # ===== AI настройки =====
    OPENAI_API_KEY: str = ""
    OPENAI_MODEL: str = "qwen-plus"
    DASHSCOPE_BASE_URL: str = "https://dashscope.aliyuncs.com/compatible-mode/v1"
    AI_CONFIDENCE_THRESHOLD: float = 0.95
    AI_MAX_RETRIES: int = 3
    
    # ===== Безопасность =====
    SECRET_KEY: str = ""  # Должен быть установлен в production
    ALGORITHM: str = "HS256"
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 60 * 24 * 7  # 7 дней
    
    # ===== Telegram =====
    TELEGRAM_BOT_TOKEN: str = ""
    TELEGRAM_CHAT_ID: str = ""
    TELEGRAM_ENABLED: bool = False
    
    # ===== Банки =====
    KASPI_API_KEY: str = ""
    KASPI_MERCHANT_ID: str = ""
    HALYK_API_KEY: str = ""
    HALYK_CLIENT_ID: str = ""
    
    # ===== Celery =====
    CELERY_BROKER_URL: str = "redis://localhost:6379/0"
    CELERY_RESULT_BACKEND: str = "redis://localhost:6379/0"
    
    # ===== Логирование =====
    LOG_LEVEL: str = "INFO"
    LOG_FILE: str = "logs/accountant.log"
    LOG_MAX_SIZE: int = 10 * 1024 * 1024  # 10 MB
    LOG_BACKUP_COUNT: int = 5
    
    # ===== CORS =====
    CORS_ORIGINS: list = ["*"]
    
    # ===== Rate Limiting =====
    RATE_LIMIT_PER_MINUTE: int = 60
    
    class Config:
        env_file = ".env"
        case_sensitive = False
        extra = "ignore"

# Базовые директории
BASE_DIR = Path(__file__).parent.parent.parent
DATA_DIR = BASE_DIR / "data"
LOGS_DIR = BASE_DIR / "logs"

# Создаём директории
DATA_DIR.mkdir(exist_ok=True)
LOGS_DIR.mkdir(exist_ok=True)

@lru_cache()
def get_settings() -> Settings:
    """Кэшированный инстанс настроек"""
    return Settings()

settings = get_settings()

# Авто-настройка SECRET_KEY для разработки
if settings.DEBUG and not settings.SECRET_KEY:
    import secrets
    settings.SECRET_KEY = secrets.token_urlsafe(32)
