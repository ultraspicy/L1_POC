use std::collections::HashSet;
use std::net::SocketAddr;
use warp::Filter;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Peer {
    address: String,
}

#[derive(Debug, Clone)]
struct PeerRegistry {
    peers: Arc<RwLock<HashSet<String>>>,
}

impl PeerRegistry {
    fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    async fn add_peer(&self, peer: String) {
        let mut peers = self.peers.write().await;
        peers.insert(peer);
    }

    async fn get_peers(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        peers.iter().cloned().collect()
    }
}

#[tokio::main]
async fn main() {
    let registry = PeerRegistry::new();

    let registry_filter = warp::any().map(move || registry.clone());

    let add_peer = warp::path("add_peer")
        .and(warp::post())
        .and(warp::body::json())
        .and(registry_filter.clone())
        .and_then(handle_add_peer);

    let get_peers = warp::path("get_peers")
        .and(warp::get())
        .and(registry_filter.clone())
        .and_then(handle_get_peers);

    let routes = add_peer.or(get_peers);

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

async fn handle_add_peer(peer: Peer, registry: PeerRegistry) -> Result<impl warp::Reply, warp::Rejection> {
    registry.add_peer(peer.address).await;
    Ok(warp::reply::json(&"Peer added"))
}

async fn handle_get_peers(registry: PeerRegistry) -> Result<impl warp::Reply, warp::Rejection> {
    let peers = registry.get_peers().await;
    Ok(warp::reply::json(&peers))
}
