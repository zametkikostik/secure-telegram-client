//! Wi-Fi LAN Transport — Real Working Implementation
//!
//! Pure Rust — works on Linux, macOS, Windows:
//! - UDP multicast/broadcast for peer discovery
//! - TCP for message exchange
//! - No OS-specific dependencies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{debug, info, warn, error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WifiLanError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Timeout")]
    Timeout,
}

pub type WifiLanResult<T> = Result<T, WifiLanError>;

const MULTICAST_ADDR: &str = "239.255.0.1";
const DISCOVERY_PORT: u16 = 9877;
const DISCOVERY_INTERVAL: Duration = Duration::from_secs(5);
const PEER_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryPacket {
    peer_id: String,
    display_name: String,
    tcp_port: u16,
}

#[derive(Debug, Clone)]
pub struct LanPeer {
    pub peer_id: String,
    pub display_name: String,
    pub ip_address: String,
    pub tcp_port: u16,
    pub last_seen: Instant,
    pub is_online: bool,
}

pub type MessageCallback = Arc<dyn Fn(Vec<u8>, String) + Send + Sync>;
pub type PeerCallback = Arc<dyn Fn(LanPeer) + Send + Sync>;

pub struct WifiLanTransport {
    peer_id: String,
    display_name: String,
    tcp_port: u16,
    peers: Arc<RwLock<HashMap<String, LanPeer>>>,
    message_callback: Arc<Mutex<Option<MessageCallback>>>,
    peer_callback: Arc<Mutex<Option<PeerCallback>>>,
    running: Arc<RwLock<bool>>,
}

async fn write_frame(stream: &mut tokio::net::TcpStream, data: &[u8]) -> WifiLanResult<()> {
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes()).await.map_err(|e| WifiLanError::Network(e.to_string()))?;
    stream.write_all(data).await.map_err(|e| WifiLanError::Network(e.to_string()))?;
    stream.flush().await.map_err(|e| WifiLanError::Network(e.to_string()))?;
    Ok(())
}

async fn read_frame(stream: &mut tokio::net::TcpStream) -> WifiLanResult<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.map_err(|e| WifiLanError::Network(e.to_string()))?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 10 * 1024 * 1024 { return Err(WifiLanError::Network("Too large".into())); }
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data).await.map_err(|e| WifiLanError::Network(e.to_string()))?;
    Ok(data)
}

impl WifiLanTransport {
    pub fn new(peer_id: &str, tcp_port: u16) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            display_name: "Secure Messenger".to_string(),
            tcp_port,
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_callback: Arc::new(Mutex::new(None)),
            peer_callback: Arc::new(Mutex::new(None)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn on_message<F>(&self, cb: F) where F: Fn(Vec<u8>, String) + Send + Sync + 'static {
        *self.message_callback.lock().await = Some(Arc::new(cb));
    }

