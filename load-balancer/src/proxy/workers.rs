use rand::prelude::*;
use tokio::sync::RwLock;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};

use tracing::{event, Level};

use super::load_balancer::LoadBalancerAlgorithm;

#[derive(Debug)]
pub struct Workers {
    pub worker_addrs: Vec<Arc<SocketAddr>>,
    pub workers_health: HashMap<SocketAddr, bool>,
    pub current_worker: usize,
    pub current_worker_loads: HashMap<SocketAddr, usize>,
    pub algorithm: LoadBalancerAlgorithm,
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
            algorithm: LoadBalancerAlgorithm::Random,
        }
    }

    pub async fn get_next(&mut self) -> Arc<SocketAddr> {
        self.optimal_algorithm();
        let lb = self.algorithm.clone();
        event!(Level::INFO, "Current Algorithm: {:?}", lb);

        let worker = match lb {
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
                    .unwrap()
                    .clone();
                worker
            }
        };
        println!("{:?}", self.current_worker_loads);
        self.increase_worker_count(&worker);
        worker
    }

    fn increase_worker_count(&mut self, addr: &Arc<SocketAddr>) {
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

    pub fn update_healthy_workers(&mut self, updated_map: HashMap<SocketAddr, bool>) {
        self.workers_health = updated_map;
    }

    fn optimal_algorithm (&mut self) {
        let loads = self.current_worker_loads.clone().into_values().collect::<Vec<usize>>();

        let mut values: Vec<&usize> = loads.iter().collect();
        
        values.sort_by(|a, b| a.cmp(b));

        let max = values[values.len() - 1];
        let min = values[0];
        let algorithm_candidate: LoadBalancerAlgorithm;
        if max - min > 10 {
            algorithm_candidate = LoadBalancerAlgorithm::LeastConnections;
        } else if max - min > 5 {
            algorithm_candidate = LoadBalancerAlgorithm::Random;
        } else {
            algorithm_candidate = LoadBalancerAlgorithm::RoundRobin
        }

        if self.algorithm != algorithm_candidate {
            
            event!(Level::WARN, "Algorithm switched: {:?}", algorithm_candidate);
            self.algorithm = algorithm_candidate;
        }

    }
}
