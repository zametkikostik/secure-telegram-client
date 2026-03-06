// messenger/src/webrtc/walkie_talkie.rs
//! Рация (Push-to-Talk, бесплатно)

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, SampleRate};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct WalkieTalkie {
    is_transmitting: Arc<AtomicBool>,
    host: Host,
    device: Device,
}

impl WalkieTalkie {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("Нет доступного аудио устройства")?;
        
        Ok(Self {
            is_transmitting: Arc::new(AtomicBool::new(false)),
            host,
            device,
        })
    }
    
    pub fn start_transmit(&self) {
        self.is_transmitting.store(true, Ordering::SeqCst);
        println!("🎤 Передача началась...");
    }
    
    pub fn stop_transmit(&self) {
        self.is_transmitting.store(false, Ordering::SeqCst);
        println!("🔇 Передача остановлена");
    }
    
    pub fn is_transmitting(&self) -> bool {
        self.is_transmitting.load(Ordering::SeqCst)
    }
    
    pub fn get_audio_config(&self) -> cpal::StreamConfig {
        self.device.default_output_config().unwrap().config()
    }
}
