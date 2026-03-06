#!/usr/bin/env python3
"""
Telegram Importer - Импорт из Telegram Export (JSON)
"""

import json
import sys
from datetime import datetime
from typing import List, Dict, Any

def load_telegram_export(export_path: str) -> Dict[str, Any]:
    """Загрузка Telegram экспорта"""
    with open(export_path, 'r', encoding='utf-8') as f:
        return json.load(f)

def convert_message(msg: Dict[str, Any]) -> Dict[str, Any]:
    """Конвертация сообщения в формат Liberty Reach"""
    text = msg.get('text', '')
    if isinstance(text, list):
        text = ' '.join(t if isinstance(t, str) else t.get('text', '') for t in text)
    
    return {
        'id': msg.get('id', 0),
        'date': msg.get('date', ''),
        'from': msg.get('from', ''),
        'text': text,
        'source': 'telegram'
    }

def migrate(export_path: str, output_path: str, translator=None):
    """Миграция Telegram экспорта"""
    export = load_telegram_export(export_path)
    
    messages = []
    for msg in export.get('messages', []):
        converted = convert_message(msg)
        
        # Авто-перевод при импорте
        if translator and converted['text']:
            converted['text'] = translator.translate(
                converted['text'], 
                from_lang='auto', 
                to_lang='bg'
            )
        
        messages.append(converted)
    
    output = {
        'name': export.get('name', 'Unknown'),
        'messages': messages,
        'migrated_at': datetime.utcnow().isoformat()
    }
    
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(output, f, ensure_ascii=False, indent=2)
    
    print(f"✓ Миграция завершена: {len(messages)} сообщений")
    print(f"  Сохранено в: {output_path}")

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Использование: python telegram_importer.py <export.json> <output.json>")
        sys.exit(1)
    
    migrate(sys.argv[1], sys.argv[2])
