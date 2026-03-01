"""API endpoints для Telegram бота"""
from fastapi import APIRouter, Depends, HTTPException, Request
from sqlalchemy.orm import Session
from pydantic import BaseModel
from typing import Optional

from ..core.database import get_db, User
from ..core.security import get_current_user
from ..utils.telegram_bot import bot

router = APIRouter(prefix="/api/v1/telegram", tags=["telegram"])

class TelegramLinkRequest(BaseModel):
    chat_id: str
    verification_code: Optional[str] = None

@router.post("/link")
async def link_telegram(
    request: TelegramLinkRequest,
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Привязка Telegram аккаунта"""
    # TODO: Сохранить chat_id в БД
    current_user.telegram_chat_id = request.chat_id
    db.commit()
    
    # Отправить приветственное сообщение
    if bot.enabled:
        await bot.send_notification(
            request.chat_id,
            "✅ <b>Telegram подключён!</b>\n\n"
            "Теперь вы будете получать уведомления о:\n"
            "• Новых транзакциях\n"
            "• Напоминаниях о налогах\n"
            "• Сроках сдачи декларации\n\n"
            "Команды:\n"
            "/start - Главное меню\n"
            "/help - Помощь\n"
            "/balance - Сводка"
        )
    
    return {"status": "success", "message": "Telegram подключён"}

@router.post("/unlink")
async def unlink_telegram(
    current_user: User = Depends(get_current_user),
    db: Session = Depends(get_db)
):
    """Отвязка Telegram аккаунта"""
    chat_id = current_user.telegram_chat_id
    current_user.telegram_chat_id = None
    db.commit()
    
    if bot.enabled and chat_id:
        await bot.send_notification(
            chat_id,
            "❌ Telegram отключён от аккаунта"
        )
    
    return {"status": "success", "message": "Telegram отключён"}

@router.get("/status")
async def get_telegram_status(
    current_user: User = Depends(get_current_user)
):
    """Статус подключения Telegram"""
    return {
        "linked": current_user.telegram_chat_id is not None,
        "chat_id": current_user.telegram_chat_id,
        "bot_enabled": bot.enabled
    }

@router.post("/webhook")
async def telegram_webhook(request: Request):
    """Webhook для получения обновлений от Telegram"""
    if not bot.enabled:
        return {"status": "disabled"}
    
    try:
        update = await request.json()
        await bot.dp.feed_webhook_update(request, update, bot.bot)
        return {"status": "ok"}
    except Exception as e:
        return {"status": "error", "message": str(e)}

@router.post("/test-notification")
async def send_test_notification(
    current_user: User = Depends(get_current_user)
):
    """Тестовое уведомление"""
    if not current_user.telegram_chat_id:
        raise HTTPException(status_code=400, detail="Telegram не подключён")
    
    success = await bot.send_notification(
        current_user.telegram_chat_id,
        "🔔 <b>Тестовое уведомление</b>\n\n"
        "Если вы видите это сообщение - уведомления работают! ✅"
    )
    
    if success:
        return {"status": "success", "message": "Уведомление отправлено"}
    else:
        raise HTTPException(status_code=500, detail="Не удалось отправить")
