"""
AI Accountant KZ - Production Ready Tax API Service
FastAPI приложение для расчета налогов ИП Казахстан 2026

Endpoints:
- POST /v1/calculate/net - расчет от суммы на руки
- POST /v1/calculate/gross - расчет от оклада
- GET /v1/health - health check
- GET /v1/rates - налоговые ставки
"""
from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from contextlib import asynccontextmanager
import time
import logging
import sys
from typing import Callable

from .core.config import settings
from .core.logging_config import setup_logging, set_trace_id, get_trace_id, LoggingMiddleware
from .api.tax import router as tax_router

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


# ===== Middleware =====

@app.middleware("http")
async def performance_monitoring_middleware(
    request: Request,
    call_next: Callable
) -> Response:
    """
    Middleware для мониторинга производительности
    Логирует время выполнения каждого запроса
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
    
    logger.info(
        f"Request: {method} {path}",
        extra={
            "extra_data": {
                "client_ip": client_host,
                "user_agent": request.headers.get("user-agent", "unknown")
            }
        }
    )
    
    try:
        response = await call_next(request)
        
        # Расчет времени выполнения
        duration = time.time() - start_time
        
        logger.info(
            f"Response: {method} {path} - Status: {response.status_code} - Duration: {duration:.3f}s",
            extra={
                "extra_data": {
                    "duration_ms": round(duration * 1000, 2),
                    "status_code": response.status_code
                }
            }
        )
        
        # Добавляем заголовки с timing
        response.headers["X-Process-Time"] = str(round(duration * 1000, 2))
        response.headers["X-Trace-ID"] = trace_id
        
        return response
    
    except Exception as e:
        duration = time.time() - start_time
        logger.error(
            f"Error: {method} {path} - Duration: {duration:.3f}s - Error: {str(e)}",
            exc_info=True,
            extra={
                "extra_data": {
                    "duration_ms": round(duration * 1000, 2),
                    "error_type": type(e).__name__
                }
            }
        )
        raise


@app.middleware("http")
async def security_headers_middleware(
    request: Request,
    call_next: Callable
) -> Response:
    """Middleware для добавления security headers"""
    response = await call_next(request)
    
    # Security headers
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-Frame-Options"] = "DENY"
    response.headers["X-XSS-Protection"] = "1; mode=block"
    response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
    response.headers["Content-Security-Policy"] = "default-src 'self'"
    
    return response


# CORS Middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
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
        "health": "/v1/health"
    }


# ===== Error handlers =====

@app.exception_handler(404)
async def not_found_handler(request: Request, exc):
    """Обработчик 404 ошибок"""
    logger.warning(f"404 Not Found: {request.method} {request.url.path}")
    return {
        "error": "not_found",
        "message": f"Endpoint не найден: {request.method} {request.url.path}",
        "trace_id": get_trace_id()
    }


@app.exception_handler(500)
async def internal_error_handler(request: Request, exc):
    """Обработчик 500 ошибок"""
    logger.error(f"Internal Error: {exc}", exc_info=True)
    return {
        "error": "internal_error",
        "message": "Внутренняя ошибка сервера",
        "trace_id": get_trace_id()
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
