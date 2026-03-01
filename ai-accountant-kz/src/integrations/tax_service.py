"""Интеграция с Налоговой службой РК (СНИС/ИС ЭСФ)"""
import httpx
import json
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Optional, Dict, List
from loguru import logger
import xml.etree.ElementTree as ET
from pathlib import Path

from ..core.config import settings


class TaxServiceClient:
    """Клиент для интеграции с Налоговой службой РК"""
    
    def __init__(self):
        self.base_url = "https://salystyk.kgd.gov.kz"  # СНИС
        self.esf_url = "https://esf.gov.kz"  # ИС ЭСФ
        self.inn: Optional[str] = None
        self.certificate_path: Optional[str] = None
        self.enabled = False
    
    def configure(self, inn: str, certificate_path: str, certificate_password: str):
        """Настройка клиента с ЭЦП"""
        self.inn = inn
        self.certificate_path = certificate_path
        self.certificate_password = certificate_password
        self.enabled = True
        logger.info(f"TaxService configured for INN: {inn}")
    
    async def get_auth_token(self) -> Optional[str]:
        """Получение токена авторизации через ЭЦП"""
        if not self.enabled:
            return None
        
        try:
            # Запрос токена с использованием ЭЦП
            async with httpx.AsyncClient(timeout=30.0, verify=False) as client:
                # Подписание запроса ЭЦП
                signed_request = self._sign_request({
                    "inn": self.inn,
                    "timestamp": datetime.now().isoformat()
                })
                
                response = await client.post(
                    f"{self.base_url}/api/auth/token",
                    json=signed_request,
                    headers={"Content-Type": "application/json"}
                )
                
                if response.status_code == 200:
                    data = response.json()
                    logger.info("TaxService: получен токен авторизации")
                    return data.get('access_token')
                else:
                    logger.error(f"TaxService auth error: {response.status_code}")
                    return None
                    
        except Exception as e:
            logger.error(f"TaxService auth exception: {e}")
            return None
    
    def _sign_request(self, data: Dict) -> Dict:
        """Подписание запроса ЭЦП"""
        # TODO: Интеграция с НКЦ для подписания
        return {
            **data,
            "signature": "fake_signature_for_dev"
        }
    
    async def submit_declaration_101_02(
        self,
        period: str,
        income: Decimal,
        expense: Decimal,
        tax_amount: Decimal,
        declaration_xml: str
    ) -> Optional[Dict]:
        """
        Отправка декларации по упрощенке (форма 101.02)
        
        Args:
            period: Налоговый период (YYYY)
            income: Общий доход
            expense: Общие расходы
            tax_amount: Сумма налога
            declaration_xml: XML декларации
        
        Returns:
            Результат отправки
        """
        token = await self.get_auth_token()
        if not token:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=60.0, verify=False) as client:
                response = await client.post(
                    f"{self.base_url}/api/declarations/101.02",
                    headers={
                        "Authorization": f"Bearer {token}",
                        "Content-Type": "application/xml"
                    },
                    content=declaration_xml
                )
                
                if response.status_code == 200:
                    result = response.json()
                    logger.info(f"Декларация 101.02 отправлена: {result.get('declarationNumber')}")
                    return result
                else:
                    logger.error(f"Declaration submit error: {response.status_code} - {response.text}")
                    return None
                    
        except Exception as e:
            logger.error(f"Declaration submit exception: {e}")
            return None
    
    async def get_declaration_status(self, declaration_number: str) -> Optional[Dict]:
        """Проверка статуса декларации"""
        token = await self.get_auth_token()
        if not token:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0, verify=False) as client:
                response = await client.get(
                    f"{self.base_url}/api/declarations/{declaration_number}/status",
                    headers={"Authorization": f"Bearer {token}"}
                )
                
                if response.status_code == 200:
                    return response.json()
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"Declaration status exception: {e}")
            return None
    
    async def get_taxpayer_info(self, inn: str) -> Optional[Dict]:
        """Получение информации о налогоплательщике"""
        token = await self.get_auth_token()
        if not token:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0, verify=False) as client:
                response = await client.get(
                    f"{self.base_url}/api/taxpayers/{inn}",
                    headers={"Authorization": f"Bearer {token}"}
                )
                
                if response.status_code == 200:
                    return response.json()
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"Taxpayer info exception: {e}")
            return None
    
    async def check_counterparty(self, inn: str) -> Optional[Dict]:
        """Проверка контрагента (действующий ли ИП/ТОО)"""
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self.base_url}/api/business/{inn}/status",
                    headers={"Accept": "application/json"}
                )
                
                if response.status_code == 200:
                    data = response.json()
                    logger.info(f"Контрагент {inn}: {data.get('status')}")
                    return data
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"Counterparty check exception: {e}")
            return None
    
    async def get_esf_invoices(
        self,
        start_date: datetime,
        end_date: datetime,
        inn: Optional[str] = None
    ) -> Optional[List[Dict]]:
        """Получение ЭСФ (электронных счетов-фактур)"""
        token = await self.get_auth_token()
        if not token:
            return None
        
        try:
            async with httpx.AsyncClient(timeout=30.0, verify=False) as client:
                params = {
                    "startDate": start_date.strftime("%Y-%m-%d"),
                    "endDate": end_date.strftime("%Y-%m-%d")
                }
                if inn:
                    params["inn"] = inn
                
                response = await client.get(
                    f"{self.esf_url}/api/invoices",
                    params=params,
                    headers={"Authorization": f"Bearer {token}"}
                )
                
                if response.status_code == 200:
                    return response.json().get('invoices', [])
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"ESF invoices exception: {e}")
            return None


class DeclarationGenerator:
    """Генератор налоговой декларации 101.02"""
    
    def generate_xml(
        self,
        inn: str,
        period: str,
        income: Decimal,
        expense: Decimal,
        tax_amount: Decimal,
        taxpayer_name: str
    ) -> str:
        """Генерация XML декларации"""
        
        root = ET.Element("Declaration101_02")
        root.set("version", "1.0")
        root.set("date", datetime.now().strftime("%Y-%m-%d"))
        
        # Налогоплательщик
        taxpayer = ET.SubElement(root, "Taxpayer")
        ET.SubElement(taxpayer, "INN").text = inn
        ET.SubElement(taxpayer, "Name").text = taxpayer_name
        
        # Период
        ET.SubElement(root, "Period").text = period
        
        # Доходы
        income_elem = ET.SubElement(root, "Income")
        ET.SubElement(income_elem, "TotalAmount").text = str(income)
        ET.SubElement(income_elem, "TaxableAmount").text = str(income)
        
        # Расходы
        expense_elem = ET.SubElement(root, "Expense")
        ET.SubElement(expense_elem, "TotalAmount").text = str(expense)
        
        # Налог
        tax_elem = ET.SubElement(root, "Tax")
        ET.SubElement(tax_elem, "Rate").text = "0.04"
        ET.SubElement(tax_elem, "Amount").text = str(tax_amount)
        
        # Подпись
        signature = ET.SubElement(root, "Signature")
        ET.SubElement(signature, "Date").text = datetime.now().isoformat()
        
        return ET.tostring(root, encoding="unicode", xml_declaration=True)
    
    def generate_pdf(self, declaration_data: Dict) -> bytes:
        """Генерация PDF декларации"""
        # TODO: Использовать reportlab для генерации PDF
        return b"PDF content placeholder"


# Глобальный экземпляр
tax_service = TaxServiceClient()
declaration_generator = DeclarationGenerator()
