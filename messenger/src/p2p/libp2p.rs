// messenger/src/p2p/libp2p.rs
//! P2P сеть на основе libp2p (TCP, QUIC, Noise, Yamux)

use libp2p::{
    gossipsub, kad, mdns, noise, quic, tcp, yamux, PeerId, Swarm, SwarmBuilder,
    identity::Keypair, Multiaddr,
};
use libp2p::core::transport::Transport;
use libp2p::core::upgrade::Version;
use libp2p::gossipsub::{MessageAuthenticity, ValidationMode, Message, TopicHash};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

/// Объединённая сеть для P2P
#[derive(NetworkBehaviour)]
pub struct LibertyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<libp2p::kad::store::MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
}

pub struct P2PNode {
    pub peer_id: PeerId,
    pub swarm: Swarm<LibertyBehaviour>,
    pub chat_topics: HashMap<String, TopicHash>,
}

impl P2PNode {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Генерация ключей
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        // TCP транспорт
        let tcp_transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(Version::V1)
            .authenticate(noise::Config::new(&keypair)?)
            .multiplex(yamux::Config::default());
        
        // QUIC транспорт
        let quic_config = quic::Config::new(&keypair);
        let quic_transport = quic::tokio::Transport::new(quic_config);
        
        // Комбинированный транспорт
        let transport = tcp_transport
            .or_quic(quic_transport)
            .boxed();
        
        // DHT (Kademlia)
        let store = libp2p::kad::store::MemoryStore::new(peer_id);
        let kademlia_config = kad::Config::default();
        let kademlia = kad::Behaviour::with_config(peer_id, store, kademlia_config);
        
        // Gossipsub для чатов
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(|msg: &Message| {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                msg.source.hash(&mut hasher);
                msg.sequence_number.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            })
            .build()?;
        
        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )?;
        
        // mDNS для локального обнаружения
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        )?;
        
        // Объединённое поведение
        let behaviour = LibertyBehaviour {
            gossipsub,
            kademlia,
            mdns,
        };
        
        // Создание Swarm
        let swarm = SwarmBuilder::with_tokio_executor(
            transport,
            behaviour,
            peer_id,
        )
        .build()?;
        
        Ok(Self { 
            peer_id, 
            swarm,
            chat_topics: HashMap::new(),
        })
    }
    
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
    
    /// Подписка на топик чата
    pub fn subscribe_chat(&mut self, chat_id: &str) -> Result<bool, Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new(format!("liberty-chat-{}", chat_id));
        let topic_hash = topic.hash();
        
        let subscribed = self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        self.chat_topics.insert(chat_id.to_string(), topic_hash);
        
        Ok(subscribed)
    }
    
    /// Отправка сообщения в чат
    pub fn publish_message(&mut self, chat_id: &str, message: Vec<u8>) -> Result<bool, Box<dyn Error>> {
        if let Some(topic_hash) = self.chat_topics.get(chat_id) {
            let published = self.swarm.behaviour_mut().gossipsub.publish(topic_hash.clone(), message)?;
            Ok(published)
        } else {
            Err("Топик чата не найден".into())
        }
    }
    
    /// Запуск P2P узла
    pub async fn start(&mut self, listen_addr: &str) -> Result<(), Box<dyn Error>> {
        let addr: Multiaddr = listen_addr.parse()?;
        self.swarm.listen_on(addr)?;
        
        tracing::info!("P2P узел запущен: {}", self.peer_id);
        tracing::info!("Слушаем адрес: {}", listen_addr);
        
        // Основной цикл обработки событий
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!("Новый адрес прослушивания: {}", address);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            tracing::info!("Подключение к пиру: {}", peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            tracing::info!("Отключение от пира: {}", peer_id);
                        }
                        SwarmEvent::Behaviour(LibertyBehaviour { gossipsub, kademlia, mdns }) => {
                            // Обработка gossipsub событий
                            if let Some(gossipsub::Event::Message { message, .. }) = gossipsub.into_iter().next() {
                                tracing::info!("Получено сообщение: {} байт", message.data.len());
                            }
                            
                            // Обработка mDNS событий
                            if let Some(mdns::Event::Discovered(peers)) = mdns.into_iter().next() {
                                for (peer_id, addr) in peers {
                                    tracing::info!("Обнаружен пир {}: {}", peer_id, addr);
                                    self.swarm.dial(addr)?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    /// Dial к другому пиру
    pub fn dial(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.dial(addr)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_creation() {
        let node = P2PNode::new();
        assert!(node.is_ok());
        
        let node = node.unwrap();
        assert!(!node.peer_id().to_base58().is_empty());
    }
}
