mod utils;

use std::collections::HashSet;
use warp::Filter;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;
use std::error::Error;
use std::env;
use kube::{Client, api::{Api, ListParams}};
use k8s_openapi::api::core::v1::Service;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Peer {
    address: String,
}

#[derive(Debug, Clone)]
struct PeerRegistry {
    address: String,
    peers: Arc<RwLock<HashSet<String>>>,
}

impl PeerRegistry {
    fn new(address: String) -> Self {
        Self {
            address: address,
            peers: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    async fn add_peer(&self, peer: String) {
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

    async fn get_peers(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        let peers_list = peers.iter().cloned().collect::<Vec<String>>();
        println!("The own address is: {:?}. Current peers: {:?}", self.address, peers_list);
        peers_list
    }

    async fn broadcast_join(&self) {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let own_address = env::var("OWN_ADDRESS").ok();
    let mut is_self_bootstrap = false;
    match own_address {
        Some(address) => {
            // This is a bootstrap node
            println!("This is a bootstrap node with address: {}", address);
            is_self_bootstrap = true;
        },
        None => {
            println!("This is not a bootstrap node.");
        },
    }

    let namespace = env::var("NAMESPACE").unwrap_or_else(|_| "default".to_string());
    let service_name = env::var("SERVICE_NAME").unwrap_or_else(|_| "my-service".to_string());
    let cluster_ip = if is_running_inside_kubernetes() {
        let client = Client::try_default().await?;
        if let Some(ip) = get_cluster_ip(client, &namespace, &service_name).await {
            ip
        } else {
            "Cluster IP not found".to_string()
        }
    } else {
        println!("Not running inside Kubernetes. Using default configuration.");
        // Fallback logic for when not running inside Kubernetes
        "127.0.0.1".to_string()
    };

    let registry = PeerRegistry::new(cluster_ip.clone());
    let registry_for_warp = registry.clone();
    
    let registry_filter = warp::any().map(move || registry_for_warp.clone());
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

    // server accessible on port 8080 through any IP address assigned to any network interface on the machine
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    if !is_self_bootstrap {
        // this is a hard-coded IP address for bootstrap. Any new peer will connect to, and be broadcasted by this node
        let bootstrap_address = "10.96.0.12".to_string(); 
        registry.add_peer(bootstrap_address.clone()).await;
    }

    Ok(())
}

async fn handle_add_peer(peer: Peer, registry: PeerRegistry) -> Result<impl warp::Reply, warp::Rejection> {
    registry.add_peer(peer.address).await;
    Ok(warp::reply::json(&"Peer added"))
}

async fn handle_get_peers(registry: PeerRegistry) -> Result<impl warp::Reply, warp::Rejection> {
    let peers = registry.get_peers().await;
    Ok(warp::reply::json(&peers))
}

async fn get_cluster_ip(client: Client, namespace: &str, service_name: &str) -> Option<String> {
    let services: Api<Service> = Api::namespaced(client, namespace);

    match services.get(service_name).await {
        Ok(service) => {
            service.spec.and_then(|s| s.cluster_ip)
        },
        Err(e) => {
            eprintln!("Failed to get service: {}", e);
            None
        }
    }
}

fn is_running_inside_kubernetes() -> bool {
    env::var("KUBERNETES_SERVICE_HOST").is_ok()
}