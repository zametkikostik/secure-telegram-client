"""OCR для распознавания чеков и квитанций"""
import base64
import io
import httpx
from PIL import Image
from pathlib import Path
from typing import Optional, Dict
from loguru import logger

from ..core.config import settings


class ReceiptOCR:
    """Распознавание чеков с помощью AI"""

    def __init__(self):
        self.api_key = settings.OPENAI_API_KEY
        self.model = settings.OPENAI_MODEL
        self.base_url = settings.DASHSCOPE_BASE_URL
        self.enabled = bool(self.api_key)
        
        # Hugging Face Inference API (бесплатный, без привязки карты)
        self.hf_api_url = "https://api-inference.huggingface.co/models/google/vit-base-patch16-224"
    
    async def recognize_from_file(self, file_path: str) -> Optional[Dict]:
        """Распознавание чека из файла"""
        if not self.enabled:
            logger.warning("OCR отключен (нет API ключа)")
            return None
        
        try:
            # Загрузка изображения
            with open(file_path, 'rb') as f:
                image_data = f.read()
            
            # Конвертация в base64
            image_base64 = base64.b64encode(image_data).decode('utf-8')
            
            return await self._recognize_from_base64(image_base64)
            
        except Exception as e:
            logger.error(f"OCR file error: {e}")
            return None
    
    async def recognize_from_base64(self, image_base64: str) -> Optional[Dict]:
        """Распознавание чека из base64"""
        if not self.enabled:
            return None
        
        return await self._recognize_from_base64(image_base64)
    
    async def _recognize_from_base64(self, image_base64: str) -> Optional[Dict]:
        """Внутренний метод распознавания"""
        try:
            async with httpx.AsyncClient(timeout=60.0) as client:
                # Запрос к Qwen-VL (Vision Language модель)
                response = await client.post(
                    f"{self.base_url}/chat/completions",
                    headers={
                        "Authorization": f"Bearer {self.api_key}",
                        "Content-Type": "application/json"
                    },
                    json={
                        "model": "qwen-vl-plus",
                        "messages": [
                            {
                                "role": "user",
                                "content": [
                                    {
                                        "type": "image_url",
                                        "image_url": {"url": f"data:image/jpeg;base64,{image_base64}"}
                                    },
                                    {
                                        "type": "text",
                                        "text": """Это чек или квитанция. Извлеки следующую информацию в формате JSON:
{
    "type": "income или expense",
    "amount": число (сумма),
    "date": "YYYY-MM-DD",
    "merchant": "название магазина/организации",
    "inn": "ИНН продавца",
    "items": ["список товаров"],
    "payment_method": "наличный/безналичный"
}

Если это не чек, верни {"error": "not_a_receipt"}"""
                                    }
                                ]
                            }
                        ],
                        "max_tokens": 500
                    }
                )
                
                if response.status_code == 200:
                    data = response.json()
                    content = data["choices"][0]["message"]["content"]
                    
                    # Парсинг JSON из ответа
                    import json
                    result = json.loads(content.strip())
                    
                    logger.info(f"OCR распознал: {result.get('merchant', 'unknown')} - {result.get('amount', 0)}")
                    return result
                else:
                    logger.error(f"OCR API error: {response.status_code}")
                    return None
                    
        except Exception as e:
            logger.error(f"OCR exception: {e}")
            return None
    
    async def recognize_with_tesseract(self, image_path: str) -> Optional[str]:
        """Распознавание текста с помощью Tesseract (fallback)"""
        try:
            import pytesseract
            from PIL import Image, ImageFilter, ImageEnhance

            image = Image.open(image_path)
            
            # Предобработка изображения для лучшего распознавания
            # Конвертация в оттенки серого
            image = image.convert('L')
            
            # Увеличение контраста
            enhancer = ImageEnhance.Contrast(image)
            image = enhancer.enhance(2.0)
            
            # Увеличение резкости
            enhancer = ImageEnhance.Sharpness(image)
            image = enhancer.enhance(2.0)
            
            # Бинаризация (чёрно-белое)
            threshold = 128
            image = image.point(lambda p: 255 if p > threshold else 0)
            
            # Распознавание с улучшенными параметрами
            config = '--oem 3 --psm 6'  # OEM 3 = LSTM + legacy, PSM 6 = Uniform block of text
            text = pytesseract.image_to_string(image, lang='rus+eng', config=config)

            logger.info(f"Tesseract распознал {len(text)} символов")
            return text

        except Exception as e:
            logger.error(f"Tesseract error: {e}")
            return None


class ReceiptProcessor:
    """Обработчик чеков для создания транзакций"""
    
    def __init__(self):
        self.ocr = ReceiptOCR()
    
    async def process_receipt(self, image_data: bytes) -> Dict:
        """
        Обработка чека и создание данных для транзакции
        
        Returns:
            {
                "success": bool,
                "transaction_data": dict или None,
                "raw_text": str,
                "confidence": float
            }
        """
        # Сохраняем временный файл
        temp_path = Path("data/temp_receipts")
        temp_path.mkdir(exist_ok=True)
        
        temp_file = temp_path / f"receipt_{id(image_data)}.jpg"
        with open(temp_file, 'wb') as f:
            f.write(image_data)
        
        try:
            # AI распознавание
            ocr_result = await self.ocr.recognize_from_file(str(temp_file))
            
            if ocr_result and 'error' not in ocr_result:
                # Извлекаем данные для транзакции
                transaction_data = {
                    "type": ocr_result.get('type', 'expense'),
                    "amount": float(ocr_result.get('amount', 0)),
                    "date": ocr_result.get('date'),
                    "counterparty": ocr_result.get('merchant'),
                    "counterparty_inn": ocr_result.get('inn'),
                    "description": f"Чек от {ocr_result.get('merchant', 'неизвестно')}",
                    "source": "ocr"
                }
                
                return {
                    "success": True,
                    "transaction_data": transaction_data,
                    "raw_text": str(ocr_result),
                    "confidence": 0.9,
                    "items": ocr_result.get('items', [])
                }
            
            # Fallback на Tesseract
            tesseract_text = await self.ocr.recognize_with_tesseract(str(temp_file))
            
            return {
                "success": tesseract_text is not None,
                "transaction_data": None,
                "raw_text": tesseract_text or "",
                "confidence": 0.5,
                "items": []
            }
            
        finally:
            # Удаляем временный файл
            if temp_file.exists():
                temp_file.unlink()


# Глобальный экземпляр
receipt_processor = ReceiptProcessor()
