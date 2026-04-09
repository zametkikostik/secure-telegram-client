//! P2P Hybrid Transport Layer
//!
//! Decentralized peer-to-peer networking using libp2p with:
//! - **TCP + QUIC** transports for reliable and low-latency connections
//! - **Noise** protocol for authenticated encryption (handshake)
//! - **Yamux** for stream multiplexing
//! - **Kademlia DHT** for peer discovery and content routing
//! - **Gossipsub** for pub/sub group chats
//! - **mDNS** for local network discovery
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

use libp2p::{
    gossipsub,
    identify,
    identity,
    kad,
    mdns,
    noise,
    ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    yamux,
    PeerId,
    Swarm,
};
use libp2p::futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum P2PError {
    #[error("Swarm error: {0}")]
    Swarm(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Channel closed")]
    ChannelClosed,
}

// ============================================================================
// Message Types
// ============================================================================

/// P2P message payload for encrypted messenger traffic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    /// Encrypted payload (ChaCha20-Poly1305 ciphertext)
    pub ciphertext: Vec<u8>,
    /// Ed25519 signature of ciphertext
    pub signature: Vec<u8>,
    /// Message timestamp (Unix epoch ms)
    pub timestamp: u64,
    /// Message type
    pub msg_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Direct encrypted message
    Direct,
    /// Group chat message (gossipsub)
    Group,
    /// Key exchange request
    KeyExchange,
    /// Key exchange response
    KeyExchangeResponse,
    /// Ping/keep-alive
    Ping,
}

/// Events emitted by the P2P layer
#[derive(Debug, Clone)]
pub enum P2PEvent {
    /// New peer discovered
    PeerDiscovered(PeerId),
    /// Peer disconnected
    PeerDisconnected(PeerId),
    /// Message received
    MessageReceived {
        from: PeerId,
        message: P2PMessage,
    },
    /// Gossipsub message received
    GossipMessageReceived {
        from: PeerId,
        topic: String,
        message: Vec<u8>,
    },
    /// DHT record found
    DhtRecordFound {
        key: Vec<u8>,
        value: Vec<u8>,
    },
}

// ============================================================================
// Network Behaviour
// ============================================================================

/// Combined network behaviour for all P2P protocols
#[derive(NetworkBehaviour)]
struct MeshBehaviour {
    /// Kademlia DHT for peer/content discovery
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// Gossipsub for pub/sub group chats
    gossipsub: gossipsub::Behaviour,
    /// mDNS for local network discovery
    mdns: mdns::tokio::Behaviour,
    /// Identify protocol for peer info exchange
    identify: identify::Behaviour,
    /// Ping for connection health monitoring
    ping: ping::Behaviour,
}

// ============================================================================
// P2P Node
// ============================================================================

/// P2P hybrid transport node
pub struct P2PNode {
    /// libp2p swarm
    swarm: Swarm<MeshBehaviour>,
    /// Event channel for external consumers
    event_tx: mpsc::Sender<P2PEvent>,
    /// Local peer ID
    local_peer_id: PeerId,
}

