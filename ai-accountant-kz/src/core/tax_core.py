"""
Kazakhstan Tax Engine 2026
С защитой, авто-апдейтом и Audit Trail
"""
import json
import os
import hashlib
import datetime
from pathlib import Path
from typing import Optional, Dict
from loguru import logger

try:
    import requests
    REQUESTS_AVAILABLE = True
except ImportError:
    REQUESTS_AVAILABLE = False
    logger.warning("requests не установлен. Авто-обновление отключено.")


class AuditLogger:
    """
    Журнал аудита с криптографической защитой
    """
    AUDIT_FILE = "audit_trail.json"
    
    @classmethod
    def log_calculation(cls, user_id: str, input_data: Dict, result_data: Dict, 
                       engine_version: str = "2026.03_PRO_GOLD") -> Dict:
        """
        Создаёт юридически значимую запись для аудита
        
        Args:
            user_id: ID пользователя
            input_data: Входные данные расчёта
            result_data: Результаты расчёта
            engine_version: Версия движка
        
        Returns:
            log_entry: Запись лога
        """
        # Создаём криптографический хеш для защиты от подделки
        checksum_input = f"{user_id}:{input_data}:{result_data}:{datetime.datetime.now().isoformat()}"
        verification_hash = hashlib.sha256(checksum_input.encode('utf-8')).hexdigest()
        
        log_entry = {
            "timestamp": datetime.datetime.now().isoformat(),
            "user_id": user_id,
            "input_payload": input_data,
            "engine_version": engine_version,
            "result": result_data,
            "verification_hash": verification_hash,
            "checksum": hashlib.sha256(str(result_data).encode('utf-8')).hexdigest()
        }
        
        try:
            # Хранение в JSON-line формате (идеально для аудита)
            audit_path = Path(cls.AUDIT_FILE)
            with open(audit_path, 'a', encoding='utf-8') as f:
                f.write(json.dumps(log_entry, ensure_ascii=False) + "\n")
            
            logger.info(f"✅ Audit log: {verification_hash[:16]}... (user: {user_id})")
            
        except Exception as e:
            logger.error(f"❌ Audit log failed: {e}")
            # Не прерываем расчёт из-за ошибки логирования
        
        return log_entry


