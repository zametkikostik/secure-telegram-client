"""JSON логирование с Trace ID для ELK/Loki"""
import logging
import json
import sys
import traceback
from datetime import datetime
from typing import Any, Dict
import uuid
from contextvars import ContextVar

# Context variable для хранения trace_id в рамках одного запроса
trace_id_var: ContextVar[str] = ContextVar('trace_id', default='')


def get_trace_id() -> str:
    """Получение текущего trace_id"""
    return trace_id_var.get() or str(uuid.uuid4())


def set_trace_id(trace_id: str) -> None:
    """Установка trace_id для текущего контекста"""
    trace_id_var.set(trace_id)


class TextFormatter(logging.Formatter):
    """Текстовый форматтер для логов в формате: [TIMESTAMP] [LEVEL] [TRACE_ID] Message"""
    
    def format(self, record: logging.LogRecord) -> str:
        """Форматирование записи в текстовом виде"""
        timestamp = datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S')
        level = record.levelname
        trace_id = getattr(record, 'trace_id', get_trace_id())
        message = record.getMessage()
        
        return f"[{timestamp}] [{level}] [{trace_id}] {message}"


class JSONFormatter(logging.Formatter):
    """JSON форматтер для логов"""
    
    def __init__(self, service_name: str = "ai-accountant-kz"):
        super().__init__()
        self.service_name = service_name
    
    def format(self, record: logging.LogRecord) -> str:
        """Форматирование записи в JSON"""
        log_entry: Dict[str, Any] = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "service": self.service_name,
            "trace_id": get_trace_id(),
            "thread": record.thread,
            "process": record.process,
        }
        
        # Добавляем extra поля если есть
        if hasattr(record, 'extra_data'):
            log_entry.update(record.extra_data)
        
        # Добавляем информацию об исключении
        if record.exc_info:
            log_entry["exception"] = {
                "type": record.exc_info[0].__name__ if record.exc_info[0] else None,
                "message": str(record.exc_info[1]) if record.exc_info[1] else None,
                "traceback": traceback.format_exception(*record.exc_info)
            }
        
        # Добавляем location для отладки
        if record.filename:
            log_entry["location"] = {
                "file": record.filename,
                "line": record.lineno,
                "function": record.funcName
            }
        
        return json.dumps(log_entry, ensure_ascii=False, default=str)


class TraceIDFilter(logging.Filter):
    """Фильтр для добавления trace_id в записи"""
    
    def filter(self, record: logging.LogRecord) -> bool:
        """Добавление trace_id в запись"""
        record.trace_id = get_trace_id()
        return True


def setup_logging(
    level: str = "INFO",
    log_format: str = "text",
    service_name: str = "ai-accountant-kz"
) -> logging.Logger:
    """
    Настройка логирования
    
    Args:
        level: Уровень логирования (DEBUG, INFO, WARNING, ERROR, CRITICAL)
        log_format: Формат логов (json, text) - по умолчанию text для формата [TIMESTAMP] [LEVEL] [TRACE_ID]
        service_name: Имя сервиса для логов
    
    Returns:
        Настроенный logger
    """
    # Создаем logger
    logger = logging.getLogger("ai-accountant")
    logger.setLevel(getattr(logging, level.upper()))
    
    # Создаем handler для stdout
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(getattr(logging, level.upper()))
    
    # Выбираем форматтер
    if log_format == "json":
        formatter = JSONFormatter(service_name)
    else:
        # Текстовый формат: [TIMESTAMP] [LEVEL] [TRACE_ID] Message
        formatter = TextFormatter()
    
    # Добавляем фильтр trace_id
    trace_filter = TraceIDFilter()
    console_handler.addFilter(trace_filter)
    
    console_handler.setFormatter(formatter)
    logger.addHandler(console_handler)
    
    # Создаем handler для файла (опционально)
    try:
        file_handler = logging.FileHandler("logs/accountant.log", encoding='utf-8')
        file_handler.setLevel(getattr(logging, level.upper()))
        file_handler.setFormatter(formatter)
        file_handler.addFilter(trace_filter)
        logger.addHandler(file_handler)
    except Exception as e:
        logger.warning(f"Не удалось создать файл логов: {e}")
    
    return logger


class LoggingMiddleware:
    """Middleware для логирования запросов с trace_id"""
    
    def __init__(self, app):
        self.app = app
        self.logger = logging.getLogger("ai-accountant")
    
    async def __call__(self, scope, receive, send):
        """Обработка запроса с логированием"""
        if scope['type'] != 'http':
            return await self.app(scope, receive, send)
        
        # Генерируем trace_id для запроса
        trace_id = str(uuid.uuid4())
        set_trace_id(trace_id)
        
        # Получаем путь и метод
        path = scope.get('path', '')
        method = scope.get('method', '')
        
        # Логируем начало запроса
        self.logger.info(f"Request started: {method} {path}")
        
        # Отслеживаем время выполнения
        import time
        start_time = time.time()
        
        # Оборачиваем send для перехвата статуса ответа
        status_code = 500
        
        async def send_wrapper(message):
            nonlocal status_code
            if message['type'] == 'http.response.start':
                status_code = message['status']
            await send(message)
        
        try:
            await self.app(scope, receive, send_wrapper)
        finally:
            # Логируем завершение запроса
            duration = time.time() - start_time
            self.logger.info(
                f"Request completed: {method} {path} - Status: {status_code} - Duration: {duration:.3f}s",
                extra={'extra_data': {'duration_ms': round(duration * 1000, 2), 'status_code': status_code}}
            )
