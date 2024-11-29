use std::net::SocketAddr;

use load_balancer::app::LoadBalancer;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let worker_hosts = vec![
        "127.0.0.1:8000".to_string(),
        "127.0.0.1:8001".to_string(),
        // "http://127.0.0.1:8002".to_string(),
        // "http://127.0.0.1:8003".to_string(),
        // "http://127.0.0.1:8004".to_string(),
    ];

    let addr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let load_balancer = LoadBalancer::new(addr, worker_hosts).expect("failed to create load balancer"); 

    println!("Listening on http://{}", addr);

    let mut lb_clone = load_balancer.clone();

    lb_clone.run_load_balancer().await
}
