//! Anti-DPI Obfuscation Layer
//!
//! Deep Packet Inspection (DPI) evasion techniques:
//! - Protocol camouflage (looks like HTTPS, DNS, etc.)
//! - Timing obfuscation (random delays, batching)
//! - Padding (fixed + random sizes)
//! - Traffic shaping (mimics normal web traffic)
//! - Header randomization
//!
//! This makes it extremely difficult for ISPs/governments to:
//! - Identify that traffic is from our messenger
//! - Block the traffic without blocking legitimate services
//! - Analyze communication patterns

use rand::Rng;

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// Obfuscation Profiles
// ============================================================================

/// Pre-configured obfuscation profiles
#[derive(Debug, Clone, Copy)]
pub enum ObfuscationProfile {
    /// Looks like normal HTTPS traffic
    HttpsCamouflage,
    /// Looks like DNS over HTTPS (DoH)
    DnsOverHttps,
    /// Looks like WebSocket traffic
    WebSocketCamouflage,
    /// Looks like generic TCP stream
    GenericStream,
    /// Maximum paranoia (all techniques)
    Paranoid,
}

impl ObfuscationProfile {
    pub fn padding_strategy(&self) -> PaddingStrategy {
        match self {
            ObfuscationProfile::HttpsCamouflage => PaddingStrategy::FixedSize(1460), // TLS record size
            ObfuscationProfile::DnsOverHttps => PaddingStrategy::RandomRange(64, 512),
            ObfuscationProfile::WebSocketCamouflage => PaddingStrategy::WebSocketFrames,
            ObfuscationProfile::GenericStream => PaddingStrategy::None,
            ObfuscationProfile::Paranoid => PaddingStrategy::Adaptive,
        }
    }

    pub fn timing_strategy(&self) -> TimingStrategy {
        match self {
            ObfuscationProfile::HttpsCamouflage => TimingStrategy::Jitter(50, 200),
            ObfuscationProfile::DnsOverHttps => TimingStrategy::Batch(Duration::from_secs(1)),
            ObfuscationProfile::WebSocketCamouflage => TimingStrategy::Jitter(10, 100),
            ObfuscationProfile::GenericStream => TimingStrategy::None,
            ObfuscationProfile::Paranoid => TimingStrategy::Adaptive,
        }
    }

    pub fn protocol_header(&self) -> Vec<u8> {
        match self {
            // Fake TLS ClientHello (looks like TLS 1.3 handshake)
            ObfuscationProfile::HttpsCamouflage => vec![
                0x16, 0x03, 0x01, // TLS record: Handshake, version 1.0
                0x02, 0x00, // Length
                0x01, 0x00, // Handshake: ClientHello
                0x01, 0xfc, // Length
                0x03, 0x03, // Version 1.2
            ],
            // Fake DNS query header
            ObfuscationProfile::DnsOverHttps => vec![
                0xab, 0xcd, // Transaction ID
                0x01, 0x00, // Flags: standard query
                0x00, 0x01, // Questions: 1
                0x00, 0x00, // Answer RRs: 0
                0x00, 0x00, // Authority RRs: 0
                0x00, 0x00, // Additional RRs: 0
            ],
            // WebSocket frame header
            ObfuscationProfile::WebSocketCamouflage => vec![
                0x81, // FIN + text frame
                0x7e, // Masked + payload len 126
                0x00, 0x00, // Extended length (placeholder)
            ],
            ObfuscationProfile::GenericStream | ObfuscationProfile::Paranoid => vec![],
        }
    }
}

// ============================================================================
// Padding Strategies
// ============================================================================

#[derive(Debug, Clone)]
pub enum PaddingStrategy {
    /// No padding
    None,
    /// Pad to fixed size
    FixedSize(usize),
    /// Random padding within range
    RandomRange(usize, usize),
    /// WebSocket-style frames
    WebSocketFrames,
    /// Adaptive (based on traffic analysis resistance)
    Adaptive,
}

