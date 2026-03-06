// messenger/src/telegram/importer.rs
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
pub struct TelegramExport {
    pub name: String,
    pub messages: Vec<TelegramMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
    pub id: u64,
    pub date: String,
    pub from: String,
    pub text: serde_json::Value,
}

pub struct TelegramImporter {
    export_path: String,
}

impl TelegramImporter {
    pub fn new(export_path: String) -> Self {
        Self { export_path }
    }
    
    pub fn load_export(&self) -> Result<TelegramExport, Box<dyn std::error::Error>> {
        let file = File::open(&self.export_path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}
