"""Сервис генерации платёжных квитанций для налогов РК"""
from datetime import datetime
from typing import List, Optional
from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import cm, inch
from reportlab.platypus import SimpleDocTemplate, Table, TableStyle, Paragraph, Spacer
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.lib.enums import TA_CENTER, TA_LEFT
import io
import os

from ..models.payment_slips import (
    PaymentSlipRequest,
    PaymentSlip,
    PaymentItem,
    PaymentSlipsResponse
)
from ..services.tax_service_kz import KZTaxService2026


class PaymentSlipsGenerator:
    """Генератор платёжных квитанций"""
    
    # Реквизиты получателя
    RECIPIENT_NAME = "Казначейство Министерства финансов РК"
    RECIPIENT_BIK = "KZKOKZKA"
    RECIPIENT_KBE = "16"
    
    # КБК платежи 2026 (по требованиям)
    KBK_NAMES = {
        "183102": "ОПВ (пенсионный взнос)",
        "183101": "СО (социальные отчисления)",
        "122101": "ОСМС/ВОСМС (медстрахование)",
        "183110": "ОПВР (пенсионный работодателя)",
        "101201": "ИПН (индивидуальный подоходный налог)",
        "101202": "ИПН (упрощенка)",
        "103101": "Социальный налог (упрощенка)"
    }
    
    def __init__(self):
        self.tax_service = KZTaxService2026()
        self._register_fonts()
    
    def _register_fonts(self):
        """Регистрация шрифтов с кириллицей"""
        try:
            # Пробуем зарегистрировать шрифт с кириллицей
            font_paths = [
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
                "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
                "C:\\Windows\\Fonts\\arial.ttf",  # Windows
                "/System/Library/Fonts/Arial.ttf",  # macOS
            ]
            
            for font_path in font_paths:
                if os.path.exists(font_path):
                    try:
                        pdfmetrics.registerFont(TTFont('CyrillicFont', font_path))
                        break
                    except:
                        continue
            else:
                # Если шрифт не найден, используем стандартный
                pass
        except Exception as e:
            pass
    
    def generate_slips(self, request: PaymentSlipRequest) -> PaymentSlipsResponse:
        """Генерация платёжек"""
        slips = []
        current_date = datetime.now().strftime("%d.%m.%Y")
        deadline = self._get_deadline(request.period)
        
        # Платежи ИП за себя
        if request.ie_payments:
            ie_contributions = self.tax_service.calculate_ie_contributions()
            
            payments = [
                PaymentItem(
                    kbk="101101",
                    name=self.KBK_NAMES["101101"],
                    amount=ie_contributions.opv,
                    period=request.period
                ),
                PaymentItem(
                    kbk="101102",
                    name=self.KBK_NAMES["101102"],
                    amount=ie_contributions.so,
                    period=request.period
                ),
                PaymentItem(
                    kbk="101103",
                    name=self.KBK_NAMES["101103"],
                    amount=ie_contributions.vosms,
                    period=request.period
                ),
                PaymentItem(
                    kbk="101108",
                    name=self.KBK_NAMES["101108"],
                    amount=ie_contributions.opvr,
                    period=request.period
                )
            ]
            
            ie_slip = PaymentSlip(
                slip_type="ie_own",
                payer_name=request.taxpayer_name,
                payer_inn=request.taxpayer_inn,
                payer_address=request.taxpayer_address,
                recipient_name=self.RECIPIENT_NAME,
                recipient_bik=self.RECIPIENT_BIK,
                recipient_kbe=self.RECIPIENT_KBE,
                account_number=request.account_number,
                bank_name=request.bank_name,
                bank_bik=request.bank_bik,
                payments=payments,
                total_amount=ie_contributions.total,
                date=current_date,
                deadline=deadline
            )
            slips.append(ie_slip)
        
        # Платежи за сотрудников
        if request.employee_payments and request.employee_count > 0:
            for i in range(request.employee_count):
                emp_calc = self.tax_service.calculate_employee_contributions(
                    type('EmployeeData', (), {'salary_net': request.employee_salary_gross * 0.78, 'has_children': False, 'has_disability': False})()
                )
                
                emp_payments = [
                    PaymentItem(
                        kbk="101101",
                        name=self.KBK_NAMES["101101"],
                        amount=emp_calc.deductions["opv"]["amount"],
                        period=request.period
                    ),
                    PaymentItem(
                        kbk="101101",
                        name="ИПН",
                        amount=emp_calc.deductions["ipn"]["amount"],
                        period=request.period
                    ),
                    PaymentItem(
                        kbk="101103",
                        name=self.KBK_NAMES["101103"],
                        amount=emp_calc.deductions["vosms"]["amount"],
                        period=request.period
                    ),
                    PaymentItem(
                        kbk="101102",
                        name=self.KBK_NAMES["101102"],
                        amount=emp_calc.employer["so"]["amount"],
                        period=request.period
                    ),
                    PaymentItem(
                        kbk="101104",
                        name=self.KBK_NAMES["101104"],
                        amount=emp_calc.employer["oosms"]["amount"],
                        period=request.period
                    ),
                    PaymentItem(
                        kbk="101108",
                        name=self.KBK_NAMES["101108"],
                        amount=emp_calc.employer["opvr"]["amount"],
                        period=request.period
                    )
                ]
                
                emp_slip = PaymentSlip(
                    slip_type=f"employee_{i+1}",
                    payer_name=request.taxpayer_name,
                    payer_inn=request.taxpayer_inn,
                    payer_address=request.taxpayer_address,
                    recipient_name=self.RECIPIENT_NAME,
                    recipient_bik=self.RECIPIENT_BIK,
                    recipient_kbe=self.RECIPIENT_KBE,
                    account_number=request.account_number,
                    bank_name=request.bank_name,
                    bank_bik=request.bank_bik,
                    payments=emp_payments,
                    total_amount=emp_calc.total_deductions + emp_calc.total_employer,
                    date=current_date,
                    deadline=deadline
                )
                slips.append(emp_slip)
        
        # Общая сумма
        total_amount = sum(slip.total_amount for slip in slips)
        
        return PaymentSlipsResponse(
            slips=slips,
            total_amount=total_amount,
            period=request.period,
            generated_at=current_date
        )
    
    def _get_deadline(self, period: str) -> str:
        """Получение срока уплаты"""
        # Парсим период (месяц/год)
        try:
            parts = period.split("/")
            if len(parts) == 2:
                month = int(parts[0])
                year = int(parts[1]) if len(parts[1]) == 4 else 2000 + int(parts[1])
                
                # Следующий месяц
                next_month = month + 1 if month < 12 else 1
                next_year = year + 1 if month == 12 else year
                
                months = [
                    "", "января", "февраля", "марта", "апреля", "мая", "июня",
                    "июля", "августа", "сентября", "октября", "ноября", "декабря"
                ]
                return f"до 25 {months[next_month]} {next_year} года"
        except:
            pass
        
        return "до 25 числа следующего месяца"
    
    def generate_pdf(self, slips_response: PaymentSlipsResponse) -> bytes:
        """Генерация PDF с платёжками"""
        buffer = io.BytesIO()
        doc = SimpleDocTemplate(
            buffer,
            pagesize=A4,
            rightMargin=1*cm,
            leftMargin=1*cm,
            topMargin=1*cm,
            bottomMargin=1*cm
        )
        
        elements = []
        
        # Стили
        try:
            font_name = 'CyrillicFont'
        except:
            font_name = 'Helvetica'
        
        styles = getSampleStyleSheet()
        title_style = ParagraphStyle(
            'CustomTitle',
            parent=styles['Heading1'],
            fontSize=16,
            alignment=TA_CENTER,
            fontName=font_name,
            spaceAfter=20
        )
        
        normal_style = ParagraphStyle(
            'CustomNormal',
            parent=styles['Normal'],
            fontSize=10,
            fontName=font_name,
            leading=12
        )
        
        # Заголовок
        elements.append(Paragraph("ПЛАТЁЖНЫЕ КВИТАНЦИИ", title_style))
        elements.append(Paragraph(f"Период: {slips_response.period}", normal_style))
        elements.append(Paragraph(f"Дата формирования: {slips_response.generated_at}", normal_style))
        elements.append(Spacer(1, 0.3*cm))
        
        # Платёжки
        for slip in slips_response.slips:
            # Заголовок платёжки
            slip_type_name = "ИП за себя" if slip.slip_type == "ie_own" else f"Сотрудник"
            elements.append(Paragraph(f"📄 {slip_type_name}", normal_style))
            elements.append(Spacer(1, 0.2*cm))
            
            # Таблица с реквизитами
            recipient_data = [
                ["Получатель:", self.RECIPIENT_NAME],
                ["БИК:", self.RECIPIENT_BIK],
                ["КБЕ:", self.RECIPIENT_KBE],
                ["Плательщик:", slip.payer_name],
                ["ИИН/БИН:", slip.payer_inn],
                ["Счёт:", slip.account_number],
                ["Банк:", slip.bank_name],
                ["БИК банка:", slip.bank_bik],
            ]
            
            recipient_table = Table(recipient_data, colWidths=[3*cm, 7*cm])
            recipient_table.setStyle(TableStyle([
                ('FONTNAME', (0, 0), (-1, -1), font_name),
                ('FONTSIZE', (0, 0), (-1, -1), 9),
                ('BOTTOMPADDING', (0, 0), (-1, -1), 4),
                ('TOPPADDING', (0, 0), (-1, -1), 4),
                ('ALIGN', (0, 0), (0, -1), 'LEFT'),
                ('ALIGN', (1, 0), (1, -1), 'LEFT'),
            ]))
            elements.append(recipient_table)
            elements.append(Spacer(1, 0.3*cm))
            
            # Таблица с платежами
            payments_data = [["КБК", "Назначение платежа", "Сумма (₸)"]]
            for payment in slip.payments:
                payments_data.append([
                    payment.kbk,
                    f"{payment.name} ({payment.period})",
                    f"{payment.amount:,.0f}"
                ])
            
            payments_data.append(["", "ИТОГО:", f"{slip.total_amount:,.0f}"])
            
            payments_table = Table(payments_data, colWidths=[2*cm, 9*cm, 3*cm])
            payments_table.setStyle(TableStyle([
                ('FONTNAME', (0, 0), (-1, -1), font_name),
                ('FONTSIZE', (0, 0), (-1, -1), 9),
                ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#667eea')),
                ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
                ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
                ('FONTNAME', (0, 0), (-1, 0), font_name),
                ('FONTSIZE', (0, 0), (-1, 0), 10),
                ('BOTTOMPADDING', (0, 0), (-1, 0), 8),
                ('TOPPADDING', (0, 0), (-1, 0), 8),
                ('GRID', (0, 0), (-1, -2), 0.5, colors.grey),
                ('LINEBELOW', (0, -2), (-1, -2), 2, colors.black),
                ('FONTNAME', (0, -1), (-1, -1), font_name),
                ('FONTSIZE', (0, -1), (-1, -1), 10),
                ('BACKGROUND', (0, -1), (-1, -1), colors.HexColor('#f0f0f0')),
            ]))
            elements.append(payments_table)
            
            # Срок уплаты
            elements.append(Spacer(1, 0.3*cm))
            elements.append(Paragraph(f"⏰ Срок уплаты: <b>{slip.deadline}</b>", normal_style))
            elements.append(Spacer(1, 0.5*cm))
            
            # Разделитель
            elements.append(Spacer(1, 0.3*cm))
            line = Paragraph("_" * 80, normal_style)
            elements.append(line)
            elements.append(Spacer(1, 0.5*cm))
        
        # Итоговая сумма
        elements.append(Spacer(1, 0.3*cm))
        total_style = ParagraphStyle(
            'TotalStyle',
            parent=styles['Heading2'],
            fontSize=14,
            alignment=TA_RIGHT,
            fontName=font_name
        )
        elements.append(Paragraph(f"💰 ОБЩАЯ СУММА: {slips_response.total_amount:,.0f} ₸", total_style))
        
        # Построение PDF
        doc.build(elements)
        
        # Получение байтов
        pdf_bytes = buffer.getvalue()
        buffer.close()
        
        return pdf_bytes


# Глобальный экземпляр
payment_slips_generator = PaymentSlipsGenerator()
