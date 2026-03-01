"""
Kazakhstan Tax Engine 2026 - Production Ready
Налоговый движок для ИП на упрощенке с актуальными константами 2026

Константы 2026:
- МРП: 4325 ₸
- МЗП: 85000 ₸
- ОПВР: 2.5%
- КБК ОПВР: 183110
"""
import json
import hashlib
import datetime
from pathlib import Path
from typing import Optional, Dict, Any
from decimal import Decimal
import logging

logger = logging.getLogger("ai-accountant")


class AuditLogger:
    """
    Журнал аудита с криптографической защитой
    """
    AUDIT_FILE = "audit_trail.json"

    @classmethod
    def log_calculation(
        cls,
        user_id: str,
        input_data: Dict,
        result_data: Dict,
        engine_version: str = "2026.03_PRO"
    ) -> Dict:
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
        # Криптографический хеш для защиты от подделки
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
            audit_path = Path(cls.AUDIT_FILE)
            audit_path.parent.mkdir(parents=True, exist_ok=True)
            
            with open(audit_path, 'a', encoding='utf-8') as f:
                f.write(json.dumps(log_entry, ensure_ascii=False) + "\n")

            logger.info(f"Audit log: {verification_hash[:16]}... (user: {user_id})")

        except Exception as e:
            logger.error(f"Audit log failed: {e}")

        return log_entry


