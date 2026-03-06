use libp2p::{
    gossipsub, kad, mdns, noise, quic, tcp, yamux, PeerId, Swarm, SwarmBuilder,
};
use libp2p::core::transport::Transport;
use libp2p::identity::Keypair;
use std::error::Error;

pub struct P2PNode {
    pub peer_id: PeerId,
    pub swarm: Option<Swarm<Behaviour>>,
}

#[derive(libp2p::NetworkBehaviour)]
pub struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    mdns: mdns::tokio::Behaviour,
}

impl P2PNode {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());

        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        let behaviour = Behaviour {
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub::Config::default(),
            )?,
            kademlia: kad::Behaviour::new(peer_id, kad::store::MemoryStore::new(peer_id)),
            mdns: mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                peer_id,
            )?,
        };

        Ok(P2PNode {
            peer_id,
            swarm: None,
        })
    }

    pub fn peer_id(&self) -> String {
        self.peer_id.to_string()
    }
}
