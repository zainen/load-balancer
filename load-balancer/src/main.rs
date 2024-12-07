use std::{net::SocketAddr, str::FromStr};

use load_balancer::app::{init_tracing, LoadBalancer};
use tokio::net::TcpListener;


#[tokio::main]
async fn main() -> std::io::Result<()> {
        init_tracing().expect("Failed to initialize tracing");
    let worker_hosts = vec![
        "127.0.0.1:8000".to_string(),
        "127.0.0.1:8001".to_string(),
        // "127.0.0.1:8002".to_string(),
        // "127.0.0.1:8003".to_string(),
        // "127.0.0.1:8004".to_string(),
    ];

    let mut lb = LoadBalancer::new(worker_hosts);

    let addr = SocketAddr::from_str("127.0.0.1:3000").unwrap();

    let listener = TcpListener::bind(addr).await?;

    println!("Lisening at addr: {}", addr);
    
    lb.run(listener).await
}