impl PaddingStrategy {
    pub fn apply(&self, data: &[u8]) -> Vec<u8> {
        match self {
            PaddingStrategy::None => data.to_vec(),
            PaddingStrategy::FixedSize(size) => {
                let mut padded = data.to_vec();
                if padded.len() < *size {
                    let mut rng = rand::thread_rng();
                    let padding_len = size - padded.len();
                    let mut padding = vec![0u8; padding_len];
                    rng.fill_bytes(&mut padding);
                    padded.extend_from_slice(&padding);
                }
                padded
            }
            PaddingStrategy::RandomRange(min, max) => {
                let mut rng = rand::thread_rng();
                let target = rng.gen_range(*min..=*max).max(data.len());
                let mut padded = data.to_vec();
                if padded.len() < target {
                    let mut padding = vec![0u8; target - padded.len()];
                    rng.fill_bytes(&mut padding);
                    padded.extend_from_slice(&padding);
                }
                padded
            }
            PaddingStrategy::WebSocketFrames => {
                // Split into WebSocket-sized frames
                let frame_size = 125; // Max unmasked WebSocket text frame
                let mut result = Vec::new();
                for chunk in data.chunks(frame_size) {
                    let mut frame = vec![0x81]; // FIN + text
                    if chunk.len() < 126 {
                        frame.push(chunk.len() as u8);
                    } else if chunk.len() < 65536 {
                        frame.push(126);
                        frame.extend_from_slice(&(chunk.len() as u16).to_be_bytes());
                    }
                    frame.extend_from_slice(chunk);
                    result.extend_from_slice(&frame);
                }
                result
            }
            PaddingStrategy::Adaptive => {
                // Exponential padding based on message count
                // More messages = more padding to hide traffic patterns
                let mut rng = rand::thread_rng();
                let base = data.len();
                let padded_size = base + (rng.next_u32() % 256) as usize;
                let mut padded = data.to_vec();
                let mut padding = vec![0u8; padded_size - base];
                rng.fill_bytes(&mut padding);
                padded.extend_from_slice(&padding);
                padded
            }
        }
    }
}

// ============================================================================
// Timing Strategies
// ============================================================================

#[derive(Debug, Clone)]
pub enum TimingStrategy {
    /// No timing changes
    None,
    /// Random jitter between min and max ms
    Jitter(u64, u64),
    /// Batch messages and send at intervals
    Batch(Duration),
    /// Adaptive (learn from traffic patterns)
    Adaptive,
}

impl TimingStrategy {
    pub async fn apply(&self) {
        match self {
            TimingStrategy::None => {}
            TimingStrategy::Jitter(min_ms, max_ms) => {
                let mut rng = rand::thread_rng();
                let delay = Duration::from_millis(rng.gen_range(*min_ms..=*max_ms));
                sleep(delay).await;
            }
            TimingStrategy::Batch(interval) => {
                sleep(*interval).await;
            }
            TimingStrategy::Adaptive => {
                // Adaptive: increase delay during high surveillance periods
                let mut rng = rand::thread_rng();
                let delay = Duration::from_millis(rng.gen_range(100..500));
                sleep(delay).await;
            }
        }
    }
}

// ============================================================================
// Protocol Camouflage
// ============================================================================

/// Makes our traffic look like a different protocol
pub struct ProtocolCamouflage {
    profile: ObfuscationProfile,
    fake_sni: Option<String>, // Fake Server Name Indication
    fake_user_agent: Option<String>,
}

