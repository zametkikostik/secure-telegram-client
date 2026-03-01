#!/usr/bin/env python3
"""Создание скриншотов-мокапов для Secure Telegram Client"""

from PIL import Image, ImageDraw, ImageFont
import os

def get_font(size):
    """Получить шрифт"""
    try:
        return ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", size)
    except:
        return ImageFont.load_default()

def get_bold_font(size):
    """Получить жирный шрифт"""
    try:
        return ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", size)
    except:
        return ImageFont.load_default()

def create_status_bar(draw, width):
    """Нарисовать статус-бар"""
    # Время
    draw.text((10, 5), "18:30", fill='#FFFFFF', font=get_font(14))
    # Иконки справа
    draw.text((width - 120, 5), "📶 🔋", fill='#FFFFFF', font=get_font(14))

def create_screenshot_1_home():
    """Главный экран - список чатов"""
    img = Image.new('RGB', (1080, 1920), color='#1B5E20')
    draw = ImageDraw.Draw(img)
    
    # Градиентный фон
    for y in range(1920):
        r = int(27 + (30 - 27) * y / 1920)
        g = int(125 + (60 - 125) * y / 1920)
        b = int(50 + (40 - 50) * y / 1920)
        draw.line([(0, y), (1080, y)], fill=(r, g, b))
    
    # Статус-бар
    create_status_bar(draw, 1080)
    
    # Заголовок
    draw.text((20, 50), "Secure Telegram", fill='#FFFFFF', font=get_bold_font(28))
    draw.text((20, 85), "Чаты", fill='#81C784', font=get_font(20))
    
    # Поиск
    draw.rounded_rectangle([20, 120, 1060, 170], radius=10, fill='#2E7D32', outline='#4CAF50')
    draw.text((50, 135), "🔍 Поиск...", fill='#81C784', font=get_font(18))
    
    # Список чатов
    chats = [
        ("Павел Дуров", "Добро пожаловать в Secure Telegram!", "18:25", True),
        ("Алиса", "Привет! Как тебе новое приложение?", "18:20", True),
        ("Боб", "Ключи верифицированы ✅", "17:45", False),
        ("Charlie", "Kyber-1024 работает отлично", "16:30", False),
        ("Новости", "Обновление v0.2.2 доступно", "15:00", False),
    ]
    
    y_offset = 200
    for name, message, time, unread in chats:
        # Фон чата
        draw.rectangle([20, y_offset, 1060, y_offset + 80], fill='#2E7D32' if unread else '#1B5E20')
        
        # Аватар
        draw.ellipse([35, y_offset + 10, 95, y_offset + 70], fill='#4CAF50')
        draw.text((50, y_offset + 25), name[0], fill='#FFFFFF', font=get_bold_font(24))
        
        # Имя
        draw.text((110, y_offset + 15), name, fill='#FFFFFF', font=get_bold_font(20))
        
        # Сообщение
        draw.text((110, y_offset + 40), message[:50], fill='#81C784', font=get_font(16))
        
        # Время
        draw.text((980, y_offset + 15), time, fill='#66BB6A', font=get_font(14))
        
        # Индикатор непрочитанных
        if unread:
            draw.ellipse([1020, y_offset + 30, 1045, y_offset + 55], fill='#4CAF50')
            draw.text((1025, y_offset + 33), "2", fill='#FFFFFF', font=get_bold_font(14))
        
        # Разделитель
        draw.line([(20, y_offset + 80), (1060, y_offset + 80)], fill='#2E7D32', width=1)
        
        y_offset += 85
    
    # Нижняя навигация
    draw.rectangle([0, 1820, 1080, 1920], fill='#1B5E20')
    draw.text((100, 1850), "🏠", fill='#4CAF50', font=get_font(32))
    draw.text((300, 1850), "💬", fill='#81C784', font=get_font(32))
    draw.text((500, 1850), "📞", fill='#81C784', font=get_font(32))
    draw.text((700, 1850), "👥", fill='#81C784', font=get_font(32))
    draw.text((900, 1850), "⚙️", fill='#81C784', font=get_font(32))
    
    # Сохранение
    output_path = '/home/kostik/secure-telegram-client/assets/screenshot_1_home.png'
    img.save(output_path, 'PNG')
    print(f"✅ Скриншот 1: {output_path}")

