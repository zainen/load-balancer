use std::sync::Arc;
use tracing::{event, Level};

use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use futures::FutureExt;

use super::worker_stats::Workers;

#[derive(Debug, Clone)]
pub enum LoadBalancerAlgorithm {
    RoundRobin,
    Random,
    LeastConnections,
}

#[derive(Debug)]
pub struct LoadBalancer {
    workers: Arc<RwLock<Workers>>,
    algorithm: LoadBalancerAlgorithm,
}

impl LoadBalancer {
    pub fn new(raw_workers: Vec<String>) -> Self {
        let workers = Workers::new(raw_workers);

        Self {
            workers: Arc::new(RwLock::new(workers)),
            algorithm: LoadBalancerAlgorithm::LeastConnections,
        }
    }

    pub async fn run(&mut self, listener: TcpListener) -> std::io::Result<()> {
        // let workers = self.workers.worker_addrs.clone();
        // let duration = self.health_check_interval.clone();

        // // Task spawned checking health of each worker
        // tokio::spawn(async move {
        //     loop {
        //         for worker in workers.iter() {
        //             let stream = TcpStream::connect(**worker).await;
        //
        //             if let Ok(mut stream) = stream {
        //                 let _ = stream
        //                     .write_all(
        //                         format!(
        //                 "GET /health_check HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        //                 worker
        //             )
        //                         .as_bytes(),
        //                     )
        //                     .await;
        //
        //                 let mut buf = Vec::new();
        //                 let _ = stream.read_to_end(&mut buf).await;
        //
        //                 let response = String::from_utf8_lossy(&buf);
        //
        //                 let status_code = response.lines().next().unwrap_or("");
        //                 println!("{status_code}")
        //             } else {
        //                 eprintln!("Failed to connect to worker for health check: {worker:?}")
        //             }
        //         }
        //
        //         sleep(duration);
        //     }
        // });

        while let Ok((mut inbound, _)) = listener.accept().await {
            let outbound_addr = self.workers.clone().write().await.get_next(self.algorithm.clone()).await;
            event!(Level::INFO, "Incoming Stream sent to {}", outbound_addr);

            let mut outbound = TcpStream::connect(*outbound_addr).await?;
            println!("forwarding to {:?}", outbound_addr);

            tokio::spawn(async move {
                // TODO handle return of of bidirectional result
                copy_bidirectional(&mut inbound, &mut outbound)
                    .map(|r| {
                        if let Err(e) = r {
                            eprintln!("Failed to transfer. Error: {:?}", e);
                        }
                    })
                    .await;
            });
        }
        Ok(())
    }
}
