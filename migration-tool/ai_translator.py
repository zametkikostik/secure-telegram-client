#!/usr/bin/env python3
"""
AI Translator - Перевод через Qwen API
"""

import requests
import os
from typing import Optional

class QwenTranslator:
    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key or os.environ.get('QWEN_API_KEY', '')
        self.url = 'https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation'
    
    def translate(self, text: str, from_lang: str, to_lang: str) -> str:
        """Перевод текста"""
        headers = {
            'Authorization': f'Bearer {self.api_key}',
            'Content-Type': 'application/json'
        }
        
        payload = {
            'model': 'qwen-turbo',
            'messages': [{
                'role': 'user',
                'content': f'Translate from {from_lang} to {to_lang}: {text}'
            }],
            'max_tokens': 2048
        }
        
        try:
            response = requests.post(self.url, headers=headers, json=payload)
            response.raise_for_status()
            result = response.json()
            return result.get('output', {}).get('text', text)
        except Exception as e:
            print(f"Ошибка перевода: {e}")
            return text
    
    def detect_language(self, text: str) -> str:
        """Определение языка"""
        headers = {
            'Authorization': f'Bearer {self.api_key}',
            'Content-Type': 'application/json'
        }
        
        payload = {
            'model': 'qwen-turbo',
            'messages': [{
                'role': 'user',
                'content': f'Detect language of: {text}'
            }],
            'max_tokens': 100
        }
        
        try:
            response = requests.post(self.url, headers=headers, json=payload)
            response.raise_for_status()
            result = response.json()
            return result.get('output', {}).get('text', 'en')
        except Exception as e:
            print(f"Ошибка определения языка: {e}")
            return 'en'

if __name__ == '__main__':
    translator = QwenTranslator()
    
    # Тест
    text = "Привет, как дела?"
    translated = translator.translate(text, 'ru', 'bg')
    print(f"{text} -> {translated}")