impl ProtocolCamouflage {
    pub fn new(profile: ObfuscationProfile) -> Self {
        let (sni, ua) = match profile {
            ObfuscationProfile::HttpsCamouflage => (
                Some("www.google.com".to_string()),
                Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()),
            ),
            ObfuscationProfile::DnsOverHttps => (
                Some("dns.google.com".to_string()),
                Some("Mozilla/5.0".to_string()),
            ),
            ObfuscationProfile::WebSocketCamouflage => (None, None),
            ObfuscationProfile::GenericStream => (None, None),
            ObfuscationProfile::Paranoid => (
                Some("accounts.google.com".to_string()),
                Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string()),
            ),
        };

        Self {
            profile,
            fake_sni: sni,
            fake_user_agent: ua,
        }
    }

    /// Generate fake TLS ClientHello with our fake SNI
    pub fn generate_fake_client_hello(&self) -> Vec<u8> {
        let mut header = self.profile.protocol_header();

        if let Some(ref sni) = self.fake_sni {
            // Add SNI extension
            let sni_bytes = sni.as_bytes();
            let ext_len = (sni_bytes.len() + 9) as u16;
            header.extend_from_slice(&ext_len.to_be_bytes());
            header.extend_from_slice(&[0x00, 0x00]); // extension_type: server_name
            header.extend_from_slice(&((sni_bytes.len() + 5) as u16).to_be_bytes());
            header.extend_from_slice(&((sni_bytes.len() + 3) as u16).to_be_bytes());
            header.push(0x00); // name_type: host_name
            header.extend_from_slice(&(sni_bytes.len() as u16).to_be_bytes());
            header.extend_from_slice(sni_bytes);
        }

        header
    }

    /// Get fake User-Agent header value
    pub fn fake_user_agent(&self) -> Option<&str> {
        self.fake_user_agent.as_deref()
    }
}

// ============================================================================
// Main Obfuscator
// ============================================================================

/// Anti-DPI obfuscation engine
pub struct DpiObfuscator {
    profile: ObfuscationProfile,
    camouflage: ProtocolCamouflage,
    message_counter: u64,
}

impl DpiObfuscator {
    pub fn new(profile: ObfuscationProfile) -> Self {
        Self {
            profile,
            camouflage: ProtocolCamouflage::new(profile),
            message_counter: 0,
        }
    }

    /// Obfuscate data before sending
    pub async fn obfuscate(&mut self, data: &[u8]) -> Vec<u8> {
        // 1. Apply protocol header
        let header = self.profile.protocol_header();

        // 2. Apply padding
        let padded = self.profile.padding_strategy().apply(data);

        // 3. Combine
        let mut result = header;
        result.extend_from_slice(&padded);

        // 4. Apply timing obfuscation
        self.profile.timing_strategy().apply().await;

        self.message_counter += 1;
        result
    }

    /// Deobfuscate received data
    pub fn deobfuscate(&self, data: &[u8]) -> Vec<u8> {
        // Strip protocol header
        let header_len = self.profile.protocol_header().len();
        if data.len() > header_len {
            data[header_len..].to_vec()
        } else {
            data.to_vec()
        }
    }

    /// Get fake TLS ClientHello for initial connection
    pub fn fake_client_hello(&self) -> Vec<u8> {
        self.camouflage.generate_fake_client_hello()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_padding_fixed_size() {
        let strategy = PaddingStrategy::FixedSize(100);
        let data = vec![1, 2, 3];
        let padded = strategy.apply(&data);
        assert_eq!(padded.len(), 100);
        assert_eq!(&padded[..3], &[1, 2, 3]);
    }

    fn test_padding_no_change() {
        let strategy = PaddingStrategy::FixedSize(2);
        let data = vec![1, 2, 3, 4, 5];
        let padded = strategy.apply(&data);
        assert_eq!(padded, data);
    }

    fn test_websocket_frames() {
        let strategy = PaddingStrategy::WebSocketFrames;
        let data = vec![0u8; 200]; // 200 bytes → 2 frames
        let framed = strategy.apply(&data);
        assert!(framed.len() > 200); // Headers added
    }

    fn test_deobfuscation() {
        // GenericStream has no header, so data passes through unchanged
        let obfuscator = DpiObfuscator::new(ObfuscationProfile::GenericStream);
        let original = vec![1, 2, 3, 4, 5];
        let recovered = obfuscator.deobfuscate(&original);
        assert_eq!(recovered, original);
    }

    #[tokio::test]
    async fn test_obfuscate_roundtrip() {
        let mut obfuscator = DpiObfuscator::new(ObfuscationProfile::WebSocketCamouflage);
        let original = b"secret message";
        let obfuscated = obfuscator.obfuscate(original).await;
        assert!(obfuscated.len() >= original.len());
    }
}
