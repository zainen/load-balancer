use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::{event, Level};

use tokio::{
    io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::RwLock,
    time::sleep,
};

use futures::FutureExt;

use crate::{
    services::postgres_store::PostgresWorkerStore, utils::stream_reader::read_status_code,
};

use super::workers::Workers;

#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancerAlgorithm {
    RoundRobin,
    Random,
    LeastConnections,
}

#[derive(Debug)]
pub struct LoadBalancer {
    workers: Arc<RwLock<Workers>>,
    health_check_interval: Duration,
    _db_connection: PostgresWorkerStore,
}

impl LoadBalancer {
    pub fn new(raw_workers: Vec<String>, _db_connection: PostgresWorkerStore) -> Self {
        let workers = Workers::new(raw_workers);

        Self {
            workers: Arc::new(RwLock::new(workers)),
            health_check_interval: Duration::from_secs(60),
            _db_connection,
        }
    }

    pub async fn run(&mut self, listener: TcpListener) -> std::io::Result<()> {
        let workers = self.workers.clone();
        let duration = self.health_check_interval.clone();

        // Task spawned checking health of each worker
        tokio::spawn(async move {
            loop {
                let mut worker_health_map = HashMap::new();
                for worker in workers.read().await.worker_addrs.iter() {
                    let stream = TcpStream::connect(**worker).await;

                    if let Ok(mut stream) = stream {
                        let _ = stream
                            .write_all(
                                format!(
                        "GET /health_check HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                        worker
                    )
                                .as_bytes(),
                            )
                            .await;

                        let mut buf = Vec::new();
                        let _ = stream.read_to_end(&mut buf).await;

                        let response = String::from_utf8_lossy(&buf);

                        let status_code = read_status_code(response.lines().next().unwrap_or(""));

                        // assuming all responses under 400 are healthy
                        worker_health_map.insert(worker.clone(), status_code < 400);
                        if status_code >= 400 {
                            event!(Level::WARN, "Worker Check Failed {}", worker);
                        }
                    } else {
                        worker_health_map.insert(worker.clone(), false);
                        event!(Level::ERROR, "Worker Stream Failed {}", worker);
                    }
                }
                event!(Level::TRACE, "Routine health check done");

                workers
                    .write()
                    .await
                    .update_healthy_workers(worker_health_map.clone());

                let _ = sleep(duration).await;
            }
        });
        
        while let Ok((mut inbound, _)) = listener.accept().await {
            event!(Level::TRACE, "incoming request");
            let mut workers = self.workers.clone();
            // NOTE this includes using all current healthy workers. meaning there are no healthy
            // workers left
            let try_outbound = workers.write().await.get_next().await;
            if let Some(mut outbound_addr) = try_outbound {
                let connection = TcpStream::connect(*outbound_addr).await;

                if let Err(e) = connection {
                    event!(
                        Level::WARN,
                        "Failed to connect to {outbound_addr}. Error: {e}"
                    );
                    self.health_check().await;

                    workers = self.workers.clone();
                    outbound_addr = if let Some(addr) = workers.write().await.get_next().await {
                        addr
                    } else {
                        event!(
                            Level::ERROR,
                            "Connection failed healthy server not found"
                        );
                            continue;
                    };

                    event!(Level::INFO, "Second attempt sent to {}", outbound_addr);
                }

                let connection = TcpStream::connect(*outbound_addr).await;

                match connection {
                    Ok(mut outbound) => {
                        tokio::spawn(async move {
                            // TODO handle return of of bidirectional result
                            copy_bidirectional(&mut inbound, &mut outbound)
                                .map(|r| {
                                    if let Err(e) = r {
                                        event!(Level::ERROR, "Failed to transfer. Error: {:?}", e);
                                    }
                                })
                                .await;
                            workers.write().await.decrease_worker_count(*outbound_addr);
                            event!(
                                Level::INFO,
                                "Response Sent: counts: {:?}",
                                workers.read().await.current_worker_loads
                            )
                        });
                    }
                    Err(e) => {
                        event!(
                            Level::ERROR,
                            "Failed to connect to {outbound_addr}. request failed. Error: {e}"
                        );
                        self.health_check().await;
                        workers.write().await.decrease_worker_count(*outbound_addr);
                    }
                }
            } else {
                event!(
                    Level::ERROR,
                    "workers all unhealthy {:?}",
                    workers.read().await.workers_health
                );
                event!(Level::TRACE, "run health check");
                self.health_check().await;
            }
        }
        Ok(())
    }

    async fn health_check(&mut self) {
        let workers = self.workers.clone();

        // Task spawned checking health of each worker
        let mut worker_health_map = HashMap::new();
        for worker in workers.read().await.worker_addrs.iter() {
            let stream = TcpStream::connect(**worker).await;

            if let Ok(mut stream) = stream {
                let _ = stream
                    .write_all(
                        format!(
                            "GET /health_check HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                            worker
                        )
                        .as_bytes(),
                    )
                    .await;

                let mut buf = Vec::new();
                let _ = stream.read_to_end(&mut buf).await;

                let response = String::from_utf8_lossy(&buf);

                let status_code = read_status_code(response.lines().next().unwrap_or(""));

                // assuming all responses under 400 are healthy
                worker_health_map.insert(worker.clone(), status_code < 400);
                if status_code >= 400 {
                    event!(Level::WARN, "Worker Check Failed {}", worker);
                }
            } else {
                worker_health_map.insert(worker.clone(), false);
                event!(Level::ERROR, "Worker Stream Failed {}", worker);
            }
        }
        event!(Level::INFO, "Worker Health {:?}", worker_health_map);

        workers
            .write()
            .await
            .update_healthy_workers(worker_health_map.clone());

        // TODO Potentially useful after dynamic worker Socket adders
        // let healthy_workers: Vec<Arc<SocketAddr>> = worker_health_map
        //     .iter()
        //     .filter(|(_, healthy)| **healthy == true)
        //     .map(|(addr, _)| addr.clone())
        //     .collect();
    }
}
