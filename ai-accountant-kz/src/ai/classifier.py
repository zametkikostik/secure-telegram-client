"""AI-классификатор транзакций с guardrails"""
import json
import httpx
from typing import Optional
from loguru import logger
from decimal import Decimal

from ..core.tax_rules import INCOME_CATEGORIES, EXPENSE_CATEGORIES, EXEMPT_SOURCES
from ..core.config import settings

CLASSIFICATION_PROMPT = """Ты AI-ассистент для классификации транзакций ИП в Казахстане на упрощёнке.

КАТЕГОРИИ ДОХОДОВ:
%s

КАТЕГОРИИ РАСХОДОВ:
%s

ИСКЛЮЧЕНИЯ (не считаются доходом для налога):
%s

ПРАВИЛА:
1. Для доходов выбирай категорию из "КАТЕГОРИИ ДОХОДОВ"
2. Для расходов выбирай категорию из "КАТЕГОРИИ РАСХОДОВ"
3. Если это исключение (возврат кредита, перевод между своими счетами) - укажи это
4. Возврат от поставщика - это исключение, не доход
5. Получение кредита - исключение, не доход

Транзакция:
- Тип: %s
- Сумма: %s ₸
- Описание: %s
- Контрагент: %s

Верни ТОЛЬКО JSON без markdown:
{
    "category": "название категории",
    "confidence": 0.0-1.0,
    "reasoning": "краткое объяснение",
    "needs_review": false,
    "is_exempt": false,
    "exempt_reason": "причина если исключение"
}
"""

class AIClassifier:
    def __init__(self):
        self.min_confidence = settings.AI_CONFIDENCE_THRESHOLD
        self.api_key = settings.OPENAI_API_KEY
        self.model = settings.OPENAI_MODEL
        self.base_url = "https://dashscope.aliyuncs.com/compatible-mode/v1"
    
    async def classify(
        self,
        description: str,
        amount: float,
        counterparty: Optional[str] = None,
        tx_type: str = "income"
    ) -> dict:
        prompt = CLASSIFICATION_PROMPT % (
            "\n".join([f"- {k}: {v}" for k, v in INCOME_CATEGORIES.items()]),
            "\n".join([f"- {k}: {v}" for k, v in EXPENSE_CATEGORIES.items()]),
            "\n".join([f"- {k}: {v}" for k, v in EXEMPT_SOURCES.items()]),
            "Доход" if tx_type == "income" else "Расход",
            amount,
            description,
            counterparty or "Н/Д"
        )

        # Если нет API ключа - возвращаем заглушку
        if not self.api_key:
            logger.warning("Нет API ключа для AI, используем заглушку")
            return self._fallback_classify(description, tx_type)
        
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(
                    f"{self.base_url}/chat/completions",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    },
                    json={
                        "model": self.model,
                        "messages": [
                            {"role": "system", "content": "Ты помощник для классификации финансовых транзакций. Отвечай ТОЛЬКО JSON."},
                            {"role": "user", "content": prompt}
                        ],
                        "temperature": 0.1,
                        "max_tokens": 500
                    }
                )
                
                if response.status_code == 200:
                    data = response.json()
                    content = data["choices"][0]["message"]["content"]
                    result = json.loads(content.strip())
                    
                    # Проверка confidence
                    if result.get("confidence", 0) < self.min_confidence:
                        result["needs_review"] = True
                        result["warning"] = f"Низкая уверенность ({result.get('confidence', 0):.0%})"
                    
                    logger.info(f"Classified: {description[:30]}... -> {result.get('category')} ({result.get('confidence', 0):.0%})")
                    return result
                else:
                    logger.error(f"AI API error: {response.status_code} - {response.text}")
                    return self._fallback_classify(description, tx_type)
                    
        except json.JSONDecodeError as e:
            logger.error(f"JSON parse error: {e}")
            return self._fallback_classify(description, tx_type)
        except Exception as e:
            logger.error(f"AI classification error: {e}")
            return self._fallback_classify(description, tx_type)
    
    def _fallback_classify(self, description: str, tx_type: str) -> dict:
        """Заглушка если AI недоступен"""
        description_lower = description.lower()
        
        # Простая эвристика
        if tx_type == "income":
            if any(word in description_lower for word in ["возврат", "кредит", "займ"]):
                return {
                    "category": "refund",
                    "confidence": 0.9,
                    "reasoning": "Похоже на возврат или кредит",
                    "needs_review": False,
                    "is_exempt": True,
                    "exempt_reason": "Возврат кредита/займа"
                }
            return {
                "category": "sales",
                "confidence": 0.7,
                "reasoning": "Доход по умолчанию",
                "needs_review": True,
                "is_exempt": False
            }
        else:
            if any(word in description_lower for word in ["аренда", "помещение"]):
                category = "rent"
            elif any(word in description_lower for word in ["зарплата", "сотрудник"]):
                category = "salary"
            elif any(word in description_lower for word in ["налог", "сбор"]):
                category = "taxes"
            else:
                category = "suppliers"
            
            return {
                "category": category,
                "confidence": 0.6,
                "reasoning": "Классификация по умолчанию",
                "needs_review": True,
                "is_exempt": False
            }

# Глобальный экземпляр
classifier = AIClassifier()