    pub async fn on_peer_discovered<F>(&self, cb: F) where F: Fn(LanPeer) + Send + Sync + 'static {
        *self.peer_callback.lock().await = Some(Arc::new(cb));
    }

    pub async fn start_discovery(&self) -> WifiLanResult<()> {
        *self.running.write().await = true;
        let peer_id = self.peer_id.clone();
        let display_name = self.display_name.clone();
        let tcp_port = self.tcp_port;
        let peers = self.peers.clone();
        let peer_cb = self.peer_callback.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let recv_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DISCOVERY_PORT);
            let socket = match std::net::UdpSocket::bind(recv_addr) {
                Ok(s) => s, Err(e) => { warn!("UDP bind failed: {}", e); return; }
            };
            socket.set_nonblocking(true).ok();
            let _ = socket.join_multicast_v4(&Ipv4Addr::new(239,255,0,1), &Ipv4Addr::UNSPECIFIED);
            info!("LAN discovery started");
            let mut last_bc = Instant::now();

            while *running.read().await {
                if last_bc.elapsed() >= DISCOVERY_INTERVAL {
                    Self::broadcast(&peer_id, &display_name, tcp_port);
                    last_bc = Instant::now();
                }
                let mut buf = [0u8; 1024];
                match socket.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        if let Ok(pkt) = serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                            if pkt.peer_id == *peer_id { continue; }
                            let peer = LanPeer {
                                peer_id: pkt.peer_id.clone(), display_name: pkt.display_name.clone(),
                                ip_address: src.ip().to_string(), tcp_port: pkt.tcp_port,
                                last_seen: Instant::now(), is_online: true,
                            };
                            if let Some(cb) = peer_cb.lock().await.as_ref() { cb(peer.clone()); }
                            peers.write().await.insert(pkt.peer_id, peer);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => { debug!("Discovery error: {}", e); }
                }
                // Cleanup stale
                {
                    let mut pm = peers.write().await;
                    pm.retain(|_, p| { let ok = p.last_seen.elapsed() < PEER_TIMEOUT; p.is_online = ok; ok });
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
        Ok(())
    }

    fn broadcast(peer_id: &str, display_name: &str, tcp_port: u16) {
        let pkt = DiscoveryPacket { peer_id: peer_id.into(), display_name: display_name.into(), tcp_port };
        if let Ok(data) = serde_json::to_vec(&pkt) {
            if let Ok(sock) = std::net::UdpSocket::bind("0.0.0.0:0") {
                sock.set_broadcast(true).ok();
                let mc: SocketAddr = format!("{}:{}", MULTICAST_ADDR, DISCOVERY_PORT).parse().unwrap();
                let _ = sock.send_to(&data, mc);
                let bc: SocketAddr = format!("255.255.255.255:{}", DISCOVERY_PORT).parse().unwrap();
                let _ = sock.send_to(&data, bc);
            }
        }
    }

    pub async fn start_server(&self) -> WifiLanResult<()> {
        *self.running.write().await = true;
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), self.tcp_port);
        let listener = TcpListener::bind(addr).await.map_err(|e| WifiLanError::Network(e.to_string()))?;
        let msg_cb = self.message_callback.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("LAN server on {}", addr);
            while *running.read().await {
                if let Ok((mut stream, peer_addr)) = listener.accept().await {
                    let cb = msg_cb.clone();
                    let peer_ip = peer_addr.ip().to_string();
                    tokio::spawn(async move {
                        loop {
                            match read_frame(&mut stream).await {
                                Ok(data) => {
                                    if let Some(callback) = cb.lock().await.as_ref() {
                                        callback(data, peer_ip.clone());
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }
            }
        });
        Ok(())
    }

    pub async fn send_message(&self, peer_ip: &str, peer_port: u16, data: &[u8]) -> WifiLanResult<()> {
        let addr_str = format!("{}:{}", peer_ip, peer_port);
        let addr: SocketAddr = addr_str.parse()
            .map_err(|e: std::net::AddrParseError| WifiLanError::Network(e.to_string()))?;
        let stream = tokio::net::TcpStream::connect(addr).await
            .map_err(|e: std::io::Error| WifiLanError::Network(e.to_string()))?;
        let mut stream = stream;
        write_frame(&mut stream, data).await?;
        debug!("Sent {} bytes to {}", data.len(), addr_str);
        Ok(())
    }

    pub async fn broadcast_message(&self, data: &[u8]) -> WifiLanResult<usize> {
        let peers = self.peers.read().await;
        let mut count = 0;
        for p in peers.values() {
            if p.is_online && self.send_message(&p.ip_address, p.tcp_port, data).await.is_ok() { count += 1; }
        }
        Ok(count)
    }

    pub async fn get_peers(&self) -> Vec<LanPeer> { self.peers.read().await.values().cloned().collect() }
    pub async fn get_online_peers(&self) -> Vec<LanPeer> { self.peers.read().await.values().filter(|p| p.is_online).cloned().collect() }
    pub async fn stop(&self) { *self.running.write().await = false; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_wifi_lan_server_and_client() {
        // Use a high port to avoid conflicts
        let server_port = 31000 + (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u16 % 1000);

        let received = Arc::new(AtomicUsize::new(0));
        let server = WifiLanTransport::new("server-peer", server_port);
        let recv_clone = received.clone();
        server.on_message(move |_data, _addr| {
            recv_clone.fetch_add(1, Ordering::SeqCst);
        }).await;

        // Start server
        let server_result = server.start_server().await;
        assert!(server_result.is_ok(), "Server failed to start: {:?}", server_result);

        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Create client and send
        let client = WifiLanTransport::new("client-peer", 0);
        let result = client.send_message("127.0.0.1", server_port, b"hello").await;
        assert!(result.is_ok(), "Send failed: {:?}", result);

        // Wait for message processing
        tokio::time::sleep(Duration::from_millis(500)).await;

        let count = received.load(Ordering::SeqCst);
        assert!(count >= 1, "Expected >= 1 message, got {}", count);
        server.stop().await;
    }

    #[tokio::test]
    async fn test_frame_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let client_task = tokio::spawn(async move {
            let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            write_frame(&mut s, &[1,2,3,4,5]).await.unwrap();
        });

        let server_task = tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let data = read_frame(&mut s).await.unwrap();
            assert_eq!(data, vec![1,2,3,4,5]);
        });

        client_task.await.unwrap();
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_peer_discovery() {
        let t = WifiLanTransport::new("test", 29999);
        assert!(t.get_peers().await.is_empty());
        assert!(t.get_online_peers().await.is_empty());
    }
}
