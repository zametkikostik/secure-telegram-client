// messenger/src/webrtc/translator.rs
//! AI перевод звонков в реальном времени

use crate::ai::translator::AITranslator;
use crate::ai::speech_to_text::SpeechToText;
use crate::ai::text_to_speech::QwenTTS;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct CallTranslator {
    translator: AITranslator,
    stt: Option<SpeechToText>,
    tts: Option<QwenTTS>,
    source_language: String,
    target_language: String,
}

impl CallTranslator {
    pub fn new(
        translator: AITranslator,
        source: String,
        target: String,
        model_path: Option<&str>,
        tts_api_key: Option<String>,
    ) -> Self {
        let stt = model_path.and_then(|path| SpeechToText::new(path, 16000).ok());
        let tts = tts_api_key.map(|key| QwenTTS::new(key));
        
        Self {
            translator,
            stt,
            tts,
            source_language: source,
            target_language: target,
        }
    }
    
    /// Перевод аудио чанка в реальном времени
    pub async fn translate_audio_chunk(
        &self,
        audio_data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Шаг 1: Speech-to-Text
        let text = if let Some(stt) = &self.stt {
            // Для потокового распознавания
            let mut recognizer = stt.transcribe_stream()?;
            
            // Обработка аудио данных
            if let Some(partial_text) = recognizer.process(audio_data)? {
                if !partial_text.is_empty() {
                    partial_text
                } else {
                    return Ok(audio_data.to_vec()); // Возвращаем оригинал если нет текста
                }
            } else {
                return Ok(audio_data.to_vec());
            }
        } else {
            return Ok(audio_data.to_vec()); // Без STT возвращаем оригинал
        };
        
        // Шаг 2: Перевод текста
        let translated_text = self.translator
            .translate_text(&text, &self.source_language, &self.target_language)
            .await?;
        
        // Шаг 3: Text-to-Speech
        if let Some(tts) = &self.tts {
            let audio_output = tts.synthesize(&translated_text, "ru-RU", "wav").await?;
            Ok(audio_output)
        } else {
            // Без TTS возвращаем оригинальное аудио
            Ok(audio_data.to_vec())
        }
    }
    
    /// Генерация WebVTT субтитров
    pub async fn generate_subtitles(
        &self,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use crate::ai::subtitles::{SubtitleGenerator, Subtitle};
        use chrono::Duration;
        
        let translated = self.translator
            .translate_text(text, &self.source_language, &self.target_language)
            .await?;
        
        let generator = SubtitleGenerator::new();
        let subtitles = vec![Subtitle {
            start: Duration::zero(),
            end: Duration::seconds(5),
            text: translated,
        }];
        
        Ok(generator.generate_webvtt(&subtitles))
    }
    
    /// Потоковый перевод с обратным давлением
    pub async fn translate_stream(
        &self,
        mut input_rx: mpsc::Receiver<Vec<u8>>,
        output_tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use futures::StreamExt;
        
        while let Some(audio_chunk) = input_rx.recv().await {
            match self.translate_audio_chunk(&audio_chunk).await {
                Ok(translated) => {
                    if output_tx.send(translated).await.is_err() {
                        break; // Получатель закрыт
                    }
                }
                Err(e) => {
                    tracing::error!("Ошибка перевода аудио: {}", e);
                    // Продолжаем работу даже при ошибке
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_translator_creation() {
        let translator = AITranslator::new("test_key".to_string());
        let call_translator = CallTranslator::new(
            translator,
            "ru".to_string(),
            "bg".to_string(),
            None,
            None,
        );
        
        assert_eq!(call_translator.source_language, "ru");
        assert_eq!(call_translator.target_language, "bg");
    }
}
