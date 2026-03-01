#!/usr/bin/env python3
"""Создание логотипа для Secure Telegram Client"""

from PIL import Image, ImageDraw
import os

# Создаём изображение 512x512
img = Image.new('RGB', (512, 512), color='#2E7D32')  # Зелёный фон
draw = ImageDraw.Draw(img)

# Рисуем щит (shield)
shield_points = [
    (256, 60),   # Верх
    (420, 120),  # Правый верх
    (420, 280),  # Правый низ
    (256, 460),  # Низ
    (92, 280),   # Левый низ
    (92, 120),   # Левый верх
]
draw.polygon(shield_points, fill='#1B5E20', outline='#4CAF50', width=8)

# Рисуем замок внутри щита
# Тело замка
draw.rectangle([200, 240, 312, 380], fill='#81C784', outline='#2E7D32', width=4)

# Дужка замка
draw.arc([220, 180, 292, 260], start=180, end=0, fill='#81C784', width=12)

# Замочная скважина
draw.ellipse([246, 290, 266, 310], fill='#1B5E20')
draw.rectangle([251, 310, 261, 340], fill='#1B5E20')

# Добавляем блеск
draw.polygon([(256, 70), (410, 125), (256, 100)], fill='#4CAF50')

# Сохраняем
output_path = '/home/kostik/secure-telegram-client/assets/app_icon_512.png'
os.makedirs(os.path.dirname(output_path), exist_ok=True)
img.save(output_path, 'PNG')

print(f"✅ Логотип создан: {output_path}")
print(f"Размер: 512x512 пикселей")
print(f"Размер файла: {os.path.getsize(output_path) / 1024:.1f} KB")
