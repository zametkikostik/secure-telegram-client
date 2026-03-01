"""1С интеграция для обмена данными"""
import xml.etree.ElementTree as ET
from datetime import datetime
from decimal import Decimal
from typing import List, Dict, Optional
from io import BytesIO
import json
from loguru import logger


class _1CExporter:
    """Экспорт данных в 1С:Бухгалтерия"""
    
    def __init__(self):
        self.encoding = "UTF-8"
    
    def export_transactions_to_xml(self, transactions: List[Dict]) -> str:
        """
        Экспорт транзакций в XML формат 1С
        
        Args:
            transactions: Список транзакций
        
        Returns:
            XML строка
        """
        root = ET.Element("БухгалтерскиеДанные")
        root.set("ВерсияФормата", "2.1")
        root.set("ДатаВыгрузки", datetime.now().isoformat())
        
        # Транзакции
        transactions_elem = ET.SubElement(root, "Транзакции")
        
        for tx in transactions:
            tx_elem = ET.SubElement(transactions_elem, "Транзакция")
            
            ET.SubElement(tx_elem, "Ид").text = tx.get('id', '')
            ET.SubElement(tx_elem, "Дата").text = tx.get('date', '')
            ET.SubElement(tx_elem, "Сумма").text = str(tx.get('amount', 0))
            ET.SubElement(tx_elem, "Валюта").text = tx.get('currency', 'KZT')
            
            # Тип операции
            tx_type = "Доход" if tx.get('type') == 'income' else "Расход"
            ET.SubElement(tx_elem, "ТипОперации").text = tx_type
            
            # Контрагент
            if tx.get('counterparty'):
                counterparty_elem = ET.SubElement(tx_elem, "Контрагент")
                ET.SubElement(counterparty_elem, "Наименование").text = tx.get('counterparty')
                if tx.get('counterparty_inn'):
                    ET.SubElement(counterparty_elem, "ИНН").text = tx.get('counterparty_inn')
            
            # Статья
            if tx.get('ai_category') or tx.get('manual_category'):
                ET.SubElement(tx_elem, "Статья").text = tx.get('ai_category') or tx.get('manual_category')
            
            # Описание
            ET.SubElement(tx_elem, "Содержание").text = tx.get('description', '')
            
            # Документ основание
            if tx.get('document_number'):
                doc_elem = ET.SubElement(tx_elem, "ДокументОснование")
                ET.SubElement(doc_elem, "Номер").text = tx.get('document_number')
                ET.SubElement(doc_elem, "Дата").text = tx.get('document_date', tx.get('date', ''))
        
        # Генерация XML
        xml_str = ET.tostring(root, encoding=self.encoding, xml_declaration=True)
        return xml_str.decode(self.encoding)
    
    def export_to_1c_exchange_format(self, data: Dict) -> str:
        """
        Экспорт в формат обмена 1С (JSON)
        
        Args:
            data: Данные для экспорта
        
        Returns:
            JSON строка
        """
        exchange_data = {
            "$schema": "1C_Exchange_v2.1",
            "export_date": datetime.now().isoformat(),
            "company": {
                "name": data.get('company_name', ''),
                "inn": data.get('company_inn', ''),
                "kpp": data.get('company_kpp', '')
            },
            "period": {
                "start": data.get('period_start'),
                "end": data.get('period_end')
            },
            "transactions": [],
            "employees": [],
            "counterparties": []
        }
        
        # Транзакции
        for tx in data.get('transactions', []):
            exchange_data['transactions'].append({
                "id": tx.get('id'),
                "date": tx.get('date'),
                "amount": float(tx.get('amount', 0)),
                "currency": tx.get('currency', 'KZT'),
                "type": tx.get('type'),
                "counterparty": tx.get('counterparty'),
                "inn": tx.get('counterparty_inn'),
                "category": tx.get('category'),
                "description": tx.get('description')
            })
        
        # Сотрудники
        for emp in data.get('employees', []):
            exchange_data['employees'].append({
                "id": emp.get('id'),
                "full_name": emp.get('full_name'),
                "inn": emp.get('inn'),
                "position": emp.get('position'),
                "salary": float(emp.get('salary', 0))
            })
        
        return json.dumps(exchange_data, ensure_ascii=False, indent=2)
    
    def generate_1c_import_script(self, file_path: str) -> str:
        """
        Генерация скрипта для импорта в 1С
        
        Args:
            file_path: Путь к файлу с данными
        
        Returns:
            1C скрипт (bsl)
        """
        script = f"""
// Скрипт импорта данных в 1С
// Сгенерирован AI-Бухгалтер {datetime.now().strftime('%d.%m.%Y')}

Процедура ИмпортДанныхИзФайла()
    
    // Путь к файлу обмена
    ПутьКФайлу = "{file_path}";
    
    // Чтение данных
    Если Не ФайлСуществует(ПутьКФайлу) Тогда
        Сообщить("Файл не найден: " + ПутьКФайлу);
        Возврат;
    КонецЕсли;
    
    // Чтение JSON
    ЧтениеJSON = Новый ЧтениеJSON;
    ЧтениеJSON.ОткрытьФайл(ПутьКФайлу);
    
    Данные = ПрочитатьJSON(ЧтениеJSON);
    
    // Импорт транзакций
    Для Каждого Транзакция Из Данные.Транзакции Цикл
        Документ = Документы.ПоступлениеДенежныхСредств.СоздатьДокумент();
        Документ.Дата = Транзакция.Дата;
        Документ.Сумма = Транзакция.Сумма;
        Документ.Контрагент = Справочники.Контрагенты.НайтиПоИНН(Транзакция.ИНН);
        Документ.СтатьяДвиженияДенежныхСредств = 
            Справочники.СтатьиДвиженияДенежныхСредств.НайтиПоНаименованию(Транзакция.Категория);
        Документ.Записать();
    КонецЦикла;
    
    // Импорт сотрудников
    Для Каждого Сотрудник Из Данные.Сотрудники Цикл
        СотрудникОбъект = Справочники.Сотрудники.СоздатьЭлемент();
        СотрудникОбъект.ФИО = Сотрудник.ФИО;
        СотрудникОбъект.ИНН = Сотрудник.ИНН;
        СотрудникОбъект.Должность = Сотрудник.Должность;
        СотрудникОбъект.Оклад = Сотрудник.Оклад;
        СотрудникОбъект.Записать();
    КонецЦикла;
    
    Сообщить("Импорт завершён успешно!");
    
КонецПроцедуры
"""
        return script


