use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::{sync::Arc, collections::HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Peer {
    pub address: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PeerRegistry {
    pub address: String,
    pub peers: Arc<RwLock<HashSet<String>>>,
}

impl PeerRegistry {
    pub(crate) fn new(address: String) -> Self {
        Self {
            address: address,
            peers: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub(crate) async fn add_peer(&self, peer: String) {
        {
            let mut peers = self.peers.write().await;
        
            if peers.insert(peer.clone()) {
                println!("Peer added: {}", peer);
            } else {
                println!("Peer already exists: {}", peer);
            }
        } // we want to make sure that the broadcast_join happens after the write() finishes

        self.broadcast_join().await;
    }

    pub(crate) async fn get_peers(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        let peers_list = peers.iter().cloned().collect::<Vec<String>>();
        println!("The own address is: {:?}. Current peers: {:?}", self.address, peers_list);
        peers_list
    }

    pub(crate) async fn broadcast_join(&self) {
        let peers = self.get_peers().await;
        let mut all_peers = peers.into_iter().collect::<HashSet<_>>();
        all_peers.insert(self.address.clone());

        for peer in all_peers {
            let client = reqwest::Client::new();
            let response = client.post(&format!("http://{}/add_peer", peer)) //create an HTTP POST request.
                .json(&Peer { address: self.address.clone() }) // serializes the given data into JSON format.
                .send()
                .await;

            match response {
                Ok(_) => println!("Successfully notified peer at {}", peer),
                Err(err) => eprintln!("Failed to notify peer at {}: {:?}", peer, err),
            }
        }
    }
}