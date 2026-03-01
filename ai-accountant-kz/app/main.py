"""
AI Accountant KZ - Production Ready Tax API Service
FastAPI приложение для расчета налогов ИП Казахстан 2026

Endpoints:
- POST /api/v1/calculate — основной расчет
- GET /api/v1/health — health check
- GET /api/v1/config — ставки МРП/МЗП
"""
from fastapi import FastAPI, Request, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from fastapi.exceptions import RequestValidationError
from contextlib import asynccontextmanager
import time
import logging
import sys
from typing import Callable, Union
from datetime import datetime, timezone

from .core.config import settings
from .core.logging_config import setup_logging, set_trace_id, get_trace_id
from .api.tax import router as tax_router
from .schemas.tax import ErrorResponse

# Настройка логирования
setup_logging(
    level=settings.LOG_LEVEL,
    log_format=settings.LOG_FORMAT,
    service_name=settings.APP_NAME
)

logger = logging.getLogger("ai-accountant")


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifecycle manager для приложения"""
    # Startup
    logger.info("🇰🇿 AI Accountant KZ запускается...")
    logger.info(f"Окружение: {settings.ENVIRONMENT}")
    logger.info(f"Debug: {settings.DEBUG}")
    logger.info(f"API Version: {settings.API_VERSION}")
    logger.info(f"Rate Limit: {settings.RATE_LIMIT_PER_MINUTE} req/min")
    logger.info(f"CORS Origins: {settings.CORS_ORIGINS}")

    yield

    # Shutdown
    logger.info("AI Accountant KZ останавливается")


app = FastAPI(
    title=settings.APP_NAME,
    version="2026.03.01",
    description="Production Ready API для расчета налогов ИП Казахстан 2026",
    docs_url="/docs",
    redoc_url="/redoc",
    openapi_url="/openapi.json",
    lifespan=lifespan
)


# ===== Custom Exception Handler =====

@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    """
    Глобальный обработчик исключений
    
    Возвращает чистый JSON с кодом ошибки вместо 500 страницы
    """
    trace_id = get_trace_id()
    timestamp = datetime.now(timezone.utc).isoformat()
    
    logger.error(
        f"Unhandled exception: {type(exc).__name__}: {str(exc)}",
        exc_info=True,
        extra={"extra_data": {"trace_id": trace_id}}
    )
    
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content=ErrorResponse(
            success=False,
            error="internal_error",
            message="Внутренняя ошибка сервера",
            trace_id=trace_id,
            timestamp=timestamp
        ).model_dump()
    )


@app.exception_handler(RequestValidationError)
async def validation_exception_handler(
    request: Request, 
    exc: RequestValidationError
) -> JSONResponse:
    """
    Обработчик ошибок валидации
    
    Возвращает детальную информацию об ошибках валидации
    """
    trace_id = get_trace_id()
    timestamp = datetime.now(timezone.utc).isoformat()
    
    errors = []
    for error in exc.errors():
        errors.append({
            "field": ".".join(str(x) for x in error.get("loc", [])),
            "message": error.get("msg", ""),
            "type": error.get("type", "")
        })
    
    logger.warning(f"Validation error: {errors}")
    
    return JSONResponse(
        status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
        content=ErrorResponse(
            success=False,
            error="validation_error",
            message=f"Ошибка валидации данных: {errors[0]['message'] if errors else 'Неверные данные'}",
            trace_id=trace_id,
            timestamp=timestamp
        ).model_dump()
    )


# ===== Middleware =====

@app.middleware("http")
async def performance_monitoring_middleware(
    request: Request,
    call_next: Callable
) -> Union[Request, Callable]:
    """
    Middleware для мониторинга производительности
    Логирует время выполнения каждого запроса с Trace-ID
    """
    # Генерируем trace_id для запроса
    import uuid
    trace_id = str(uuid.uuid4())
    set_trace_id(trace_id)
    
    start_time = time.time()
    
    # Получаем информацию о запросе
    method = request.method
    path = request.url.path
    client_host = request.client.host if request.client else "unknown"
    
    # Формат: [TIMESTAMP] [LEVEL] [TRACE_ID] Message
    logger.info(
        f"[{datetime.now(timezone.utc).isoformat()}] [INFO] [{trace_id}] "
        f"Request: {method} {path} from {client_host}"
    )
    
    try:
        response = await call_next(request)
        
        # Расчет времени выполнения
        duration = time.time() - start_time
        
        logger.info(
            f"[{datetime.now(timezone.utc).isoformat()}] [INFO] [{trace_id}] "
            f"Response: {method} {path} - Status: {response.status_code} - Duration: {duration*1000:.2f}ms"
        )
        
        # Добавляем заголовки с timing
        response.headers["X-Process-Time"] = str(round(duration * 1000, 2))
        response.headers["X-Trace-ID"] = trace_id
        
        return response
    
    except Exception as e:
        duration = time.time() - start_time
        logger.error(
            f"[{datetime.now(timezone.utc).isoformat()}] [ERROR] [{trace_id}] "
            f"Error: {method} {path} - Duration: {duration*1000:.2f}ms - Error: {str(e)}",
            exc_info=True
        )
        raise


@app.middleware("http")
async def security_headers_middleware(request: Request, call_next: Callable):
    """Middleware для добавления security headers"""
    response = await call_next(request)
    
    # Security headers
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-Frame-Options"] = "DENY"
    response.headers["X-XSS-Protection"] = "1; mode=block"
    response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
    response.headers["Content-Security-Policy"] = "default-src 'self'"
    response.headers["Referrer-Policy"] = "strict-origin-when-cross-origin"
    
    return response


# ===== CORS Middleware =====
# Ограничиваем доступ только с доверенных доменов

allowed_origins = settings.CORS_ORIGINS if settings.CORS_ORIGINS != ["*"] else [
    "https://ai-accountant.kz",
    "https://www.ai-accountant.kz",
    "https://api.ai-accountant.kz"
]

if settings.DEBUG:
    allowed_origins.append("http://localhost:3000")
    allowed_origins.append("http://localhost:8000")

app.add_middleware(
    CORSMiddleware,
    allow_origins=allowed_origins,  # Только доверенные домены
    allow_credentials=True,
    allow_methods=["GET", "POST", "OPTIONS"],
    allow_headers=["Authorization", "Content-Type", "X-API-Key"],
    expose_headers=["X-Trace-ID", "X-Process-Time"],
    max_age=3600,
)


# ===== API Routes =====

app.include_router(tax_router)


# ===== Root endpoint =====

@app.get("/", tags=["root"])
async def root():
    """Корневой endpoint с информацией о сервисе"""
    return {
        "service": settings.APP_NAME,
        "version": "2026.03.01",
        "environment": settings.ENVIRONMENT,
        "docs": "/docs",
        "health": "/api/v1/health",
        "config": "/api/v1/config",
        "calculate": "/api/v1/calculate"
    }


if __name__ == "__main__":
    import uvicorn
    
    uvicorn.run(
        "app.main:app",
        host="0.0.0.0",
        port=8000,
        reload=settings.DEBUG,
        workers=1 if settings.DEBUG else 4,
        log_level=settings.LOG_LEVEL.lower()
    )