class _1CImporter:
    """Импорт данных из 1С"""
    
    def parse_1c_export(self, xml_content: str) -> Dict:
        """
        Парсинг экспорта из 1С
        
        Args:
            xml_content: XML строка из 1С
        
        Returns:
            Dict с данными
        """
        try:
            root = ET.fromstring(xml_content)
            
            data = {
                "transactions": [],
                "counterparties": [],
                "employees": []
            }
            
            # Парсинг транзакций
            for tx_elem in root.findall(".//Транзакция"):
                tx = {
                    "id": self._get_text(tx_elem, "Ид"),
                    "date": self._get_text(tx_elem, "Дата"),
                    "amount": float(self._get_text(tx_elem, "Сумма", "0")),
                    "currency": self._get_text(tx_elem, "Валюта", "KZT"),
                    "type": "expense" if self._get_text(tx_elem, "ТипОперации") == "Расход" else "income",
                    "counterparty": self._get_text(tx_elem, "Контрагент/Наименование"),
                    "counterparty_inn": self._get_text(tx_elem, "Контрагент/ИНН"),
                    "category": self._get_text(tx_elem, "Статья"),
                    "description": self._get_text(tx_elem, "Содержание")
                }
                data["transactions"].append(tx)
            
            logger.info(f"Parsed {len(data['transactions'])} transactions from 1C export")
            return data
            
        except ET.ParseError as e:
            logger.error(f"XML parse error: {e}")
            return {}
    
    def _get_text(self, elem: ET.Element, path: str, default: str = "") -> str:
        """Получение текста из XML элемента"""
        child = elem.find(path)
        return child.text if child is not None and child.text else default


# Глобальные экземпляры
exporter = _1CExporter()
importer = _1CImporter()
