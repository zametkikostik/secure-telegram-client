"""Главный модуль приложения - Production Ready"""
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from contextlib import asynccontextmanager
from loguru import logger
import os

from .core.config import settings, DATA_DIR, LOGS_DIR
from .core.database import init_db
from .core.tax_rules import TAX_RULES, INCOME_CATEGORIES, EXPENSE_CATEGORIES
from .utils.logging import setup_logging
from .utils.middleware import RateLimitMiddleware, SecurityHeadersMiddleware, RequestLoggingMiddleware

# API роутеры
from .api.auth import router as auth_router
from .api.transactions import router as transactions_router
from .api.reports import router as reports_router
from .api.ai_api import router as ai_router
from .api.integrations import router as integrations_router
from .api.telegram import router as telegram_router
from .api.ocr import router as ocr_router
from .api.documents import router as documents_router
from .api.employees import router as employees_router
from .api.kpi import router as kpi_router
from .api.two_fa import router as two_fa_router
from .api.notifications import router as notifications_router
from .api.one_c import router as one_c_router
from .api.kaspi_shop import router as kaspi_shop_router
from .api.tax_kz import router as tax_kz_router
from .api.payment_slips import router as payment_slips_router
from .api.tax_engine_api import router as tax_engine_router

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifecycle manager"""
    # Startup
    setup_logging()
    init_db()
    
    logger.info("🇰🇿 AI-Бухгалтер запускается...")
    logger.info(f"Окружение: {settings.ENVIRONMENT}")
    logger.info(f"Налоговая ставка: {TAX_RULES.rate_percent}")
    logger.info(f"Лимит дохода: {TAX_RULES.max_income:,} ₸")
    logger.info(f"База данных: {settings.DATABASE_URL or settings.DATABASE_URL_SQLITE}")
    logger.info(f"AI модель: {settings.OPENAI_MODEL}")
    logger.info(f"Telegram уведомления: {'включены' if settings.TELEGRAM_ENABLED else 'отключены'}")
    
    yield
    
    # Shutdown
    logger.info("AI-Бухгалтер останавливается")

app = FastAPI(
    title=settings.APP_NAME,
    version="1.0.0",
    description="AI-помощник для ИП на упрощенке в Казахстане",
    lifespan=lifespan
)

# ===== Middleware =====
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.add_middleware(RateLimitMiddleware, requests_per_minute=settings.RATE_LIMIT_PER_MINUTE)
app.add_middleware(SecurityHeadersMiddleware)
app.add_middleware(RequestLoggingMiddleware)

# ===== Статика и шаблоны =====
app.mount("/static", StaticFiles(directory="static"), name="static")
templates = Jinja2Templates(directory="templates")

# ===== API роуты =====
app.include_router(auth_router)
app.include_router(transactions_router)
app.include_router(reports_router)
app.include_router(ai_router)
app.include_router(integrations_router)
app.include_router(telegram_router)
app.include_router(ocr_router)
app.include_router(documents_router)
app.include_router(employees_router)
app.include_router(kpi_router)
app.include_router(two_fa_router)
app.include_router(notifications_router)
app.include_router(one_c_router)
app.include_router(kaspi_shop_router)
app.include_router(tax_kz_router)
app.include_router(payment_slips_router)
app.include_router(tax_engine_router)

# ===== Web роуты =====
@app.get("/")
async def root(request: Request):
    """Главная страница - веб интерфейс"""
    return templates.TemplateResponse("index.html", {
        "request": request,
        "tax_rate": TAX_RULES.rate_percent,
        "max_income": TAX_RULES.max_income,
        "app_name": settings.APP_NAME
    })

@app.get("/auth")
async def auth_page(request: Request):
    """Страница входа/регистрации"""
    return templates.TemplateResponse("auth.html", {
        "request": request,
        "app_name": settings.APP_NAME
    })

@app.get("/tax-calculator")
async def tax_calculator_page(request: Request):
    """Страница налогового калькулятора ИП Казахстан 2026"""
    return templates.TemplateResponse("tax-calculator.html", {
        "request": request
    })

@app.get("/static/manifest.json")
async def get_manifest(request: Request):
    """PWA Manifest"""
    from starlette.responses import FileResponse
    return FileResponse("static/manifest.json", media_type="application/json")

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "environment": settings.ENVIRONMENT,
        "version": "1.0.0"
    }

@app.get("/api/v1/categories")
async def get_categories():
    """Категории доходов и расходов"""
    return {
        "income": INCOME_CATEGORIES,
        "expense": EXPENSE_CATEGORIES
    }

# ===== Error handlers =====
@app.exception_handler(404)
async def not_found_handler(request: Request, exc):
    return templates.TemplateResponse("404.html", {
        "request": request,
        "path": request.url.path
    }, status_code=404)

@app.exception_handler(500)
async def internal_error_handler(request: Request, exc):
    logger.error(f"Internal error: {exc}")
    return templates.TemplateResponse("500.html", {
        "request": request
    }, status_code=500)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "src.main:app",
        host="0.0.0.0",
        port=8000,
        reload=settings.DEBUG,
        workers=1 if settings.DEBUG else 4
    )