def create_screenshot_2_chat():
    """Экран чата"""
    img = Image.new('RGB', (1080, 1920), color='#0D3D1A')
    draw = ImageDraw.Draw(img)
    
    # Градиентный фон
    for y in range(1920):
        r = int(13 + (20 - 13) * y / 1920)
        g = int(61 + (40 - 61) * y / 1920)
        b = int(26 + (30 - 26) * y / 1920)
        draw.line([(0, y), (1080, y)], fill=(r, g, b))
    
    # Статус-бар
    create_status_bar(draw, 1080)
    
    # Заголовок чата
    draw.rectangle([0, 40, 1080, 110], fill='#1B5E20')
    draw.ellipse([30, 55, 80, 105], fill='#4CAF50')
    draw.text((95, 60), "Алиса", fill='#FFFFFF', font=get_bold_font(24))
    draw.text((95, 88), "в сети", fill='#81C784', font=get_font(14))
    draw.text((1000, 65), "🔐", fill='#4CAF50', font=get_font(28))
    
    # Сообщения
    messages = [
        ("left", "Привет! 👋", "18:15"),
        ("right", "Привет! Как тебе Secure Telegram?", "18:16"),
        ("left", "Отлично! Шифрование работает?", "18:17"),
        ("right", "Да! Kyber-1024 + XChaCha20", "18:18"),
        ("left", "Круто! Ключи уже верифицированы?", "18:19"),
        ("right", "✅ Да, всё зелёное!", "18:20"),
    ]
    
    y_offset = 140
    for side, text, time in messages:
        if side == "right":
            # Исходящее сообщение
            draw.rounded_rectangle([540, y_offset, 1050, y_offset + 55], radius=15, fill='#2E7D32')
            draw.text([560, y_offset + 15], text, fill='#FFFFFF', font=get_font(18))
            draw.text([1000, y_offset + 35], time, fill='#81C784', font=get_font(12))
        else:
            # Входящее сообщение
            draw.rounded_rectangle([30, y_offset, 520, y_offset + 55], radius=15, fill='#2E7D32')
            draw.text([50, y_offset + 15], text, fill='#FFFFFF', font=get_font(18))
            draw.text([470, y_offset + 35], time, fill='#81C784', font=get_font(12))
        y_offset += 70
    
    # Поле ввода
    draw.rectangle([0, 1820, 1080, 1920], fill='#1B5E20')
    draw.rounded_rectangle([20, 1835, 950, 1905], radius=25, fill='#2E7D32')
    draw.text([50, 1855], "Сообщение...", fill='#81C784', font=get_font(18))
    draw.ellipse([970, 1845, 1050, 1895], fill='#4CAF50')
    draw.text([995, 1855], "➤", fill='#FFFFFF', font=get_font(24))
    
    # Сохранение
    output_path = '/home/kostik/secure-telegram-client/assets/screenshot_2_chat.png'
    img.save(output_path, 'PNG')
    print(f"✅ Скриншот 2: {output_path}")

def create_screenshot_3_settings():
    """Настройки"""
    img = Image.new('RGB', (1080, 1920), color='#1B5E20')
    draw = ImageDraw.Draw(img)
    
    # Градиентный фон
    for y in range(1920):
        g = int(125 - 40 * y / 1920)
        draw.line([(0, y), (1080, y)], fill=(27, g, 50))
    
    # Статус-бар
    create_status_bar(draw, 1080)
    
    # Заголовок
    draw.text((20, 50), "Настройки", fill='#FFFFFF', font=get_bold_font(32))
    
    # Секции настроек
    sections = [
        ("🔐 Безопасность", [
            "Post-Quantum шифрование",
            "Obfs4 обфускация",
            "Verifiy ключи",
        ]),
        ("🌐 Сеть", [
            "DNS over HTTPS",
            "Прокси: Выключено",
            "P2P режим",
        ]),
        ("📱 Приложение", [
            "Тема: Зелёная",
            "Язык: Русский",
            "Версия: 0.2.2",
        ]),
    ]
    
    y_offset = 130
    for section_title, items in sections:
        # Заголовок секции
        draw.text((20, y_offset), section_title, fill='#4CAF50', font=get_bold_font(18))
        y_offset += 20
        
        # Элементы
        for item in items:
            draw.rectangle([20, y_offset, 1060, y_offset + 60], fill='#2E7D32')
            draw.text((40, y_offset + 20), item, fill='#FFFFFF', font=get_font(18))
            draw.text((1020, y_offset + 20), "›", fill='#81C784', font=get_bold_font(28))
            y_offset += 70
        
        y_offset += 20
    
    # Сохранение
    output_path = '/home/kostik/secure-telegram-client/assets/screenshot_3_settings.png'
    img.save(output_path, 'PNG')
    print(f"✅ Скриншот 3: {output_path}")

