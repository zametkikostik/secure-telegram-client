"""Middleware для приложения"""
from fastapi import Request, HTTPException, status
from fastapi.responses import JSONResponse
from starlette.middleware.base import BaseHTTPMiddleware
from collections import defaultdict
from datetime import datetime, timedelta
from typing import Dict
import time

from ..core.config import settings

class RateLimitMiddleware(BaseHTTPMiddleware):
    """Rate limiting middleware"""
    
    def __init__(self, app, requests_per_minute: int = None):
        super().__init__(app)
        self.limit = requests_per_minute or settings.RATE_LIMIT_PER_MINUTE
        self.requests: Dict[str, list] = defaultdict(list)
    
    async def dispatch(self, request: Request, call_next):
        # Пропускаем health check
        if request.url.path == "/health":
            return await call_next(request)
        
        # Получаем IP клиента
        client_ip = request.client.host
        now = time.time()
        
        # Очищаем старые запросы (старше 1 минуты)
        self.requests[client_ip] = [
            req_time for req_time in self.requests[client_ip]
            if now - req_time < 60
        ]
        
        # Проверяем лимит
        if len(self.requests[client_ip]) >= self.limit:
            return JSONResponse(
                status_code=status.HTTP_429_TOO_MANY_REQUESTS,
                content={"detail": "Слишком много запросов. Попробуйте позже."},
                headers={"Retry-After": "60"}
            )
        
        # Добавляем текущий запрос
        self.requests[client_ip].append(now)
        
        return await call_next(request)

class SecurityHeadersMiddleware(BaseHTTPMiddleware):
    """Middleware для добавления security headers"""
    
    async def dispatch(self, request: Request, call_next):
        response = await call_next(request)
        
        # Security headers
        response.headers["X-Content-Type-Options"] = "nosniff"
        response.headers["X-Frame-Options"] = "DENY"
        response.headers["X-XSS-Protection"] = "1; mode=block"
        response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
        response.headers["Content-Security-Policy"] = "default-src 'self'; script-src 'self' 'unsafe-inline' cdn.jsdelivr.net; style-src 'self' 'unsafe-inline'"
        
        return response

class RequestLoggingMiddleware(BaseHTTPMiddleware):
    """Middleware для логирования запросов"""
    
    async def dispatch(self, request: Request, call_next):
        from loguru import logger
        
        start_time = time.time()
        
        response = await call_next(request)
        
        duration = time.time() - start_time
        
        logger.info(
            f"{request.method} {request.url.path} - "
            f"{response.status_code} - {duration:.3f}s"
        )
        
        return response
