"""Генерация PDF документов (счета, акты, накладные)"""
from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import mm, cm
from reportlab.platypus import SimpleDocTemplate, Table, TableStyle, Paragraph, Spacer, Image
from reportlab.lib.enums import TA_CENTER, TA_LEFT, TA_RIGHT
from reportlab.pdfgen import canvas
from datetime import datetime
from decimal import Decimal
from typing import List, Dict, Optional
from io import BytesIO
import os


class DocumentGenerator:
    """Генератор PDF документов"""
    
    def __init__(self):
        self.styles = getSampleStyleSheet()
        self._setup_styles()
    
    def _setup_styles(self):
        """Настройка стилей"""
        self.styles.add(ParagraphStyle(
            name='Header1',
            parent=self.styles['Heading1'],
            fontSize=16,
            spaceAfter=10
        ))
        
        self.styles.add(ParagraphStyle(
            name='Header2',
            parent=self.styles['Heading2'],
            fontSize=12,
            spaceAfter=6
        ))
        
        self.styles.add(ParagraphStyle(
            name='NormalRU',
            parent=self.styles['Normal'],
            fontSize=10,
            leading=12
        ))
    
    def generate_invoice(self, data: Dict) -> bytes:
        """
        Генерация счёта на оплату
        
        Args:
            data: {
                "invoice_number": "123",
                "invoice_date": "28.02.2026",
                "seller": {"name": "ИП Иванов И.И.", "inn": "123456789012", "bank": "Kaspi", "bik": "123456789", "account": "1234567890123456"},
                "buyer": {"name": "ТОО «Ромашка»", "inn": "987654321098"},
                "items": [{"name": "Услуга 1", "quantity": 1, "unit": "шт", "price": 100000, "total": 100000}],
                "total": 100000,
                "vat": "Без НДС",
                "signature": "Иванов И.И."
            }
        """
        buffer = BytesIO()
        doc = SimpleDocTemplate(
            buffer,
            pagesize=A4,
            rightMargin=15*mm,
            leftMargin=15*mm,
            topMargin=15*mm,
            bottomMargin=15*mm
        )
        
        elements = []
        
        # Заголовок
        elements.append(Paragraph("СЧЁТ НА ОПЛАТУ №" + str(data.get('invoice_number', '')), self.styles['Header1']))
        elements.append(Paragraph(f"от {data.get('invoice_date', '')}", self.styles['NormalRU']))
        elements.append(Spacer(1, 15))
        
        # Таблица с продавцом и покупателем
        seller_data = data.get('seller', {})
        buyer_data = data.get('buyer', {})
        
        seller_table = Table([
            ['Продавец:', seller_data.get('name', '')],
            ['ИНН:', seller_data.get('inn', '')],
            ['Банк:', seller_data.get('bank', '')],
            ['БИК:', seller_data.get('bik', '')],
            ['Счёт:', seller_data.get('account', '')],
        ], colWidths=[4*cm, 12*cm])
        
        seller_table.setStyle(TableStyle([
            ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
            ('FONTNAME', (0, 0), (0, -1), 'Helvetica-Bold'),
            ('FONTSIZE', (0, 0), (-1, -1), 9),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 3),
        ]))
        elements.append(seller_table)
        elements.append(Spacer(1, 10))
        
        buyer_table = Table([
            ['Покупатель:', buyer_data.get('name', '')],
            ['ИНН:', buyer_data.get('inn', '')],
        ], colWidths=[4*cm, 12*cm])
        
        buyer_table.setStyle(TableStyle([
            ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
            ('FONTNAME', (0, 0), (0, -1), 'Helvetica-Bold'),
            ('FONTSIZE', (0, 0), (-1, -1), 9),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 3),
        ]))
        elements.append(buyer_table)
        elements.append(Spacer(1, 20))
        
        # Таблица товаров/услуг
        items = data.get('items', [])
        table_data = [['№', 'Наименование', 'Ед.', 'Кол-во', 'Цена', 'Сумма']]
        
        for i, item in enumerate(items, 1):
            table_data.append([
                str(i),
                item.get('name', ''),
                item.get('unit', 'шт'),
                str(item.get('quantity', 1)),
                f"{item.get('price', 0):,.2f} ₸",
                f"{item.get('total', 0):,.2f} ₸"
            ])
        
        items_table = Table(table_data, colWidths=[1*cm, 6*cm, 2*cm, 2*cm, 3*cm, 3*cm])
        items_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#667eea')),
            ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
            ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
            ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
            ('FONTSIZE', (0, 0), (-1, -1), 9),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
            ('TOPPADDING', (0, 0), (-1, -1), 6),
            ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
            ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, colors.HexColor('#f8f9fa')]),
        ]))
        elements.append(items_table)
        elements.append(Spacer(1, 15))
        
        # Итого
        total = data.get('total', 0)
        vat = data.get('vat', 'Без НДС')
        
        total_table = Table([
            ['Итого:', f"{total:,.2f} ₸"],
            ['В том числе НДС:', vat],
            ['Всего к оплате:', f"{total:,.2f} ₸"],
        ], colWidths=[12*cm, 5*cm])
        
        total_table.setStyle(TableStyle([
            ('ALIGN', (0, 0), (-1, -1), 'RIGHT'),
            ('FONTNAME', (0, -1), (-1, -1), 'Helvetica-Bold'),
            ('FONTSIZE', (0, 0), (-1, -1), 10),
            ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
        ]))
        elements.append(total_table)
        elements.append(Spacer(1, 30))
        
        # Подписи
        signature = data.get('signature', '')
        elements.append(Paragraph("Продавец: _________________ /" + signature + "/", self.styles['NormalRU']))
        elements.append(Spacer(1, 10))
        elements.append(Paragraph("Покупатель: _________________ /_________________/", self.styles['NormalRU']))
        
        # Примечание
        elements.append(Spacer(1, 20))
        elements.append(Paragraph("Оплата в течение 3 банковских дней", self.styles['NormalRU']))
        
        # Build PDF
        doc.build(elements)
        
        pdf_bytes = buffer.getvalue()
        buffer.close()
        return pdf_bytes
    
    def generate_act(self, data: Dict) -> bytes:
        """
        Генерация акта выполненных работ
        
        Args:
            data: {
                "act_number": "123",
                "act_date": "28.02.2026",
                "period": "Февраль 2026",
                "seller": {"name": "ИП Иванов И.И.", "inn": "123456789012"},
                "buyer": {"name": "ТОО «Ромашка»", "inn": "987654321098"},
                "services": [{"name": "Услуга 1", "amount": 100000}],
                "total": 100000,
                "signature": "Иванов И.И."
            }
        """
        buffer = BytesIO()
        doc = SimpleDocTemplate(buffer, pagesize=A4, rightMargin=15*mm, leftMargin=15*mm, topMargin=15*mm, bottomMargin=15*mm)
        
        elements = []
        
        # Заголовок
        elements.append(Paragraph("АКТ ВЫПОЛНЕННЫХ РАБОТ №" + str(data.get('act_number', '')), self.styles['Header1']))
        elements.append(Paragraph(f"от {data.get('act_date', '')} за период {data.get('period', '')}", self.styles['NormalRU']))
        elements.append(Spacer(1, 15))
        
        # Стороны
        seller = data.get('seller', {})
        buyer = data.get('buyer', {})
        
        elements.append(Paragraph(f"<b>Исполнитель:</b> {seller.get('name', '')}, ИНН {seller.get('inn', '')}", self.styles['NormalRU']))
        elements.append(Paragraph(f"<b>Заказчик:</b> {buyer.get('name', '')}, ИНН {buyer.get('inn', '')}", self.styles['NormalRU']))
        elements.append(Spacer(1, 15))
        
        # Таблица услуг
        services = data.get('services', [])
        table_data = [['№', 'Наименование услуги', 'Сумма']]
        
        for i, service in enumerate(services, 1):
            table_data.append([
                str(i),
                service.get('name', ''),
                f"{service.get('amount', 0):,.2f} ₸"
            ])
        
        services_table = Table(table_data, colWidths=[1*cm, 12*cm, 4*cm])
        services_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#667eea')),
            ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
            ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
            ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
            ('FONTSIZE', (0, 0), (-1, -1), 9),
            ('GRID', (0, 0), (-1, -1), 0.5, colors.grey),
            ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.white, colors.HexColor('#f8f9fa')]),
        ]))
        elements.append(services_table)
        elements.append(Spacer(1, 15))
        
        # Итого
        total = data.get('total', 0)
        elements.append(Paragraph(f"<b>Итого:</b> {total:,.2f} ₸", self.styles['NormalRU']))
        elements.append(Spacer(1, 20))
        
        # Подписи
        elements.append(Paragraph("Работы выполнены в полном объеме. Претензий нет.", self.styles['NormalRU']))
        elements.append(Spacer(1, 20))
        elements.append(Paragraph("Исполнитель: _________________ /" + data.get('signature', '') + "/", self.styles['NormalRU']))
        elements.append(Spacer(1, 10))
        elements.append(Paragraph("Заказчик: _________________ /_________________/", self.styles['NormalRU']))
        
        doc.build(elements)
        
        pdf_bytes = buffer.getvalue()
        buffer.close()
        return pdf_bytes
    
    def generate_waybill(self, data: Dict) -> bytes:
        """Генерация товарной накладной"""
        # Аналогично счёту и акту
        return self.generate_invoice(data)


# Глобальный экземпляр
doc_generator = DocumentGenerator()
