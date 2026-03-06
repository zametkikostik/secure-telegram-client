// messenger/src/telegram/whatsapp_importer.rs
//! Импорт из WhatsApp TXT

use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
pub struct WhatsAppMessage {
    pub id: u64,
    pub date: String,
    pub from: String,
    pub text: String,
}

pub struct WhatsAppImporter;

impl WhatsAppImporter {
    pub fn parse_txt(path: &str) -> Result<Vec<WhatsAppMessage>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut messages = Vec::new();
        let mut id: u64 = 0;
        
        for line in reader.lines() {
            let line = line?;
            
            // Формат: DD/MM/YYYY, HH:MM - Name: Message
            if let Some(rest) = line.splitn(2, " - ").nth(1) {
                if let Some((name, text)) = rest.splitn(2, ": ").collect::<Vec<_>>().split_first() {
                    id += 1;
                    
                    messages.push(WhatsAppMessage {
                        id,
                        date: String::new(), // Нужно распарсить дату из строки
                        from: name.to_string(),
                        text: text.to_string(),
                    });
                }
            }
        }
        
        Ok(messages)
    }
}
