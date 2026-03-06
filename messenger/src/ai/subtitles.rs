// messenger/src/ai/subtitles.rs
//! Генерация субтитров (WebVTT формат)

use chrono::{Duration, NaiveTime};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Subtitle {
    pub start: Duration,
    pub end: Duration,
    pub text: String,
}

pub struct SubtitleGenerator;

impl SubtitleGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_webvtt(&self, subtitles: &[Subtitle]) -> String {
        let mut output = String::from("WEBVTT\n\n");
        
        for (i, subtitle) in subtitles.iter().enumerate() {
            let start = self.duration_to_webvtt(subtitle.start);
            let end = self.duration_to_webvtt(subtitle.end);
            
            writeln!(output, "{}", i + 1).unwrap();
            writeln!(output, "{} --> {}", start, end).unwrap();
            writeln!(output, "{}\n", subtitle.text).unwrap();
        }
        
        output
    }
    
    fn duration_to_webvtt(&self, duration: Duration) -> String {
        let total_secs = duration.num_seconds();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let millis = (duration.num_milliseconds() % 1000) as u32;
        
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
    }
    
    pub fn from_transcript(
        &self,
        transcript: &str,
        words_per_subtitle: usize,
    ) -> Vec<Subtitle> {
        let words: Vec<&str> = transcript.split_whitespace().collect();
        let mut subtitles = Vec::new();
        
        let mut current_time = Duration::zero();
        let avg_word_duration = Duration::milliseconds(300); // ~200 слов/минуту
        
        for chunk in words.chunks(words_per_subtitle) {
            let text = chunk.join(" ");
            let duration = Duration::milliseconds(chunk.len() as i64 * 300);
            
            subtitles.push(Subtitle {
                start: current_time,
                end: current_time + duration,
                text,
            });
            
            current_time += duration + Duration::milliseconds(100); // Пауза между субтитрами
        }
        
        subtitles
    }
}