def create_screenshot_4_privacy():
    """Экран приватности"""
    img = Image.new('RGB', (1080, 1920), color='#0D3D1A')
    draw = ImageDraw.Draw(img)
    
    # Градиентный фон
    for y in range(1920):
        draw.line([(0, y), (1080, y)], fill=(13, 61, 26))
    
    # Статус-бар
    create_status_bar(draw, 1080)
    
    # Заголовок
    draw.text((20, 50), "Приватность", fill='#FFFFFF', font=get_bold_font(32))
    
    # Щит в центре
    draw.ellipse([340, 150, 740, 550], fill='#1B5E20', outline='#4CAF50', width=8)
    draw.text([480, 320], "🔐", fill='#4CAF50', font=get_font(100))
    
    # Статус
    draw.text([250, 600], "✅ Всё защищено", fill='#4CAF50', font=get_bold_font(28))
    
    # Список функций
    features = [
        ("✅", "Kyber-1024 шифрование", "Постквантовая защита"),
        ("✅", "XChaCha20-Poly1305", "Симметричное шифрование"),
        ("✅", "X25519", "Обмен ключами"),
        ("✅", "Obfs4", "Маскировка трафика"),
        ("✅", "DNS over HTTPS", "Обход DNS блокировок"),
        ("✅", "P2P коммуникация", "Без центрального сервера"),
    ]
    
    y_offset = 700
    for icon, title, desc in features:
        draw.rectangle([40, y_offset, 1040, y_offset + 70], fill='#1B5E20')
        draw.text([60, y_offset + 20], icon, fill='#4CAF50', font=get_font(24))
        draw.text([110, y_offset + 18], title, fill='#FFFFFF', font=get_bold_font(18))
        draw.text([110, y_offset + 42], desc, fill='#81C784', font=get_font(14))
        y_offset += 80
    
    # Сохранение
    output_path = '/home/kostik/secure-telegram-client/assets/screenshot_4_privacy.png'
    img.save(output_path, 'PNG')
    print(f"✅ Скриншот 4: {output_path}")

def create_screenshot_5_about():
    """О приложении"""
    img = Image.new('RGB', (1080, 1920), color='#1B5E20')
    draw = ImageDraw.Draw(img)
    
    # Градиентный фон
    for y in range(1920):
        g = int(125 - 30 * y / 1920)
        draw.line([(0, y), (1080, y)], fill=(27, g, 50))
    
    # Статус-бар
    create_status_bar(draw, 1080)
    
    # Логотип
    draw.ellipse([390, 100, 690, 400], fill='#2E7D32', outline='#4CAF50', width=8)
    draw.text([480, 220], "🔐", fill='#FFFFFF', font=get_font(80))
    
    # Название
    draw.text([180, 430], "Secure Telegram", fill='#FFFFFF', font=get_bold_font(36))
    draw.text([450, 475], "v0.2.2", fill='#81C784', font=get_font(20))
    
    # Описание
    draw.text([100, 530], "Децентрализованный мессенджер", fill='#FFFFFF', font=get_font(22))
    draw.text([150, 565], "с постквантовым шифрованием", fill='#81C784', font=get_font(20))
    
    # Информация
    info_y = 680
    info_items = [
        ("Движок:", "Rust + Kotlin"),
        ("Telegram API:", "TDLib"),
        ("Шифрование:", "Kyber-1024 + XChaCha20"),
        ("Лицензия:", "MIT"),
    ]
    
    for label, value in info_items:
        draw.text([100, info_y], label, fill='#4CAF50', font=get_bold_font(18))
        draw.text([350, info_y], value, fill='#FFFFFF', font=get_font(18))
        info_y += 50
    
    # GitHub
    draw.rectangle([100, 850, 980, 920], fill='#2E7D32', outline='#4CAF50')
    draw.text([350, 875], "github.com/zametkikostik", fill='#FFFFFF', font=get_font(20))
    draw.text([420, 900], "/secure-telegram-client", fill='#81C784', font=get_font(18))
    
    # Кнопки
    draw.rounded_rectangle([100, 1000, 520, 1070], radius=10, fill='#4CAF50')
    draw.text([230, 1025], "Проверить обновления", fill='#FFFFFF', font=get_bold_font(18))
    
    draw.rounded_rectangle([560, 1000, 980, 1070], radius=10, fill='#2E7D32', outline='#4CAF50')
    draw.text([680, 1025], "Исходный код", fill='#FFFFFF', font=get_bold_font(18))
    
    # Footer
    draw.text([280, 1800], "Made with ❤️ for Privacy", fill='#66BB6A', font=get_font(16))
    
    # Сохранение
    output_path = '/home/kostik/secure-telegram-client/assets/screenshot_5_about.png'
    img.save(output_path, 'PNG')
    print(f"✅ Скриншот 5: {output_path}")

# Создание всех скриншотов
if __name__ == "__main__":
    os.makedirs('/home/kostik/secure-telegram-client/assets', exist_ok=True)
    
    create_screenshot_1_home()
    create_screenshot_2_chat()
    create_screenshot_3_settings()
    create_screenshot_4_privacy()
    create_screenshot_5_about()
    
    print("\n✅ Все скриншоты созданы!")
    print("Путь: /home/kostik/secure-telegram-client/assets/")
