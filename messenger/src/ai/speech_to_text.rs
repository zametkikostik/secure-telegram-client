// messenger/src/ai/speech_to_text.rs
//! Speech-to-Text на основе Vosk

use vosk::{Model, Recognizer, SampleFormat};
use std::fs::File;
use std::io::BufReader;

pub struct SpeechToText {
    model: Model,
    sample_rate: i32,
}

impl SpeechToText {
    pub fn new(model_path: &str, sample_rate: i32) -> Result<Self, Box<dyn std::error::Error>> {
        let model = Model::new(model_path)?;
        
        Ok(Self {
            model,
            sample_rate,
        })
    }
    
    pub fn transcribe(&self, audio_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let file = File::open(audio_path)?;
        let mut reader = BufReader::new(file);
        
        let mut rec = Recognizer::new(&self.model, self.sample_rate as f32)?;
        
        let mut buffer = [0u8; 4096];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            rec.accept_waveform(&buffer[..n], SampleFormat::S16LE);
        }
        
        let result = rec.final_result()?;
        let json: serde_json::Value = serde_json::from_str(&result)?;
        
        Ok(json["text"].as_str().unwrap_or("").to_string())
    }
    
    pub fn transcribe_stream(&self) -> Result<StreamingRecognizer, Box<dyn std::error::Error>> {
        let rec = Recognizer::new(&self.model, self.sample_rate as f32)?;
        Ok(StreamingRecognizer { recognizer: rec })
    }
}

pub struct StreamingRecognizer {
    recognizer: Recognizer,
}

impl StreamingRecognizer {
    pub fn process(&mut self, data: &[u8]) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.recognizer.accept_waveform(data, SampleFormat::S16LE);
        
        let partial = self.recognizer.partial_result()?;
        let json: serde_json::Value = serde_json::from_str(&partial)?;
        
        if let Some(text) = json["partial"].as_str() {
            if !text.is_empty() {
                return Ok(Some(text.to_string()));
            }
        }
        
        Ok(None)
    }
    
    pub fn finish(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.recognizer.final_result()?;
        let json: serde_json::Value = serde_json::from_str(&result)?;
        Ok(json["text"].as_str().unwrap_or("").to_string())
    }
}
