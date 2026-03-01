"""Celery задачи для фоновой обработки"""
from celery import Celery
from celery.schedules import crontab
from datetime import datetime, timedelta
from loguru import logger
import asyncio
import httpx

from .core.config import settings
from .core.database import SessionLocal
from .integrations.kaspi import kaspi_client
from .integrations.halyk import halyk_client
from .integrations.tax_service import tax_service, declaration_generator
from .utils.telegram import notifier

# ===== Celery Config =====
celery_app = Celery(
    'ai_accountant',
    broker=settings.CELERY_BROKER_URL if hasattr(settings, 'CELERY_BROKER_URL') else 'redis://localhost:6379/0',
    backend=settings.CELERY_RESULT_BACKEND if hasattr(settings, 'CELERY_RESULT_BACKEND') else 'redis://localhost:6379/0'
)

celery_app.conf.update(
    task_serializer='json',
    accept_content=['json'],
    result_serializer='json',
    timezone='Asia/Almaty',
    enable_utc=True,
    task_track_started=True,
    task_time_limit=300,
    worker_prefetch_multiplier=1,
)

# ===== Periodic Tasks =====
celery_app.conf.beat_schedule = {
    'sync-bank-transactions-every-hour': {
        'task': 'tasks.sync_all_bank_transactions',
        'schedule': crontab(minute=0),
    },
    'daily-tax-reminder': {
        'task': 'tasks.send_daily_tax_reminder',
        'schedule': crontab(hour=9, minute=0),
    },
    'monthly-declaration-reminder': {
        'task': 'tasks.send_declaration_reminder',
        'schedule': crontab(day_of_month='15,25', hour=10),
    },
    'check-tax-limit-weekly': {
        'task': 'tasks.check_tax_limit_warning',
        'schedule': crontab(day_of_week='mon', hour=8),
    },
}


def run_async(coro):
    """Helper для запуска async кода в Celery"""
    loop = asyncio.new_event_loop()
    try:
        return loop.run_until_complete(coro)
    finally:
        loop.close()


# ===== Tasks =====

@celery_app.task(bind=True, max_retries=3)
def sync_kaspi_transactions(self, user_id: str, days: int = 7):
    """Синхронизация транзакций Kaspi"""
    db = SessionLocal()
    try:
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)
        
        result = run_async(kaspi_client.get_transactions(start_date, end_date))
        if not result:
            return {"status": "error", "message": "Failed to get Kaspi transactions"}
        
        transactions = result.get('transactions', [])
        imported = 0
        
        for tx_data in transactions:
            # TODO: Сохранить в БД
            imported += 1
        
        logger.info(f"Kaspi sync: imported {imported} transactions for user {user_id}")
        return {"status": "success", "imported": imported}
        
    except Exception as e:
        logger.error(f"Kaspi sync error: {e}")
        raise self.retry(exc=e, countdown=60)
    finally:
        db.close()

@celery_app.task(bind=True, max_retries=3)
def sync_halyk_transactions(self, user_id: str, days: int = 7):
    """Синхронизация транзакций Halyk"""
    db = SessionLocal()
    try:
        accounts = run_async(halyk_client.get_accounts())
        if not accounts:
            return {"status": "error", "message": "No Halyk accounts"}
        
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)
        total_imported = 0
        
        for account in accounts:
            account_id = account.get('id')
            transactions = run_async(halyk_client.get_transactions(
                account_id, start_date, end_date
            ))
            
            if transactions:
                for tx_data in transactions:
                    total_imported += 1
        
        logger.info(f"Halyk sync: imported {total_imported} transactions for user {user_id}")
        return {"status": "success", "imported": total_imported}
        
    except Exception as e:
        logger.error(f"Halyk sync error: {e}")
        raise self.retry(exc=e, countdown=60)
    finally:
        db.close()