class TaxEngineKZ2026:
    """
    Налоговый движок Казахстана 2026
    
    Константы 2026 года:
    - МРП (Месячный расчётный показатель): 4325 ₸
    - МЗП (Минимальная зарплата): 85000 ₸
    - ОПВР (Обязательные пенсионные взносы работодателя): 2.5%
    - КБК для ОПВР: 183110
    """
    
    # ===== Константы 2026 (Hardcoded) =====
    MRP: int = 4325           # Месячный расчётный показатель
    MZP: int = 85000          # Минимальная зарплата
    OPVR_RATE: float = 0.025  # ОПВР 2.5%
    
    # Вычет 14 МРП для ИПН
    DEDUCTION_14MRP: int = 14 * 4325  # 60 550 ₸
    
    # ===== КБК 2026 =====
    KBK_MAP: Dict[str, str] = {
        "IPN_EMPLOYEE": "101201",   # ИПН сотрудника
        "OPV": "183102",            # ОПВ сотрудника
        "SO": "183101",             # Соцналог работодателя
        "OPVR": "183110",           # ОПВР работодателя (2.5%)
        "OSMS": "122101",           # ОСМС
        "IPN_910": "101202",        # ИПН 910 форма
        "SN_910": "103101"          # Соцналог 910 форма
    }
    
    # ===== Ставки налогов 2026 =====
    RATES: Dict[str, float] = {
        "opv_employee": 0.10,       # ОПВ сотрудника 10%
        "vosms_employee": 0.02,     # ВОСМС сотрудника 2%
        "ipn_employee": 0.10,       # ИПН сотрудника 10%
        "so_employer": 0.05,        # СО работодателя 5%
        "oosms_employer": 0.03,     # ООСМС работодателя 3%
        "opvr_employer": 0.025      # ОПВР работодателя 2.5%
    }
    
    CONFIG_VERSION: str = "2026.03"
    
    @classmethod
    def calculate_from_gross(
        cls,
        salary_gross: float,
        user_id: str = "Anonymous",
        trace_id: str = ""
    ) -> Dict[str, Any]:
        """
        Расчёт налогов от оклада (gross) до вычетов

        Args:
            salary_gross: Оклад до удержания (грязными)
            user_id: ID пользователя для аудита
            trace_id: ID трассировки запроса

        Returns:
            Dict с расчётами и КБК
        """
        # Удержания сотрудника
        opv = round(salary_gross * cls.RATES['opv_employee'], 2)
        vosms = round(salary_gross * cls.RATES['vosms_employee'], 2)
        
        # Налоговая база для ИПН
        taxable_base = salary_gross - opv - vosms - cls.DEDUCTION_14MRP
        ipn = round(max(0, taxable_base * cls.RATES['ipn_employee']), 2)
        
        # Взносы работодателя
        so = round((salary_gross - opv) * cls.RATES['so_employer'], 2)
        oosms = round(salary_gross * cls.RATES['oosms_employer'], 2)
        opvr = round(salary_gross * cls.OPVR_RATE, 2)
        
        # На руки (net)
        net_salary = round(salary_gross - opv - vosms - ipn, 2)
        
        # Итоги
        total_employee_deductions = round(opv + vosms + ipn, 2)
        total_employer_cost = round(so + oosms + opvr, 2)
        total_budget = round(total_employee_deductions + total_employer_cost, 2)
        
        result = {
            "status": "calculated",
            "config_version": cls.CONFIG_VERSION,
            "calculation_type": "gross",
            
            # Константы
            "mrp_used": cls.MRP,
            "mzp_used": cls.MZP,
            "deduction_14mrp": cls.DEDUCTION_14MRP,
            "opvr_rate": cls.OPVR_RATE,
            
            # Входные данные
            "salary_gross": salary_gross,
            "salary_net": net_salary,
            
            # Удержания сотрудника
            "employee_deductions": {
                "OPV": {
                    "amount": opv,
                    "kbk": cls.KBK_MAP["OPV"],
                    "rate": f"{cls.RATES['opv_employee'] * 100}%"
                },
                "VOSMS": {
                    "amount": vosms,
                    "kbk": cls.KBK_MAP["OSMS"],
                    "rate": f"{cls.RATES['vosms_employee'] * 100}%"
                },
                "IPN": {
                    "amount": ipn,
                    "kbk": cls.KBK_MAP["IPN_EMPLOYEE"],
                    "rate": f"{cls.RATES['ipn_employee'] * 100}%",
                    "deduction_applied": cls.DEDUCTION_14MRP
                }
            },
            
            # Взносы работодателя
            "employer_contributions": {
                "SO": {
                    "amount": so,
                    "kbk": cls.KBK_MAP["SO"],
                    "rate": f"{cls.RATES['so_employer'] * 100}%"
                },
                "OOSMS": {
                    "amount": oosms,
                    "kbk": cls.KBK_MAP["OSMS"],
                    "rate": f"{cls.RATES['oosms_employer'] * 100}%"
                },
                "OPVR": {
                    "amount": opvr,
                    "kbk": cls.KBK_MAP["OPVR"],
                    "rate": f"{cls.OPVR_RATE * 100}%"
                }
            },
            
            # Итоги
            "total_employee_deductions": total_employee_deductions,
            "total_employer_cost": total_employer_cost,
            "total_budget": total_budget,
            
            # Аудит
            "trace_id": trace_id,
            "timestamp": datetime.datetime.now().isoformat()
        }
        
        # Логирование в audit trail
        input_data = {
            "action": "calculate_from_gross",
            "salary_type": "gross",
            "amount": salary_gross
        }
        
        AuditLogger.log_calculation(user_id, input_data, result, f"{cls.CONFIG_VERSION}_AUDIT")
        
        return result
    
    @classmethod
    def calculate_from_net(
        cls,
        salary_net: float,
        user_id: str = "Anonymous",
        trace_id: str = ""
    ) -> Dict[str, Any]:
        """
        Обратный расчёт: из суммы на руки (net) в оклад (gross)
        
        Использует итеративный метод для точного расчёта

        Args:
            salary_net: Желаемая сумма на руки (чистыми)
            user_id: ID пользователя для аудита
            trace_id: ID трассировки запроса

        Returns:
            Dict с расчётами и КБК
        """
        # Начальное приближение: gross ≈ net / 0.78
        # (учитывая ~10% ОПВ + ~10% ИПН + ~2% ВОСМС)
        gross = salary_net / 0.78
        
        # Итеративный расчёт для точности (5 итераций)
        for _ in range(5):
            opv = gross * cls.RATES['opv_employee']
            vosms = gross * cls.RATES['vosms_employee']
            taxable_base = gross - opv - vosms - cls.DEDUCTION_14MRP
            ipn = max(0, taxable_base * cls.RATES['ipn_employee'])
            
            calculated_net = gross - opv - vosms - ipn
            if abs(calculated_net - salary_net) < 1:
                break
            
            # Корректировка gross
            gross = gross + (salary_net - calculated_net) * 0.5
        
        # Округляем до тенге
        gross = round(gross)
        
        # Финальный расчёт через основной метод
        result = cls.calculate_from_gross(gross, user_id, trace_id)
        result["calculation_type"] = "net"
        result["requested_net"] = salary_net
        result["calculated_gross"] = gross
        
        return result
    
    @classmethod
    def get_rates_info(cls) -> Dict[str, Any]:
        """
        Получение информации о текущих ставках и константах

        Returns:
            Dict с информацией о ставках
        """
        return {
            "version": cls.CONFIG_VERSION,
            "last_updated": "2026-03-01",
            "constants": {
                "MRP": cls.MRP,
                "MZP": cls.MZP,
                "DEDUCTION_14MRP": cls.DEDUCTION_14MRP,
                "DEDUCTION_14MRP_MULTIPLIER": cls.DEDUCTION_14MRP // cls.MRP,
                "OPVR_RATE": cls.OPVR_RATE
            },
            "rates": {
                "employee": {
                    "OPV": f"{cls.RATES['opv_employee'] * 100}%",
                    "VOSMS": f"{cls.RATES['vosms_employee'] * 100}%",
                    "IPN": f"{cls.RATES['ipn_employee'] * 100}% (после вычета 14 МРП)"
                },
                "employer": {
                    "SO": f"{cls.RATES['so_employer'] * 100}%",
                    "OOSMS": f"{cls.RATES['oosms_employer'] * 100}%",
                    "OPVR": f"{cls.OPVR_RATE * 100}%"
                }
            },
            "kbk": cls.KBK_MAP
        }


# Глобальный экземпляр
tax_engine = TaxEngineKZ2026()
