"""Email сервис для рассылок"""
import smtplib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from typing import List, Optional
from loguru import logger

from ..core.config import settings


class EmailService:
    """Сервис для отправки Email"""
    
    def __init__(self):
        # SMTP настройки (можно использовать Gmail, SendGrid, Mailgun и т.д.)
        self.smtp_host = getattr(settings, 'SMTP_HOST', 'smtp.gmail.com')
        self.smtp_port = getattr(settings, 'SMTP_PORT', 587)
        self.smtp_user = getattr(settings, 'SMTP_USER', '')
        self.smtp_password = getattr(settings, 'SMTP_PASSWORD', '')
        self.from_email = getattr(settings, 'FROM_EMAIL', 'noreply@ai-accountant.kz')
        self.enabled = bool(self.smtp_user and self.smtp_password)
    
    async def send_email(
        self,
        to: str,
        subject: str,
        html: str,
        text: Optional[str] = None
    ) -> bool:
        """
        Отправка Email
        
        Args:
            to: Email получателя
            subject: Тема письма
            html: HTML содержимое
            text: Текстовое содержимое (опционально)
        
        Returns:
            True если отправлено успешно
        """
        if not self.enabled:
            logger.warning("Email service not configured")
            return False
        
        try:
            msg = MIMEMultipart('alternative')
            msg['Subject'] = subject
            msg['From'] = self.from_email
            msg['To'] = to
            
            # Текстовая версия
            if text:
                part1 = MIMEText(text, 'plain', 'utf-8')
                msg.attach(part1)
            
            # HTML версия
            part2 = MIMEText(html, 'html', 'utf-8')
            msg.attach(part2)
            
            # Отправка
            with smtplib.SMTP(self.smtp_host, self.smtp_port) as server:
                server.starttls()
                server.login(self.smtp_user, self.smtp_password)
                server.sendmail(self.from_email, to, msg.as_string())
            
            logger.info(f"Email sent to {to}: {subject}")
            return True
            
        except Exception as e:
            logger.error(f"Email send error: {e}")
            return False
    
    async def send_welcome(self, to: str, name: str) -> bool:
        """Приветственное письмо"""
        subject = "🇰🇿 Добро пожаловать в AI-Бухгалтер!"
        
        html = f"""
        <html>
        <body style="font-family: Arial, sans-serif; line-height: 1.6;">
            <h1 style="color: #667eea;">🇰🇿 Добро пожаловать, {name}!</h1>
            
            <p>Спасибо за регистрацию в AI-Бухгалтер для ИП!</p>
            
            <h2>🚀 Что вы можете делать:</h2>
            <ul>
                <li>✅ Вести учёт доходов и расходов</li>
                <li>✅ Автоматически рассчитывать налог (4%)</li>
                <li>✅ Получать напоминания о сроках сдачи</li>
                <li>✅ Генерировать счета и акты</li>
                <li>✅ Распознавать чеки через AI</li>
            </ul>
            
            <a href="https://ai-accountant.kz/dashboard" 
               style="display: inline-block; padding: 12px 24px; background: #667eea; color: white; text-decoration: none; border-radius: 8px; margin: 20px 0;">
                Перейти в кабинет
            </a>
            
            <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
            
            <p style="color: #888; font-size: 14px;">
                С уважением,<br>
                Команда AI-Бухгалтер<br>
                📧 support@ai-accountant.kz
            </p>
        </body>
        </html>
        """
        
        return await self.send_email(to, subject, html)
    
    async def send_tax_reminder(self, to: str, amount: float, deadline: str) -> bool:
        """Напоминание об уплате налога"""
        subject = "🧾 Напоминание: Срок уплаты налога приближается!"
        
        html = f"""
        <html>
        <body style="font-family: Arial, sans-serif; line-height: 1.6;">
            <h1 style="color: #f59e0b;">🧾 Напоминание об уплате налога</h1>
            
            <div style="background: #fef3c7; padding: 20px; border-radius: 8px; margin: 20px 0;">
                <p style="font-size: 18px; margin: 0;">
                    💰 Сумма к уплате: <strong style="color: #92400e;">{amount:,.2f} ₸</strong>
                </p>
                <p style="font-size: 18px; margin: 10px 0 0 0;">
                    📅 Срок до: <strong style="color: #92400e;">{deadline}</strong>
                </p>
            </div>
            
            <p>Не забудьте оплатить налог вовремя, чтобы избежать штрафов!</p>
            
            <a href="https://ai-accountant.kz/tax" 
               style="display: inline-block; padding: 12px 24px; background: #f59e0b; color: white; text-decoration: none; border-radius: 8px; margin: 20px 0;">
                Оплатить налог
            </a>
            
            <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
            <p style="color: #888; font-size: 14px;">
                Это автоматическое напоминание от AI-Бухгалтер
            </p>
        </body>
        </html>
        """
        
        return await self.send_email(to, subject, html)
    
    async def send_declaration_reminder(self, to: str, deadline: str) -> bool:
        """Напоминание о сдаче декларации"""
        subject = "📋 Напоминание: Срок сдачи декларации!"
        
        html = f"""
        <html>
        <body style="font-family: Arial, sans-serif; line-height: 1.6;">
            <h1 style="color: #3b82f6;">📋 Напоминание о декларации</h1>
            
            <div style="background: #dbeafe; padding: 20px; border-radius: 8px; margin: 20px 0;">
                <p style="font-size: 18px; margin: 0;">
                    📅 Срок сдачи: <strong style="color: #1e40af;">{deadline}</strong>
                </p>
            </div>
            
            <p>Пора сдать декларацию по упрощёнке (форма 101.02)!</p>
            
            <a href="https://ai-accountant.kz/tax/declarations" 
               style="display: inline-block; padding: 12px 24px; background: #3b82f6; color: white; text-decoration: none; border-radius: 8px; margin: 20px 0;">
                Сдать декларацию
            </a>
            
            <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
            <p style="color: #888; font-size: 14px;">
                Это автоматическое напоминание от AI-Бухгалтер
            </p>
        </body>
        </html>
        """
        
        return await self.send_email(to, subject, html)
    
    async def send_invoice(self, to: str, invoice_data: dict) -> bool:
        """Отправка счёта клиенту"""
        subject = f"📄 Счёт на оплату №{invoice_data.get('number', '')}"
        
        html = f"""
        <html>
        <body style="font-family: Arial, sans-serif; line-height: 1.6;">
            <h1 style="color: #667eea;">📄 Счёт на оплату</h1>
            
            <p>Здравствуйте!</p>
            <p>Направляем вам счёт на оплату.</p>
            
            <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; margin: 20px 0;">
                <p><strong>Счёт №:</strong> {invoice_data.get('number', '')}</p>
                <p><strong>Дата:</strong> {invoice_data.get('date', '')}</p>
                <p><strong>Сумма:</strong> {invoice_data.get('total', 0):,.2f} ₸</p>
            </div>
            
            <p>Прикреплённый файл содержит счёт в формате PDF.</p>
            
            <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
            <p style="color: #888; font-size: 14px;">
                С уважением,<br>
                {invoice_data.get('seller_name', '')}
            </p>
        </body>
        </html>
        """
        
        return await self.send_email(to, subject, html)


# Глобальный экземпляр
email_service = EmailService()
