use std::{error::Error, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use rand::prelude::*;

use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream}, sync::RwLock,
};

#[derive(Debug)]
enum LoadBalancerAlgorithm {
    RoundRobin,
    Random,
    LeastConnections,
}

use futures::FutureExt;
#[derive(Debug)]
pub struct LoadBalancer {
    workers: Vec<Arc<SocketAddr>>,
    current_worker: Arc<RwLock<usize>>,
    algorith: LoadBalancerAlgorithm,
}

impl LoadBalancer {
    pub fn new(raw_workers: Vec<String>) -> Self {
        let mut workers: Vec<Arc<SocketAddr>> = vec![];

        for addr in raw_workers {
            workers.push(Arc::new(
                SocketAddr::from_str(&addr).expect("Failed to parse url"),
            ));
        }

        Self {
            workers,
            current_worker: Arc::new(RwLock::new(0)),
            algorith: LoadBalancerAlgorithm::LeastConnections,

        }
    }
    pub async fn get_next(&mut self) -> Arc<SocketAddr> {
        match self.algorith {
            LoadBalancerAlgorithm::RoundRobin => {
                let mut current = self.current_worker.write().await;
                // Use a round-robin strategy to select a worker
                let worker = self.workers.get(*current).unwrap();
                *current = (*current + 1) % self.workers.len();
                worker.clone()
            },
            LoadBalancerAlgorithm::Random => {
                let num_workers = self.workers.len();
                let mut rng = rand::thread_rng();
                let rand_num = (num_workers as f32 * rng.gen::<f32>()).floor();

                let mut current = self.current_worker.write().await;

                let worker = self.workers.get(rand_num as usize).unwrap();
                *current = rand_num as usize;

                worker.clone()


            },
            LoadBalancerAlgorithm::LeastConnections => {
                let current_count = self.current_worker_counts();
                println!("{current_count:?}");
                let min_value = current_count.iter().min().unwrap();
                let index = current_count.iter().position(|x| x == min_value); 
                println!("lowest used index: {:?}", index);
                self.workers[index.unwrap_or(0)].clone()
            }
        }
    }

    pub fn current_worker_counts(&mut self) -> Vec<usize> {
        let mut workers_count: Vec<usize> = vec![];
        for worker in self.workers.iter() {
            workers_count.push(Arc::strong_count(&worker));
        }
        workers_count
    }

    pub async fn run(&mut self, listener: TcpListener) -> Result<(), Box<dyn Error>> {
        while let Ok((mut inbound, _)) = listener.accept().await {
            let outbound_addr = self.get_next().await;

            let mut outbound = TcpStream::connect(*outbound_addr).await?;
            println!("forwarding to {:?}", outbound_addr);

            tokio::task::spawn(async move {
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
