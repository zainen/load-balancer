use std::{error::Error, net::SocketAddr, str::FromStr};

use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
};

use futures::FutureExt;

struct LoadBalancer {
    pub workers: Vec<SocketAddr>,
    current_worker: usize,
}

impl LoadBalancer {
    pub fn new(raw_workers: Vec<String>) -> Self {
        let mut workers: Vec<SocketAddr> = vec![];

        for addr in raw_workers {
            workers.push(SocketAddr::from_str(&addr).expect("Failed to parse url"));
        }
        
        Self {
            workers,
            current_worker: 0,
        }

    }

    pub fn get_next(&mut self) -> SocketAddr {
        // Use a round-robin strategy to select a worker
        let worker = self.workers.get(self.current_worker).unwrap();
        self.current_worker = (self.current_worker + 1) % self.workers.len();
        *worker
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    println!("listening on {:?}", addr);

    while let Ok((mut inbound, _)) = listener.accept().await {
        let outbound_addr = lb.get_next();
        let mut outbound =
            TcpStream::connect(outbound_addr).await?;
        println!("forwarding to {:?}", outbound_addr);

        tokio::spawn(async move {
            copy_bidirectional(&mut inbound, &mut outbound)
                .map(|r| {
                    if let Err(e) = r {
                        println!("Failed to transfer. Error: {:?}", e);
                    }
                })
                .await
        });
    }
    Ok(())
}

