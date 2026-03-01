"""Конфигурация приложения через pydantic-settings"""
from pydantic_settings import BaseSettings, SettingsConfigDict
from pydantic import Field, field_validator
from typing import Optional
from functools import lru_cache
import secrets


class Settings(BaseSettings):
    """Настройки приложения с валидацией"""
    
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore"
    )
    
    # ===== Приложение =====
    APP_NAME: str = Field(default="AI Accountant KZ Tax API", description="Название приложения")
    DEBUG: bool = Field(default=False, description="Режим отладки")
    ENVIRONMENT: str = Field(default="production", description="Окружение")
    API_VERSION: str = Field(default="v1", description="Версия API")
    
    # ===== Безопасность =====
    SECRET_KEY: str = Field(
        default="", 
        description="Секретный ключ для JWT"
    )
    API_KEY: str = Field(
        default="", 
        description="API Key для аутентификации запросов"
    )
    ALGORITHM: str = Field(default="HS256", description="Алгоритм шифрования")
    ACCESS_TOKEN_EXPIRE_MINUTES: int = Field(default=10080, description="Время жизни токена (7 дней)")
    
    # ===== AI (OpenRouter/DashScope) =====
    OPENROUTER_API_KEY: str = Field(default="", description="OpenRouter API Key")
    OPENAI_API_KEY: str = Field(default="", description="OpenAI/DashScope API Key")
    OPENAI_MODEL: str = Field(default="qwen-plus", description="AI модель")
    DASHSCOPE_BASE_URL: str = Field(
        default="https://dashscope.aliyuncs.com/compatible-mode/v1",
        description="DashScope базовый URL"
    )
    
    # ===== База данных =====
    DATABASE_URL: Optional[str] = Field(default=None, description="PostgreSQL URL")
    DATABASE_URL_SQLITE: str = Field(
        default="sqlite:///./data/accountant.db",
        description="SQLite URL"
    )
    
    # ===== Redis =====
    REDIS_URL: str = Field(default="redis://localhost:6379/0", description="Redis URL")
    CELERY_BROKER_URL: str = Field(default="redis://localhost:6379/0", description="Celery broker")
    
    # ===== Логирование =====
    LOG_LEVEL: str = Field(default="INFO", description="Уровень логирования")
    LOG_FORMAT: str = Field(default="json", description="Формат логов (json/text)")
    LOG_FILE: str = Field(default="logs/accountant.log", description="Файл логов")
    
    # ===== CORS =====
    CORS_ORIGINS: list = Field(
        default=["https://ai-accountant.kz", "https://www.ai-accountant.kz"],
        description="Доверенные домены для CORS"
    )
    
    # ===== Rate Limiting =====
    RATE_LIMIT_PER_MINUTE: int = Field(default=60, description="Лимит запросов в минуту")
    
    # ===== Telegram =====
    TELEGRAM_BOT_TOKEN: str = Field(default="", description="Telegram bot token")
    TELEGRAM_CHAT_ID: str = Field(default="", description="Telegram chat ID")
    TELEGRAM_ENABLED: bool = Field(default=False, description="Telegram уведомления")
    
    # ===== Банки =====
    KASPI_API_KEY: str = Field(default="", description="Kaspi API Key")
    KASPI_MERCHANT_ID: str = Field(default="", description="Kaspi Merchant ID")
    HALYK_API_KEY: str = Field(default="", description="Halyk API Key")
    HALYK_CLIENT_ID: str = Field(default="", description="Halyk Client ID")
    
    @field_validator('SECRET_KEY')
    @classmethod
    def validate_secret_key(cls, v: str) -> str:
        """Валидация SECRET_KEY - минимум 32 символа"""
        if not v:
            # Генерируем случайный ключ для dev режима
            return secrets.token_urlsafe(32)
        if len(v) < 32:
            raise ValueError("SECRET_KEY должен быть минимум 32 символа")
        return v
    
    @field_validator('API_KEY')
    @classmethod
    def validate_api_key(cls, v: str) -> str:
        """Валидация API_KEY"""
        if not v:
            # Генерируем случайный API ключ для dev режима
            return secrets.token_urlsafe(24)
        return v
    
    @property
    def database_url(self) -> str:
        """Получение URL базы данных"""
        return self.DATABASE_URL or self.DATABASE_URL_SQLITE


@lru_cache()
def get_settings() -> Settings:
    """Кэшированный инстанс настроек"""
    return Settings()


settings = get_settings()