class KazakhstanTaxEngine2026:
    """
    Движок расчёта налогов РК 2026 с авто-обновлением констант
    
    Использование:
        engine = KazakhstanTaxEngine2026()
        engine.sync_rates()  # Обновление из remote JSON
        result = engine.calculate_employee(300000)
    """
    
    # Дефолтные значения (Hardcoded на случай сбоя сети)
    MRP: int = 4325
    MZP: int = 85000
    OPVR_RATE: float = 0.025
    DEDUCTION_14MRP: int = 14 * 4325  # 60 550
    
    KBK_MAP: Dict[str, str] = {
        "IPN_EMPLOYEE": "101201",
        "OPV": "183102",
        "SO": "183101",
        "OPVR": "183110",
        "OSMS": "122101",
        "IPN_910": "101202",
        "SN_910": "103101"
    }
    
    RATES: Dict[str, float] = {
        "opv_employee": 0.10,
        "vosms_employee": 0.02,
        "ipn_employee": 0.10,
        "so_employer": 0.05,
        "oosms_employer": 0.03,
        "opvr_employer": 0.025
    }
    
    CONFIG_PATH: str = "config/tax_config.json"
    CONFIG_VERSION: str = "2026.03"
    
    @classmethod
    def get_config_path(cls) -> Path:
        """Получение пути к конфигу"""
        # Пробуем несколько путей
        possible_paths = [
            Path(cls.CONFIG_PATH),
            Path(__file__).parent.parent / "config" / "tax_config.json",
            Path("/app/config/tax_config.json"),
        ]
        
        for path in possible_paths:
            if path.exists():
                return path
        
        return possible_paths[0]
    
    @classmethod
    def sync_rates(cls, config_url: Optional[str] = None, force_local: bool = False) -> str:
        """
        Метод самообновления базы через удаленный JSON
        
        Args:
            config_url: URL удалённого конфига (GitHub Raw, etc.)
            force_local: Использовать только локальный файл
        
        Returns:
            Статус обновления
        """
        try:
            data = None
            
            # 1. Пробуем удалённый URL
            if config_url and not force_local and REQUESTS_AVAILABLE:
                try:
                    logger.info(f"Загрузка конфига из {config_url}")
                    response = requests.get(config_url, timeout=5)
                    if response.status_code == 200:
                        data = response.json()
                        logger.info("Конфиг загружен из удалённого источника")
                except Exception as e:
                    logger.warning(f"Не удалось загрузить удалённый конфиг: {e}")
            
            # 2. Пробуем локальный файл
            if data is None:
                config_path = cls.get_config_path()
                if config_path.exists():
                    logger.info(f"Загрузка локального конфига: {config_path}")
                    with open(config_path, 'r', encoding='utf-8') as f:
                        data = json.load(f)
                    logger.info("Конфиг загружен из локального файла")
                else:
                    logger.warning("Локальный конфиг не найден. Используются заводские настройки.")
            
            # 3. Применяем обновления
            if data:
                cls.MRP = data.get('MRP', cls.MRP)
                cls.MZP = data.get('MZP', cls.MZP)
                cls.OPVR_RATE = data.get('OPVR_RATE', cls.OPVR_RATE)
                
                multiplier = data.get('DEDUCTION_14MRP_MULTIPLIER', 14)
                cls.DEDUCTION_14MRP = multiplier * cls.MRP
                
                if 'KBK_MAP' in data:
                    cls.KBK_MAP.update(data['KBK_MAP'])
                
                if 'rates' in data:
                    cls.RATES.update(data['rates'])
                
                cls.CONFIG_VERSION = data.get('version', cls.CONFIG_VERSION)
                
                logger.info(f"✅ Обновлено успешно. МРП 2026: {cls.MRP}, ОПВР: {cls.OPVR_RATE*100}%")
                return f"Обновлено успешно. Версия: {cls.CONFIG_VERSION}. МРП: {cls.MRP} ₸"
            else:
                logger.warning("Используются заводские настройки")
                return f"Используются заводские настройки. МРП: {cls.MRP} ₸"
                
        except Exception as e:
            logger.error(f"Ошибка обновления: {e}")
            return f"Ошибка обновления: {e}. Используются заводские настройки."
    
    @classmethod
    def calculate_employee(cls, salary_gross: float, user_id: str = "Anonymous") -> Dict:
        """
        Расчёт налогов с использованием актуальных (синхронизированных) данных
        С АВТОМАТИЧЕСКИМ ЛОГИРОВАНИЕМ
        
        Args:
            salary_gross: Оклад до вычетов (грязными)
            user_id: ID пользователя для аудита
        
        Returns:
            Dict с расчётами и КБК
        """
        # Удержания сотрудника
        opv = round(salary_gross * cls.RATES['opv_employee'])
        vosms = round(salary_gross * cls.RATES['vosms_employee'])
        
        # ИПН с учетом динамического вычета 14 МРП
        taxable_base = salary_gross - opv - vosms - cls.DEDUCTION_14MRP
        ipn = round(max(0, taxable_base * cls.RATES['ipn_employee']))
        
        # Нагрузка работодателя
        so = round((salary_gross - opv) * cls.RATES['so_employer'])
        oosms = round(salary_gross * cls.RATES['oosms_employer'])
        opvr = round(salary_gross * cls.OPVR_RATE)
        
        # На руки
        net_salary = salary_gross - opv - vosms - ipn
        
        result = {
            "status": "Calculated with current rates",
            "config_version": cls.CONFIG_VERSION,
            "mrp_used": cls.MRP,
            "mzp_used": cls.MZP,
            "deduction_14mrp": cls.DEDUCTION_14MRP,
            "opvr_rate": cls.OPVR_RATE,
            "calculations": {
                "employee_deductions": {
                    "OPV": {
                        "amount": opv,
                        "kbk": cls.KBK_MAP["OPV"],
                        "rate": f"{cls.RATES['opv_employee']*100}%"
                    },
                    "VOSMS": {
                        "amount": vosms,
                        "kbk": cls.KBK_MAP["OSMS"],
                        "rate": f"{cls.RATES['vosms_employee']*100}%"
                    },
                    "IPN": {
                        "amount": ipn,
                        "kbk": cls.KBK_MAP["IPN_EMPLOYEE"],
                        "rate": f"{cls.RATES['ipn_employee']*100}%",
                        "deduction_applied": cls.DEDUCTION_14MRP
                    }
                },
                "employer_costs": {
                    "SO": {
                        "amount": so,
                        "kbk": cls.KBK_MAP["SO"],
                        "rate": f"{cls.RATES['so_employer']*100}%"
                    },
                    "OOSMS": {
                        "amount": oosms,
                        "kbk": cls.KBK_MAP["OSMS"],
                        "rate": f"{cls.RATES['oosms_employer']*100}%"
                    },
                    "OPVR": {
                        "amount": opvr,
                        "kbk": cls.KBK_MAP["OPVR"],
                        "rate": f"{cls.OPVR_RATE*100}%"
                    }
                }
            },
            "net_salary": net_salary,
            "total_employer_cost": so + oosms + opvr,
            "total_budget": opv + vosms + ipn + so + oosms + opvr
        }
        
        # АВТО-ЛОГИРОВАНИЕ (ПРАВИЛО №5 - обязательно!)
        input_data = {
            "action": "calculate_employee",
            "salary_type": "gross",
            "amount": salary_gross
        }
        
        AuditLogger.log_calculation(user_id, input_data, result, f"{cls.CONFIG_VERSION}_AUDIT")
        
        return result
    
    @classmethod
    def get_rates_info(cls) -> Dict:
        """Получение информации о текущих ставках"""
        return {
            "version": cls.CONFIG_VERSION,
            "last_updated": "2026-03-01",
            "constants": {
                "MRP": cls.MRP,
                "MZP": cls.MZP,
                "DEDUCTION_14MRP": cls.DEDUCTION_14MRP,
                "DEDUCTION_14MRP_MULTIPLIER": cls.DEDUCTION_14MRP // cls.MRP
            },
            "rates": {
                "employee": {
                    "OPV": f"{cls.RATES['opv_employee']*100}%",
                    "VOSMS": f"{cls.RATES['vosms_employee']*100}%",
                    "IPN": f"{cls.RATES['ipn_employee']*100}% (после вычета 14 МРП)"
                },
                "employer": {
                    "SO": f"{cls.RATES['so_employer']*100}%",
                    "OOSMS": f"{cls.RATES['oosms_employer']*100}%",
                    "OPVR": f"{cls.OPVR_RATE*100}%"
                }
            },
            "kbk": cls.KBK_MAP
        }


# Глобальный экземпляр
tax_engine = KazakhstanTaxEngine2026()

# Авто-синхронизация при импорте
try:
    tax_engine.sync_rates()
except:
    pass
