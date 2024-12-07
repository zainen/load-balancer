use rand::prelude::*;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};

use tracing::{event, Level};

use super::load_balancer::LoadBalancerAlgorithm;

#[derive(Debug)]
pub struct Workers {
    pub worker_addrs: Vec<Arc<SocketAddr>>,
    pub workers_health: HashMap<SocketAddr, bool>,
    pub current_worker: usize,
    pub current_worker_loads: HashMap<SocketAddr, usize>,
}

impl Workers {
    pub fn new(raw_workers: Vec<String>) -> Self {
        let mut worker_addrs: Vec<Arc<SocketAddr>> = vec![];
        let mut workers_health_map = HashMap::new();
        let mut worker_loads_map = HashMap::new();

        for addr in raw_workers {
            let addr = SocketAddr::from_str(&addr).expect("Failed to parse url");
            worker_addrs.push(Arc::new(addr));
            workers_health_map.insert(addr, true); // Optimistic health check
            worker_loads_map.insert(addr, 0);
        }

        Self {
            worker_addrs,
            workers_health: workers_health_map,
            current_worker: 0,
            current_worker_loads: worker_loads_map,
        }
    }

    pub async fn get_next(&mut self, lb_algorithm: LoadBalancerAlgorithm) -> Arc<SocketAddr> {
        event!(Level::INFO, "Current Algorithm: {:?}", lb_algorithm);

        let worker = match lb_algorithm {
            LoadBalancerAlgorithm::RoundRobin => {
                // Use a round-robin strategy to select a worker
                let worker = self.worker_addrs.get(self.current_worker).unwrap().clone();
                self.current_worker = (self.current_worker + 1) % self.worker_addrs.len();

                worker.clone()
            }

            LoadBalancerAlgorithm::Random => {
                let num_workers = self.worker_addrs.len();
                let mut rng = rand::thread_rng();
                let rand_num = (num_workers as f32 * rng.gen::<f32>()).floor();

                let worker = self.worker_addrs.get(rand_num as usize).unwrap().clone();
                self.current_worker = rand_num as usize;

                worker.clone()
            }

            LoadBalancerAlgorithm::LeastConnections => {
                let read = self.current_worker_loads.clone();
                let (addr, _) = read.iter().min_by_key(|entry| entry.1).unwrap();
                let worker = self
                    .worker_addrs
                    .iter()
                    .find(|worker| **worker == Arc::new(*addr))
                    .clone()
                    .unwrap()
                    .clone();

                worker
            }
        };

        self.increase_worker_count(&worker);
        worker
    }

    pub fn increase_worker_count(&mut self, addr: &Arc<SocketAddr>) {
        let count = self.current_worker_loads.get_mut(&addr);

        if let Some(current_count) = count {
            *current_count += 1;
            // println!("increment count: {current_count:?}");
        } else {
            self.current_worker_loads.insert(*addr.clone(), 1);
        }
    }

    pub fn decrease_worker_count(&mut self, addr: SocketAddr) {
        let count = self.current_worker_loads.get_mut(&addr);

        if let Some(current_count) = count {
            *current_count -= 1;
            // println!("decrement count: {current_count:?}");
        } else {
            event!(Level::ERROR, "worker map record should not be missing");
        }
    }
}
