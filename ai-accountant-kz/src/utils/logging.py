"""Настройка логирования для продакшена"""
from loguru import logger
from pathlib import Path
import sys
import logging

from ..core.config import settings

def setup_logging():
    """Настройка логирования"""
    
    # Удаляем стандартный handler
    logger.remove()
    
    # Консольный вывод
    logger.add(
        sys.stderr,
        format="<green>{time:YYYY-MM-DD HH:mm:ss}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan>:<cyan>{function}</cyan>:<cyan>{line}</cyan> - <level>{message}</level>",
        level=settings.LOG_LEVEL,
        colorize=True
    )
    
    # Файловый вывод
    log_file = Path(settings.LOG_FILE)
    log_file.parent.mkdir(exist_ok=True)
    
    logger.add(
        log_file,
        format="{time:YYYY-MM-DD HH:mm:ss} | {level: <8} | {name}:{function}:{line} - {message}",
        level=settings.LOG_LEVEL,
        rotation=settings.LOG_MAX_SIZE,
        retention=settings.LOG_BACKUP_COUNT,
        compression="zip",
        enqueue=True  # Асинхронная запись
    )
    
    # Логирование ошибок в отдельный файл
    logger.add(
        "logs/error_{time:YYYY-MM-DD}.log",
        format="{time:YYYY-MM-DD HH:mm:ss} | {level: <8} | {name}:{function}:{line} - {message}",
        level="ERROR",
        rotation="1 day",
        retention=30,
        compression="zip",
        enqueue=True
    )
    
    logger.info("Логирование настроено")

# Interception стандартного logging
class InterceptHandler(logging.Handler):
    def emit(self, record):
        logger_opt = logger.opt(depth=6, exception=record.exc_info)
        logger_opt.log(record.levelname, record.getMessage())

logging.basicConfig(handlers=[InterceptHandler()], level=0)
