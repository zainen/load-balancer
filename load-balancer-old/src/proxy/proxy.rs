use std::{net::SocketAddr, sync::{Arc, Mutex}};


use super::{Backend, LoadBalancingAlgorithm};

pub struct Proxy {
    pub backends: Arc<Vec<Backend>>,
    pub current_index: Mutex<usize>,
    pub algorithm: LoadBalancingAlgorithm,

}

impl Proxy {
    pub fn new(worker_hosts: Vec<String>) -> Self {
        let mut backends: Vec<Backend> = vec![];

        for host in worker_hosts {
            backends.push(Backend::new(host));
        }

        Self {
            backends: Arc::new(backends),
            current_index: Mutex::new(0),
            algorithm: LoadBalancingAlgorithm::RoundRobin
        }
    }
    pub async fn get_next(&self, algorithm: LoadBalancingAlgorithm) -> SocketAddr {
        match algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                let mut index = self.current_index.lock().unwrap();
                let backends = self.backends.clone();
                let addr = backends[*index].listening_addr;
                *index = (*index + 1) % self.backends.len();
                addr
            },
            _ => {
                todo!()
            }
        }
    }
}
