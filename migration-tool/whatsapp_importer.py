#!/usr/bin/env python3
"""
WhatsApp Importer - Импорт из WhatsApp TXT
"""

import re
import json
import sys
from datetime import datetime
from typing import List, Dict, Any

def parse_whatsapp_txt(txt_path: str) -> List[Dict[str, Any]]:
    """Парсинг WhatsApp TXT экспорта"""
    messages = []
    pattern = r'^(\d{2}/\d{2}/\d{4}, \d{2}:\d{2}) - ([^:]+): (.*)$'
    
    with open(txt_path, 'r', encoding='utf-8') as f:
        for line in f:
            match = re.match(pattern, line.strip())
            if match:
                date, name, text = match.groups()
                messages.append({
                    'date': date,
                    'from': name.strip(),
                    'text': text
                })
    
    return messages

def migrate(txt_path: str, output_path: str, translator=None):
    """Миграция WhatsApp экспорта"""
    messages = parse_whatsapp_txt(txt_path)
    
    lr_messages = []
    for i, msg in enumerate(messages, 1):
        text = msg['text']
        
        # Авто-перевод при импорте
        if translator and text:
            text = translator.translate(text, from_lang='auto', to_lang='bg')
        
        lr_messages.append({
            'id': i,
            'date': msg['date'],
            'from': msg['from'],
            'text': text,
            'source': 'whatsapp'
        })
    
    output = {
        'messages': lr_messages,
        'migrated_at': datetime.utcnow().isoformat()
    }
    
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(output, f, ensure_ascii=False, indent=2)
    
    print(f"✓ Миграция завершена: {len(lr_messages)} сообщений")
    print(f"  Сохранено в: {output_path}")

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Использование: python whatsapp_importer.py <export.txt> <output.json>")
        sys.exit(1)
    
    migrate(sys.argv[1], sys.argv[2])