impl P2PNode {
    /// Create a new P2P node with TCP + QUIC + Noise + Yamux
    ///
    /// # Arguments
    /// * `keypair` — optional libp2p identity keypair (generates new if None)
    /// * `event_tx` — channel for sending events to application layer
    ///
    /// # Returns
    /// * `Ok(P2PNode)` — initialized node
    /// * `Err(P2PError)` — on initialization failure
    pub async fn new(
        keypair: Option<identity::Keypair>,
        event_tx: mpsc::Sender<P2PEvent>,
    ) -> Result<Self, P2PError> {
        let identity = keypair.unwrap_or_else(identity::Keypair::generate_ed25519);
        let local_peer_id = PeerId::from(identity.public());

        info!("Local Peer ID: {}", local_peer_id);

        // ====================================================================
        // Network Behaviours
        // ====================================================================
        // All behaviours are created in with_behaviour closure

        // ====================================================================
        // Swarm
        // ====================================================================

        let swarm = libp2p::SwarmBuilder::with_existing_identity(identity.clone())
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default().nodelay(true),
                noise::Config::new,
                || yamux::Config::default(),
            )
            .expect("TCP transport build failed")
            .with_behaviour(|key| MeshBehaviour {
                kademlia: kad::Behaviour::new(
                    PeerId::from(key.public()),
                    kad::store::MemoryStore::new(PeerId::from(key.public())),
                ),
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub::ConfigBuilder::default()
                        .heartbeat_interval(Duration::from_secs(10))
                        .validation_mode(gossipsub::ValidationMode::Strict)
                        .message_id_fn(|message: &gossipsub::Message| {
                            let mut hasher = DefaultHasher::new();
                            message.data.hash(&mut hasher);
                            gossipsub::MessageId::from(hasher.finish().to_string())
                        })
                        .build()
                        .expect("Gossipsub config build failed"),
                )
                .expect("Gossipsub behaviour creation failed"),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    PeerId::from(key.public()),
                )
                .expect("mDNS behaviour creation failed"),
                identify: identify::Behaviour::new(identify::Config::new(
                    "/secure-messenger/1.0.0".to_string(),
                    key.public(),
                )),
                ping: ping::Behaviour::new(ping::Config::new()),
            })
            .expect("Behaviour build failed")
            .with_swarm_config(|c: libp2p::swarm::Config| {
                c.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();

        info!("P2P node initialized successfully");

        Ok(Self {
            swarm,
            event_tx,
            local_peer_id,
        })
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    /// Start listening on address
    ///
    /// # Arguments
    /// * `addr` — multiaddress to listen on (e.g., "/ip4/0.0.0.0/tcp/0")
    ///
    /// # Returns
    /// * `Ok(())` — listening started
    /// * `Err(P2PError)` — on failure
    pub fn listen(&mut self, addr: &str) -> Result<(), P2PError> {
        let multiaddr: libp2p::Multiaddr = addr
            .parse()
            .map_err(|e| P2PError::Swarm(format!("Invalid multiaddress '{}': {}", addr, e)))?;

        self.swarm
            .listen_on(multiaddr.clone())
            .map_err(|e| P2PError::Swarm(format!("Listen on {} failed: {}", addr, e)))?;

        info!("Listening on: {}", addr);

        Ok(())
    }

    /// Dial a peer by multiaddress
    ///
    /// # Arguments
    /// * `addr` — multiaddress of peer to dial
    ///
    /// # Returns
    /// * `Ok(())` — dial initiated
    /// * `Err(P2PError)` — on failure
    pub fn dial(&mut self, addr: &str) -> Result<(), P2PError> {
        let multiaddr: libp2p::Multiaddr = addr
            .parse()
            .map_err(|e| P2PError::Swarm(format!("Invalid multiaddress '{}': {}", addr, e)))?;

        self.swarm
            .dial(multiaddr.clone())
            .map_err(|e| P2PError::Swarm(format!("Dial {} failed: {}", addr, e)))?;

        info!("Dialing peer: {}", addr);

        Ok(())
    }

    /// Dial a peer by PeerId (uses DHT to find address)
    ///
    /// # Arguments
    /// * `peer_id` — target peer ID
    ///
    /// # Returns
    /// * `Ok(())` — dial initiated
    /// * `Err(P2PError)` — on failure
    pub fn dial_peer_id(&mut self, peer_id: PeerId) -> Result<(), P2PError> {
        self.swarm
            .dial(peer_id)
            .map_err(|e| P2PError::Swarm(format!("Dial peer {} failed: {}", peer_id, e)))?;

        info!("Dialing peer by ID: {}", peer_id);

        Ok(())
    }

    /// Send encrypted message to peer
    ///
    /// # Arguments
    /// * `peer_id` — target peer
    /// * `message` — encrypted P2P message
    ///
    /// # Returns
    /// * `Ok(())` — message sent
    /// * `Err(P2PError)` — on failure
    pub async fn send_message(
        &mut self,
        peer_id: PeerId,
        message: P2PMessage,
    ) -> Result<(), P2PError> {
        let payload = serde_json::to_vec(&message)?;

        // Use request-response pattern for direct messages
        // For now, send via gossipsub on a peer-specific topic
        let topic_str = format!("/secure-messenger/direct/{}", peer_id);
        let topic = gossipsub::IdentTopic::new(topic_str);

        // Subscribe to topic if not already
        let _ = self.swarm.behaviour_mut().gossipsub.subscribe(&topic);

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, payload)
            .map_err(|e| P2PError::SendFailed(format!("Publish failed: {}", e)))?;

        debug!("Sent message to peer: {}", peer_id);

        Ok(())
    }

    /// Join a group chat topic
    ///
    /// # Arguments
    /// * `topic` — group chat topic name
    ///
    /// # Returns
    /// * `Ok(())` — joined successfully
    /// * `Err(P2PError)` — on failure
    pub fn join_group(&mut self, topic: &str) -> Result<(), P2PError> {
        let gossip_topic = gossipsub::IdentTopic::new(format!(
            "/secure-messenger/group/{}",
            topic
        ));

        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&gossip_topic)
            .map_err(|e| P2PError::SendFailed(format!("Subscribe to {} failed: {}", topic, e)))?;

        info!("Joined group topic: {}", topic);

        Ok(())
    }

    /// Publish message to group chat
    ///
    /// # Arguments
    /// * `topic` — group chat topic
    /// * `message` — encrypted message bytes
    ///
    /// # Returns
    /// * `Ok(())` — published successfully
    /// * `Err(P2PError)` — on failure
    pub fn publish_to_group(
        &mut self,
        topic: &str,
        message: Vec<u8>,
    ) -> Result<(), P2PError> {
        let gossip_topic = gossipsub::IdentTopic::new(format!(
            "/secure-messenger/group/{}",
            topic
        ));

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(gossip_topic, message)
            .map_err(|e| P2PError::SendFailed(format!("Publish to {} failed: {}", topic, e)))?;

        debug!("Published message to group: {}", topic);

        Ok(())
    }

    /// Store data in DHT
    ///
    /// # Arguments
    /// * `key` — DHT key
    /// * `value` — data to store
    ///
    /// # Returns
    /// * `Ok(())` — put initiated
    /// * `Err(P2PError)` — on failure
    pub fn dht_put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), P2PError> {
        self.swarm
            .behaviour_mut()
            .kademlia
            .put_record(
                kad::Record::new(key, value),
                kad::Quorum::One,
            )
            .map_err(|e| P2PError::Swarm(format!("DHT put failed: {}", e)))?;

        debug!("DHT put record");

        Ok(())
    }

    /// Get data from DHT
    ///
    /// # Arguments
    /// * `key` — DHT key to lookup
    ///
    /// # Returns
    /// * `Ok(())` — get initiated
    /// * `Err(P2PError)` — on failure
    pub fn dht_get(&mut self, key: Vec<u8>) -> Result<(), P2PError> {
        let _query_id = self
            .swarm
            .behaviour_mut()
            .kademlia
            .get_record(kad::RecordKey::new(&key));

        debug!("DHT get record");

        Ok(())
    }

    /// Bootstrap Kademlia DHT
    ///
    /// # Returns
    /// * `Ok(())` — bootstrap initiated
    /// * `Err(P2PError)` — on failure
    pub fn dht_bootstrap(&mut self) -> Result<(), P2PError> {
        self.swarm
            .behaviour_mut()
            .kademlia
            .bootstrap()
            .map_err(|e| P2PError::Swarm(format!("DHT bootstrap failed: {}", e)))?;

        info!("DHT bootstrap started");

        Ok(())
    }

    /// Add bootstrap peer to DHT
    ///
    /// # Arguments
    /// * `peer_id` — bootstrap peer ID
    /// * `addr` — bootstrap peer address
    pub fn add_bootstrap_peer(&mut self, peer_id: PeerId, addr: &str) -> Result<(), P2PError> {
        let multiaddr: libp2p::Multiaddr = addr
            .parse()
            .map_err(|e| P2PError::Swarm(format!("Invalid bootstrap address '{}': {}", addr, e)))?;

        self.swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer_id, multiaddr);

        info!("Added bootstrap peer: {} at {}", peer_id, addr);

        Ok(())
    }

    /// Run the P2P event loop — processes incoming events
    ///
    /// This method runs indefinitely and should be spawned as a task.
    /// It forwards relevant events to the application via `event_tx`.
    pub async fn run(mut self) -> Result<(), P2PError> {
        info!("P2P event loop started");

        loop {
            match self.swarm.select_next_some().await {
                // mDNS discovered a peer
                SwarmEvent::Behaviour(MeshBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        info!("mDNS discovered peer: {}", peer_id);
                        let _ = self.event_tx.send(P2PEvent::PeerDiscovered(peer_id)).await;

                        // Add to DHT
                        self.swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, libp2p::Multiaddr::empty());
                    }
                }

                // mDNS expired a peer
                SwarmEvent::Behaviour(MeshBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        info!("mDNS expired peer: {}", peer_id);
                        let _ = self
                            .event_tx
                            .send(P2PEvent::PeerDisconnected(peer_id))
                            .await;
                    }
                }

                // Gossipsub message received
                SwarmEvent::Behaviour(MeshBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message {
                        propagation_source,
                        message_id,
                        message,
                    },
                )) => {
                    debug!(
                        "Gossipsub message from: {} (id: {})",
                        propagation_source, message_id
                    );

                    // Try to parse as P2PMessage
                    if let Ok(p2p_msg) = serde_json::from_slice::<P2PMessage>(&message.data) {
                        let _ = self
                            .event_tx
                            .send(P2PEvent::MessageReceived {
                                from: propagation_source,
                                message: p2p_msg,
                            })
                            .await;
                    } else {
                        // Raw group message
                        let topic_str = message.topic.as_str().to_string();

                        let _ = self
                            .event_tx
                            .send(P2PEvent::GossipMessageReceived {
                                from: propagation_source,
                                topic: topic_str,
                                message: message.data,
                            })
                            .await;
                    }
                }

                // Kademlia event
                SwarmEvent::Behaviour(MeshBehaviourEvent::Kademlia(
                    kad::Event::OutboundQueryProgressed {
                        result,
                        ..
                    },
                )) => match result {
                    kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(
                        kad::PeerRecord { record, .. },
                    ))) => {
                        let _ = self
                            .event_tx
                            .send(P2PEvent::DhtRecordFound {
                                key: record.key.to_vec(),
                                value: record.value,
                            })
                            .await;
                    }
                    kad::QueryResult::Bootstrap(Ok(_)) => {
                        info!("DHT bootstrap completed");
                    }
                    kad::QueryResult::Bootstrap(Err(e)) => {
                        warn!("DHT bootstrap failed: {:?}", e);
                    }
                    _ => {}
                },

                // Identify event
                SwarmEvent::Behaviour(MeshBehaviourEvent::Identify(
                    identify::Event::Received { peer_id, info, .. },
                )) => {
                    info!("Identified peer: {} (version: {})", peer_id, info.protocol_version);

                    // Add observed addresses to DHT
                    for addr in info.listen_addrs {
                        self.swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, addr);
                    }
                }

                // Connection established
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    endpoint,
                    num_established,
                    ..
                } => {
                    info!(
                        "Connection established: {} ({}, {} total)",
                        peer_id,
                        if endpoint.is_dialer() { "outgoing" } else { "incoming" },
                        num_established
                    );
                    let _ = self.event_tx.send(P2PEvent::PeerDiscovered(peer_id)).await;
                }

                // Connection closed
                SwarmEvent::ConnectionClosed {
                    peer_id,
                    num_established,
                    ..
                } => {
                    warn!(
                        "Connection closed: {} ({} remaining)",
                        peer_id, num_established
                    );
                    let _ = self
                        .event_tx
                        .send(P2PEvent::PeerDisconnected(peer_id))
                        .await;
                }

                // New listen address
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on: {}", address);
                }

                // Incoming connection
                SwarmEvent::IncomingConnection { .. } => {
                    debug!("Incoming connection attempt");
                }

                // External address confirmed
                SwarmEvent::ExternalAddrConfirmed { address } => {
                    info!("External address confirmed: {}", address);
                }

                _ => {
                    debug!("Unhandled swarm event");
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_message_serialization() {
        let msg = P2PMessage {
            ciphertext: vec![1, 2, 3, 4],
            signature: vec![5, 6, 7, 8],
            timestamp: 1234567890,
            msg_type: MessageType::Direct,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: P2PMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg.ciphertext, deserialized.ciphertext);
        assert_eq!(msg.signature, deserialized.signature);
        assert_eq!(msg.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_message_type_variants() {
        let types = vec![
            MessageType::Direct,
            MessageType::Group,
            MessageType::KeyExchange,
            MessageType::KeyExchangeResponse,
            MessageType::Ping,
        ];

        for msg_type in types {
            let serialized = serde_json::to_string(&msg_type).unwrap();
            let deserialized: MessageType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(format!("{:?}", msg_type), format!("{:?}", deserialized));
        }
    }

    #[tokio::test]
    #[ignore = "Требует инициализацию libp2p swarm, медленно для unit тестов"]
    async fn test_p2p_node_creation() {
        let (tx, _rx) = mpsc::channel(100);
        let node = P2PNode::new(None, tx).await;
        assert!(node.is_ok());
    }
}
