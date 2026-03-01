#!/usr/bin/env python3
"""Создание Feature Graphic баннера для Secure Telegram Client"""

from PIL import Image, ImageDraw, ImageFont
import os

# Создаём изображение 1024x500
img = Image.new('RGB', (1024, 500), color='#1B5E20')  # Тёмно-зелёный фон
draw = ImageDraw.Draw(img)

# Рисуем градиентный фон (слева направо)
for x in range(1024):
    r = int(27 + (46 - 27) * x / 1024)
    g = int(125 + (125 - 125) * x / 1024)
    b = int(50 + (72 - 50) * x / 1024)
    draw.line([(x, 0), (x, 500)], fill=(r, g, b))

# Рисуем щит (shield) слева - увеличенный
shield_scale = 2.5
shield_points = [
    (180, int(60 * shield_scale / 2)),      # Верх
    (int(420 * shield_scale / 2), int(120 * shield_scale / 2)),  # Правый верх
    (int(420 * shield_scale / 2), int(280 * shield_scale / 2)),  # Правый низ
    (180, int(460 * shield_scale / 2)),     # Низ
    (int(92 * shield_scale / 2), int(280 * shield_scale / 2)),   # Левый низ
    (int(92 * shield_scale / 2), int(120 * shield_scale / 2)),   # Левый верх
]
# Сдвигаем щит влево
shield_points = [(x - 50, y + 50) for x, y in shield_points]
draw.polygon(shield_points, fill='#2E7D32', outline='#4CAF50', width=6)

# Рисуем замок внутри щита
# Тело замка
draw.rectangle([130, 260, 230, 400], fill='#81C784', outline='#1B5E20', width=3)

# Дужка замка
draw.arc([150, 200, 210, 280], start=180, end=0, fill='#81C784', width=10)

# Замочная скважина
draw.ellipse([170, 310, 190, 330], fill='#1B5E20')
draw.rectangle([175, 330, 185, 360], fill='#1B5E20')

# Текст справа
title_text = "Secure Telegram"
subtitle_text = "Privacy-Focused Messenger"
features_text = "• Post-Quantum Encryption  • Obfs4  • P2P  • Anti-Censorship"

# Используем шрифт по умолчанию (будет заменён на системный если есть)
try:
    title_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 48)
    subtitle_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Italic.ttf", 28)
    features_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 20)
except:
    title_font = ImageFont.load_default()
    subtitle_font = ImageFont.load_default()
    features_font = ImageFont.load_default()

# Рисуем текст
# Заголовок
draw.text((320, 150), title_text, fill='#FFFFFF', font=title_font)

# Подзаголовок
draw.text((320, 210), subtitle_text, fill='#81C784', font=subtitle_font)

# Разделительная линия
draw.line([(320, 260), (980, 260)], fill='#4CAF50', width=2)

# Особенности
draw.text((320, 300), features_text, fill='#C8E6C9', font=features_font)

# Версия
version_text = "v0.2.2"
draw.text((850, 440), version_text, fill='#66BB6A', font=features_font)

# Сохраняем
output_path = '/home/kostik/secure-telegram-client/assets/feature_graphic_1024x500.png'
os.makedirs(os.path.dirname(output_path), exist_ok=True)
img.save(output_path, 'PNG')

print(f"✅ Feature Graphic создан: {output_path}")
print(f"Размер: 1024x500 пикселей")
print(f"Размер файла: {os.path.getsize(output_path) / 1024:.1f} KB")
