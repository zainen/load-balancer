use rand::prelude::*;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};

use tracing::{event, Level};

use super::load_balancer::LoadBalancerAlgorithm;

#[derive(Debug)]
pub struct Workers {
    pub worker_addrs: Vec<Arc<SocketAddr>>,
    pub workers_health: HashMap<Arc<SocketAddr>, bool>,
    pub current_worker: usize,
    pub current_worker_loads: HashMap<Arc<SocketAddr>, usize>,
    pub algorithm: LoadBalancerAlgorithm,
}

impl Workers {
    pub fn new(raw_workers: Vec<String>) -> Self {
        let mut worker_addrs: Vec<Arc<SocketAddr>> = vec![];
        let mut workers_health_map = HashMap::new();
        let mut worker_loads_map = HashMap::new();

        for addr in raw_workers {
            let addr = SocketAddr::from_str(&addr).expect("Failed to parse url");
            let arc_addr = Arc::new(addr);
            worker_addrs.push(arc_addr.clone());
            workers_health_map.insert(arc_addr.clone(), true); // Optimistic health check
            worker_loads_map.insert(arc_addr, 0);
        }

        Self {
            worker_addrs,
            workers_health: workers_health_map,
            current_worker: 0,
            current_worker_loads: worker_loads_map,
            algorithm: LoadBalancerAlgorithm::Random,
        }
    }

    pub async fn get_next(&mut self) -> Option<Arc<SocketAddr>> {
        self.optimal_algorithm();
        let lb = self.algorithm.clone();
        event!(Level::INFO, "Current Algorithm: {:?}", lb);

        let healthy_workers = self.get_healthy_workers();
        // NOTE guard for unwraps
        if healthy_workers.is_empty() {
            return None
        }
        let worker = match lb {
            LoadBalancerAlgorithm::RoundRobin => {
                // Use a round-robin strategy to select a worker
                if self.current_worker > healthy_workers.len() {
                    self.current_worker = 0_usize
                } else {
                    self.current_worker = (self.current_worker + 1) % healthy_workers.len();
                }
                let worker = healthy_workers.get(self.current_worker).unwrap().clone();
                worker
            }

            LoadBalancerAlgorithm::Random => {
                let healthy_workers = self.get_healthy_workers();
                let num_workers = healthy_workers.len();
                let mut rng = rand::thread_rng();
                let rand_num = (num_workers as f32 * rng.gen::<f32>()).floor();

                let worker = healthy_workers
                    .get(rand_num as usize)
                    .unwrap_or(&self.worker_addrs[0])
                    .clone();
                self.current_worker = rand_num as usize;
                worker
            }

            LoadBalancerAlgorithm::LeastConnections => {
                let current_healthy = self.get_healthy_workers();
                let filtered_loads: Vec<(&Arc<SocketAddr>, &usize)> = self
                    .current_worker_loads
                    .iter()
                    .filter(|(addr, _)| current_healthy.contains(addr))
                    .collect();
                let (addr, _) = filtered_loads.iter().min_by_key(|entry| entry.1).unwrap();
                let worker = self
                    .worker_addrs
                    .iter()
                    .find(|worker| **worker == **addr)
                    .unwrap()
                    .clone();
                worker
            }
        };

        self.increase_worker_count(&worker);
        Some(worker)
    }

    fn increase_worker_count(&mut self, addr: &Arc<SocketAddr>) {
        let count = self.current_worker_loads.get_mut(addr);

        if let Some(current_count) = count {
            *current_count += 1;
            // println!("increment count: {current_count:?}");
        } else {
            self.current_worker_loads.insert(addr.clone(), 1);
        }
    }

    pub fn decrease_worker_count(&mut self, addr: SocketAddr) {
        let count = self.current_worker_loads.get_mut(&addr);

        if let Some(current_count) = count {
            if *current_count > 0 {
                *current_count -= 1;
            }
            // println!("decrement count: {current_count:?}");
        } else {
            event!(Level::ERROR, "worker map record should not be missing");
        }
    }

    pub fn update_healthy_workers(&mut self, updated_map: HashMap<Arc<SocketAddr>, bool>) {
        self.workers_health = updated_map;
    }

    fn optimal_algorithm(&mut self) {
        let loads = self
            .current_worker_loads
            .clone()
            .into_values()
            .collect::<Vec<usize>>();

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

    fn _update_workers(&mut self, updated_workers: Vec<Arc<SocketAddr>>) {
        // let healthy_workers: Vec<Arc<SocketAddr>> = worker_health_map
        //     .iter()
        //     .filter(|(_, healthy)| **healthy == true)
        //     .map(|(addr, _)| addr.clone())
        //     .collect();
        self.worker_addrs = updated_workers;
    }

    fn get_healthy_workers(&self) -> Vec<Arc<SocketAddr>> {
        self.workers_health
            .iter()
            .filter(|(_, health)| **health == true)
            .map(|(addr, _)| addr.clone())
            .collect()
    }
}
