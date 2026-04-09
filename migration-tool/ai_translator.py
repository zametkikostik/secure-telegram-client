#!/usr/bin/env python3
"""
AI Translator — авто-перевод импортированных сообщений.

Переводит сообщения из внутреннего формата (Telegram/WhatsApp импортеры)
в целевой язык, обновляя encrypted_content и добавляя перевод в meta.

Использование:
    # С OpenAI (требуется OPENAI_API_KEY)
    python3 ai_translator.py internal_export.json --lang en --api openai

    # С Google Translate (бесплатно, без ключа)
    python3 ai_translator.py internal_export.json --lang en --api google

    # С локальной моделью через Ollama
    python3 ai_translator.py internal_export.json --lang en --api ollama --model llama3

    # Оффлайн: только детекция языка без перевода
    python3 ai_translator.py internal_export.json --detect-only
"""

import argparse
import base64
import hashlib
import json
import os
import sys
from pathlib import Path
from typing import Optional

# Ленивый импорт API-клиентов
_openai_client = None
_google_translate = None


# ============================================================================
# Детекция языка (встроенная, без внешних зависимостей)
# ============================================================================

# Частотные n-граммы для основных языков (укороченные)
LANG_NGRAMS = {
    "en": {"the", "and", "ing", "tion", "tha", "ent", "for", " wit", "have", "thi", "not", "you", "are", "was", "his", "her"},
    "ru": {"ого", "его", "что", "это", "ный", "ние", "ова", "ева", "как", "ний", "тов", "сть", "про", "все", "мен", "при"},
    "de": {"ung", "lich", "eit", "keit", "sch", "cht", "nde", "der", "die", "das", "ein", "und", "ich", "ist"},
    "es": {"ción", "sión", "dad", "ndo", "que", "con", "por", "los", "las", "del", "una", "ser", "para"},
    "fr": {"tion", "ment", "que", "les", "des", "est", "ont", "ont", "par", "pas", "une", "dan", "pro"},
    "uk": {"ння", "ого", "ого", "сть", "ний", "ою", "ією", "при", "про", "яки", "на ", "від", "дні"},
    "zh": {"的", "了", "是", "在", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上"},
    "ja": {"の", "に", "は", "を", "が", "で", "た", "と", "て", "い", "る", "です", "ます"},
}

COMMON_STOP_WORDS = {
    "en": {"the", "and", "is", "in", "to", "of", "a", "for", "that", "on", "with", "as", "was", "are", "it", "this", "have", "from", "at", "by"},
    "ru": {"и", "в", "не", "на", "я", "с", "он", "а", "по", "это", "она", "они", "к", "у", "мы", "как", "из", "за", "что", "но"},
    "de": {"der", "die", "und", "in", "den", "von", "zu", "das", "mit", "sich", "des", "auf", "für", "ist", "dem", "nicht", "ein", "eine"},
    "es": {"de", "la", "que", "el", "en", "y", "a", "los", "del", "se", "las", "por", "un", "para", "con", "no", "una", "su", "es"},
    "fr": {"de", "le", "et", "les", "des", "un", "du", "en", "une", "la", "dans", "est", "que", "pour", "pas", "il", "qui", "ce", "se", "les"},
}


def detect_language(text: str) -> tuple[str, float]:
    """
    Простая детекция языка по n-граммам и стоп-словам.
    
    Returns:
        (lang_code, confidence) — например ("ru", 0.85)
    """
    if not text or len(text.strip()) < 3:
        return ("unknown", 0.0)
    
    text_lower = text.lower()
    scores: dict[str, float] = {}
    
    # Метод 1: n-граммы (3-4 символа)
    for lang, ngrams in LANG_NGRAMS.items():
        matches = 0
        for ngram in ngrams:
            if ngram in text_lower:
                matches += 1
        scores[lang] = matches / max(len(ngrams), 1)
    
    # Метод 2: стоп-слова
    words = set(text_lower.split())
    for lang, stop_words in COMMON_STOP_WORDS.items():
        matches = len(words & stop_words)
        if matches > 0:
            scores[lang] = scores.get(lang, 0) + matches * 0.15
    
    if not scores or max(scores.values()) == 0:
        return ("unknown", 0.0)
    
    best_lang = max(scores, key=scores.get)
    confidence = min(scores[best_lang] * 2, 1.0)  # нормализация
    
    return (best_lang, round(confidence, 2))


# ============================================================================
# API-клиенты для перевода
# ============================================================================

def translate_openai(text: str, target_lang: str, model: str = "gpt-4o-mini", api_key: Optional[str] = None) -> str:
    """Перевод через OpenAI API."""
    global _openai_client
    if _openai_client is None:
        try:
            from openai import OpenAI
            _openai_client = OpenAI(api_key=api_key or os.environ.get("OPENAI_API_KEY"))
        except ImportError:
            raise ImportError("Установите: pip install openai")
    
    try:
        response = _openai_client.chat.completions.create(
            model=model,
            messages=[
                {"role": "system", "content": f"You are a translator. Translate the following text to {target_lang}. Return ONLY the translation, nothing else."},
                {"role": "user", "content": text},
            ],
            temperature=0.3,
            max_tokens=500,
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        raise RuntimeError(f"OpenAI translation error: {e}")


def translate_google(text: str, target_lang: str) -> str:
    """
    Перевод через бесплатный Google Translate (без API ключа).
    Использует unofficial API через HTTP-запросы.
    """
    import urllib.parse
    import urllib.request
    
    url = "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}".format(
        target_lang,
        urllib.parse.quote(text.encode('utf-8'))
    )
    
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"}
    )
    
    try:
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
            # Формат: [[["translated", "original", ...], ...], ...]
            translated_parts = []
            for segment in data[0]:
                if segment and segment[0]:
                    translated_parts.append(segment[0])
            return "".join(translated_parts)
    except Exception as e:
        raise RuntimeError(f"Google Translate error: {e}")


def translate_ollama(text: str, target_lang: str, model: str = "llama3", base_url: str = "http://localhost:11434") -> str:
    """Перевод через локальную модель Ollama."""
    import urllib.request
    
    prompt = f"Translate the following text to {target_lang}. Return ONLY the translation, nothing else:\n\n{text}"
    
    payload = json.dumps({
        "model": model,
        "prompt": prompt,
        "stream": False,
    }).encode('utf-8')
    
    req = urllib.request.Request(
        f"{base_url}/api/generate",
        data=payload,
        headers={"Content-Type": "application/json"},
    )
    
    try:
        with urllib.request.urlopen(req, timeout=120) as response:
            result = json.loads(response.read().decode('utf-8'))
            return result.get("response", "").strip()
    except Exception as e:
        raise RuntimeError(f"Ollama translation error: {e}. Убедитесь что Ollama запущен: ollama serve")


# ============================================================================
# Основной переводчик
# ============================================================================

class AITranslator:
    """Авто-перевод импортированных сообщений."""
    
    def __init__(
        self,
        api: str = "google",
        target_lang: str = "en",
        model: Optional[str] = None,
        api_key: Optional[str] = None,
        ollama_base_url: str = "http://localhost:11434",
        skip_if_target_lang: bool = True,
        dry_run: bool = False,
    ):
        self.api = api
        self.target_lang = target_lang
        self.model = model
        self.api_key = api_key
        self.ollama_base_url = ollama_base_url
        self.skip_if_target_lang = skip_if_target_lang
        self.dry_run = dry_run
        
        self.stats = {
            "total_messages": 0,
            "translated": 0,
            "skipped_same_lang": 0,
            "skipped_no_text": 0,
            "skipped_media": 0,
            "errors": 0,
        }
    
    def _translate_text(self, text: str) -> str:
        """Перевод текста через выбранный API."""
        if self.api == "openai":
            return translate_openai(text, self.target_lang, self.model or "gpt-4o-mini", self.api_key)
        elif self.api == "google":
            return translate_google(text, self.target_lang)
        elif self.api == "ollama":
            return translate_ollama(text, self.target_lang, self.model or "llama3", self.ollama_base_url)
        else:
            raise ValueError(f"Unknown API: {self.api}")
    
    def translate_export(self, input_path: str, output_path: Optional[str] = None) -> dict:
        """
        Перевод всех сообщений в импортированном файле.
        
        Args:
            input_path: путь к internal_export.json (результат telegram/whatsapp импортера)
            output_path: путь к выходному файлу (если None — перезаписывает входной)
        
        Returns:
            Обновлённый dict с переведёнными сообщениями
        """
        input_path = Path(input_path)
        if not input_path.exists():
            raise FileNotFoundError(f"File not found: {input_path}")
        
        print(f"Чтение {input_path}...")
        with open(input_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        
        messages = data.get("messages", [])
        if not messages:
            raise ValueError("No messages found in export file")
        
        print(f"Найдено сообщений: {len(messages)}")
        print(f"Целевой язык: {self.target_lang}")
        print(f"API: {self.api}" + (f" ({self.model})" if self.model else ""))
        
        if self.dry_run:
            print("\n🔍 DRY RUN — только статистика языков\n")
        
        for i, msg in enumerate(messages):
            self.stats["total_messages"] += 1
            
            # Получаем оригинальный текст
            # Приоритет: 1) meta.original_text (WhatsApp)  2) расшифровка encrypted_content (Telegram)
            original_text = msg.get("meta", {}).get("original_text", "")
            
            if not original_text:
                try:
                    original_text = base64.b64decode(msg["encrypted_content"]).decode('utf-8')
                except Exception:
                    original_text = ""
            
            # Пропускаем пустые
            if not original_text or not original_text.strip():
                self.stats["skipped_no_text"] += 1
                continue
            
            # Пропускаем медиа-сообщения без текста
            msg_type = msg.get("msg_type", "text")
            if msg_type in ("service",) and not original_text.strip():
                self.stats["skipped_media"] += 1
                continue
            
            # Детекция языка
            detected_lang, confidence = detect_language(original_text)
            
            if self.dry_run:
                msg["meta"]["detected_lang"] = detected_lang
                msg["meta"]["detected_lang_confidence"] = confidence
                continue
            
            # Пропускаем если уже на целевом языке
            if self.skip_if_target_lang and detected_lang == self.target_lang:
                self.stats["skipped_same_lang"] += 1
                continue
            
            # Перевод
            try:
                translated = self._translate_text(original_text)
                
                # Обновляем encrypted_content (base64 от переведённого текста)
                msg["encrypted_content"] = base64.b64encode(translated.encode()).decode('ascii')
                
                # Добавляем перевод в meta
                msg["meta"]["translation"] = {
                    "target_lang": self.target_lang,
                    "translated_text": translated,
                    "original_text": original_text,
                    "api": self.api,
                    "detected_source_lang": detected_lang,
                    "detected_source_confidence": confidence,
                }
                
                self.stats["translated"] += 1
                
            except Exception as e:
                print(f"  ⚠️ Ошибка перевода сообщения {i}: {e}", file=sys.stderr)
                msg["meta"]["translation_error"] = str(e)
                self.stats["errors"] += 1
            
            # Прогресс
            if (i + 1) % 50 == 0:
                print(f"  Обработано: {i + 1}/{len(messages)}")
        
        if self.dry_run:
            # Статистика по языкам
            lang_stats: dict[str, int] = {}
            for msg in messages:
                lang = msg.get("meta", {}).get("detected_lang", "unknown")
                lang_stats[lang] = lang_stats.get(lang, 0) + 1
            
            print("\n📊 Распределение языков:")
            for lang, count in sorted(lang_stats.items(), key=lambda x: -x[1]):
                print(f"  {lang}: {count}")
            return data
        
        # Запись результата
        out = output_path or input_path
        print(f"\nЗапись в {out}...")
        with open(out, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        
        print(f"\n✅ Перевод завершён!")
        print(f"  Файл: {out}")
        print(f"  Переведено: {self.stats['translated']}")
        print(f"  Пропущено (тот же язык): {self.stats['skipped_same_lang']}")
        print(f"  Пропущено (нет текста): {self.stats['skipped_no_text']}")
        print(f"  Ошибок: {self.stats['errors']}")
        
        # Обновляем stats в данных
        data["stats"]["translation"] = {
            "api": self.api,
            "target_lang": self.target_lang,
            "translated_count": self.stats["translated"],
            "skipped_same_lang": self.stats["skipped_same_lang"],
            "errors": self.stats["errors"],
        }
        
        return data


# ============================================================================
# CLI
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="AI авто-перевод импортированных сообщений"
    )
    parser.add_argument(
        "input",
        help="Путь к internal_export.json (результат импортера)"
    )
    parser.add_argument(
        "--output", "-o",
        default=None,
        help="Путь к выходному файлу (по умолчанию: перезаписывает входной)"
    )
    parser.add_argument(
        "--lang", "-l",
        default="en",
        help="Целевой язык (по умолчанию: en)"
    )
    parser.add_argument(
        "--api", "-a",
        default="google",
        choices=["google", "openai", "ollama"],
        help="API для перевода (по умолчанию: google)"
    )
    parser.add_argument(
        "--model", "-m",
        default=None,
        help="Модель (gpt-4o-mini для OpenAI, llama3 для Ollama)"
    )
    parser.add_argument(
        "--api-key",
        default=None,
        help="API ключ для OpenAI (или используйте OPENAI_API_KEY)"
    )
    parser.add_argument(
        "--ollama-url",
        default="http://localhost:11434",
        help="URL Ollama сервера"
    )
    parser.add_argument(
        "--detect-only",
        action="store_true",
        help="Только детекция языка без перевода"
    )
    parser.add_argument(
        "--translate-all",
        action="store_true",
        help="Переводить даже если текст уже на целевом языке"
    )
    
    args = parser.parse_args()
    
    try:
        translator = AITranslator(
            api=args.api,
            target_lang=args.lang,
            model=args.model,
            api_key=args.api_key,
            ollama_base_url=args.ollama_url,
            skip_if_target_lang=not args.translate_all,
            dry_run=args.detect_only,
        )
        
        translator.translate_export(args.input, args.output)
        
    except FileNotFoundError as e:
        print(f"❌ Ошибка: {e}", file=sys.stderr)
        sys.exit(1)
    except ImportError as e:
        print(f"❌ Ошибка: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"❌ Неизвестная ошибка: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
