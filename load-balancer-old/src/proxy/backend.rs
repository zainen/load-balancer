use std::{net::SocketAddr, str::FromStr, time::Duration};

use tokio::{net::TcpStream, time::timeout};

#[derive(Debug, Clone)]
pub struct Backend {
    pub listening_addr: SocketAddr,
}

impl Backend {
    pub fn new(host_url: String) -> Self {
        let listening_addr = SocketAddr::from_str(&host_url).expect("Invalid Host url");
        Self { listening_addr }
    }

    pub async fn health_check(&self) -> bool {
        match timeout(Duration::from_secs(2), TcpStream::connect(&self.listening_addr)).await {
            Ok(_) => true,
            _ => false 
        }
    }
}
