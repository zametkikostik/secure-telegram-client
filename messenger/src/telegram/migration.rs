// messenger/src/telegram/migration.rs
//! Миграция данных из Telegram/WhatsApp в Liberty Reach

use crate::telegram::importer::{TelegramImporter, TelegramExport};
use crate::ai::translator::AITranslator;
use std::path::Path;

pub struct MigrationTool {
    translator: AITranslator,
    auto_translate: bool,
}

impl MigrationTool {
    pub fn new(translator: AITranslator, auto_translate: bool) -> Self {
        Self {
            translator,
            auto_translate,
        }
    }
    
    pub async fn migrate_telegram(
        &self,
        export_path: &str,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let importer = TelegramImporter::new(export_path.to_string());
        let export = importer.load_export()?;
        
        println!("Импортировано {} сообщений из {}", export.messages.len(), export.name);
        
        // Конвертация в формат Liberty Reach
        let mut lr_messages = Vec::new();
        
        for msg in &export.messages {
            let text = match &msg.text {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                }
                _ => String::new(),
            };
            
            // Авто-перевод при импорте
            let translated_text = if self.auto_translate && !text.is_empty() {
                self.translator.translate_text(&text, "auto", "bg").await
                    .unwrap_or_else(|_| text.clone())
            } else {
                text
            };
            
            lr_messages.push(serde_json::json!({
                "id": msg.id,
                "date": msg.date,
                "from": msg.from,
                "text": translated_text,
                "source": "telegram"
            }));
        }
        
        // Сохранение в формат Liberty Reach
        let output = serde_json::json!({
            "name": export.name,
            "messages": lr_messages,
            "migrated_at": chrono::Utc::now().to_rfc3339()
        });
        
        std::fs::write(output_path, output.to_string())?;
        println!("Миграция завершена: {}", output_path);
        
        Ok(())
    }
    
    pub async fn migrate_whatsapp(
        &self,
        export_path: &str,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Парсинг WhatsApp TXT экспорта
        let content = std::fs::read_to_string(export_path)?;
        let mut lr_messages = Vec::new();
        let mut id: u64 = 0;
        
        for line in content.lines() {
            // Формат WhatsApp: DD/MM/YYYY, HH:MM - Name: Message
            if let Some(parts) = line.splitn(2, " - ").nth(1) {
                if let Some((name, text)) = parts.splitn(2, ": ").collect::<Vec<_>>().split_first() {
                    id += 1;
                    
                    let translated_text = if self.auto_translate && !text.is_empty() {
                        self.translator.translate_text(text, "auto", "bg").await
                            .unwrap_or_else(|_| text.to_string())
                    } else {
                        text.to_string()
                    };
                    
                    lr_messages.push(serde_json::json!({
                        "id": id,
                        "from": name,
                        "text": translated_text,
                        "source": "whatsapp"
                    }));
                }
            }
        }
        
        let output = serde_json::json!({
            "messages": lr_messages,
            "migrated_at": chrono::Utc::now().to_rfc3339()
        });
        
        std::fs::write(output_path, output.to_string())?;
        println!("WhatsApp миграция завершена: {}", output_path);
        
        Ok(())
    }
}
