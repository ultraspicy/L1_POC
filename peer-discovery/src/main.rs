mod utils;
mod peer;

use warp::Filter;
use tokio::task;
use std::{error::Error, env};
use kube::{Client, api::Api};
use k8s_openapi::api::core::v1::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let own_address = env::var("OWN_ADDRESS").ok();
    let mut is_self_bootstrap = false;

    match own_address {
        Some(address) => {
            // we set up a static IP for bootstrap node and 
            // save it through env_var, in k8s deployment configuration
            println!("This is a bootstrap node with address: {}", address);
            is_self_bootstrap = true;
        },
        None => {
            println!("This is not a bootstrap node.");
        },
    }

    let namespace = env::var("NAMESPACE").unwrap_or_else(|_| "namespace_undefined".to_string());
    let service_name = env::var("SERVICE_NAME").unwrap_or_else(|_| "service_not_found".to_string());
    let cluster_ip = if utils::is_running_inside_kubernetes() {
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

    let registry = peer::PeerRegistry::new(cluster_ip.clone());
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

    
    // Run the Warp server in a separate task
    let server = task::spawn(async move {
        // server accessible on port 8080 through any IP address assigned to any network interface on the machine
        warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
        println!("server {:?} listening", cluster_ip)
    });

    // Wait for a short period to ensure the server has started
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    if !is_self_bootstrap {
        // this is a hard-coded IP address for bootstrap. Any new peer will connect to, and be broadcasted by this node
        let bootstrap_address = "10.96.0.12".to_string(); 
        registry.add_peer(bootstrap_address.clone()).await;
    }

    // awaiting the server task to ensure that the main function remains active and does not exit.
    server.await?;

    Ok(())
}

async fn handle_add_peer(peer: peer::Peer, registry: peer::PeerRegistry) -> Result<impl warp::Reply, warp::Rejection> {
    registry.add_peer(peer.address).await;
    Ok(warp::reply::json(&"Peer added"))
}

async fn handle_get_peers(registry: peer::PeerRegistry) -> Result<impl warp::Reply, warp::Rejection> {
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