@celery_app.task
def sync_all_bank_transactions():
    """Синхронизация всех банков"""
    logger.info("Starting bank transactions sync...")
    # TODO: Получить всех пользователей и синхронизировать
    return {"status": "completed"}

@celery_app.task
def send_daily_tax_reminder():
    """Ежедневное напоминание о налогах"""
    db = SessionLocal()
    try:
        users = []  # TODO: Получить пользователей с неоплаченным налогом
        
        for user in users:
            if user.telegram_chat_id:
                run_async(notifier.send_message(
                    f"📊 Ежедневная сводка\n\n"
                    f"Не забудьте отслеживать доходы!\n"
                    f"Текущая ставка: 4%"
                ))
        
        logger.info("Daily tax reminders sent")
        return {"status": "success"}
    finally:
        db.close()

@celery_app.task
def send_declaration_reminder():
    """Напоминание о сдаче декларации"""
    today = datetime.now()
    
    # Проверка сроков (20 февраля и 20 августа)
    if today.month == 2 and today.day >= 15:
        deadline = "20 февраля"
    elif today.month == 8 and today.day >= 15:
        deadline = "20 августа"
    else:
        return {"status": "skipped"}
    
    db = SessionLocal()
    try:
        users = []  # TODO: Получить пользователей
        
        for user in users:
            if user.telegram_chat_id:
                run_async(notifier.send_declaration_reminder(deadline))
        
        logger.info(f"Declaration reminders sent (deadline: {deadline})")
        return {"status": "success", "deadline": deadline}
    finally:
        db.close()

@celery_app.task
def check_tax_limit_warning():
    """Проверка приближения к налоговому лимиту"""
    from .core.tax_rules import TAX_RULES
    
    db = SessionLocal()
    try:
        users = []  # TODO: Получить пользователей
        
        warnings_sent = 0
        for user in users:
            income = 0  # TODO: Запрос к БД
            
            if income > TAX_RULES.max_income * 0.8:  # 80% лимита
                if user.telegram_chat_id:
                    run_async(notifier.send_limit_warning(income, TAX_RULES.max_income))
                warnings_sent += 1
        
        logger.info(f"Tax limit warnings sent: {warnings_sent}")
        return {"status": "success", "warnings": warnings_sent}
    finally:
        db.close()

@celery_app.task(bind=True, max_retries=3)
def submit_tax_declaration(self, user_id: str, period: str):
    """Отправка налоговой декларации"""
    db = SessionLocal()
    try:
        # TODO: Получить данные пользователя
        user = None
        income = 0
        expense = 0
        tax_amount = 0
        
        # Генерация XML
        xml_content = declaration_generator.generate_xml(
            inn=user.inn if user else "",
            period=period,
            income=income,
            expense=expense,
            tax_amount=tax_amount,
            taxpayer_name=user.full_name if user else ""
        )
        
        # Отправка в налоговую
        result = run_async(tax_service.submit_declaration_101_02(
            period=period,
            income=income,
            expense=expense,
            tax_amount=tax_amount,
            declaration_xml=xml_content
        ))
        
        if result:
            logger.info(f"Declaration submitted for user {user_id}: {result.get('declarationNumber')}")
            return {"status": "success", "declaration_number": result.get('declarationNumber')}
        else:
            raise Exception("Failed to submit declaration")
            
    except Exception as e:
        logger.error(f"Declaration submit error: {e}")
        raise self.retry(exc=e, countdown=300)
    finally:
        db.close()

@celery_app.task
def classify_new_transactions():
    """AI классификация новых транзакций"""
    db = SessionLocal()
    try:
        from .ai.classifier import classifier
        
        classified = 0
        # for tx in transactions:
        #     result = await classifier.classify(...)
        #     classified += 1
        
        logger.info(f"Classified {classified} transactions")
        return {"status": "success", "classified": classified}
    finally:
        db.close()


# ===== Helper Functions =====

def init_celery():
    """Инициализация Celery"""
    logger.info("Celery initialized")
    return celery_app
