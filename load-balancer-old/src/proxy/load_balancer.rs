use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use super::{Backend, Proxy};

#[derive(Clone, Copy)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
}

#[derive(Clone)]
pub struct LoadBalancer {
    pub listening_addr: SocketAddr,
    pub proxy: Arc<RwLock<Proxy>>,
}

impl LoadBalancer {
    // pub async fn forward_request(&mut self, req: Request<Incoming>) -> Result<Response<Incoming>, anyhow::Error> {
    //     let backend = self.get_next().await;
    //
    //
    //     if let Some(path_and_query) = req.uri().path_and_query() {
    //         worker_uri.push_str(path_and_query.as_str())
    //     }
    //
    //     let new_uri = Uri::from_str(&worker_uri).unwrap();
    //
    //     let headers = req.headers().clone();
    //
    //     let mut new_req = Request::builder()
    //         .method(req.method())
    //         .uri(new_uri)
    //         .body(req.into_body())
    //         .expect("request builder");
    //
    //     for (key, value) in headers.iter() {
    //         new_req.headers_mut().insert(key, value.clone());
    //     }
    //     backend.make_request(new_req).await
    // }

    // pub async fn get_next(&mut self) -> Backend {
    //     match self.current_algorithm {
    //         LoadBalancingAlgorithm::RoundRobin => {
    //             let index = self.current_worker;
    //             let backend = self.backends[index].clone();
    //             self.current_worker = (index + 1) % self.backends.len();
    //             backend
    //         },
    //         _ => todo!()
    //     }
    // }

    pub fn new(listening_addr: SocketAddr, worker_hosts: Vec<String>) -> Result<Self, String> {
        if worker_hosts.is_empty() {
            return Err("No worker hosts provided".into());
        }

        let mut backends: Vec<Backend> = vec![];

        for host in worker_hosts {
            backends.push(Backend::new(host));
        }

        let proxy = Arc::new(RwLock::new(Proxy {
            backends: Arc::new(backends),
            current_index: Mutex::new(0_usize),
            algorithm: LoadBalancingAlgorithm::RoundRobin,
        }));

        Ok(Self {
            listening_addr,
            proxy,
        })
    }

    pub async fn run_load_balancer(&mut self) -> Result<(), anyhow::Error> {
        let listening_addr = TcpListener::bind(self.listening_addr).await?;

        let thread_proxy = self.proxy.clone();

        tokio::spawn(async move {
            if let Err(e) = run_server(listening_addr, thread_proxy).await {
                println!("Error running proxy: {:?}", e);
            }
        });

        Ok(())
    }
}

async fn run_server(
    lb_listener: TcpListener,
    proxy: Arc<RwLock<Proxy>>,
) -> Result<(), anyhow::Error> {
    loop {
        let (client_stream, _) = lb_listener.accept().await?;
        let thread_proxy = proxy.clone();

        tokio::spawn(async move {
            let backend_addr = {
                let thread_proxy = thread_proxy.read().await;
                thread_proxy.get_next(thread_proxy.algorithm).await
            };

            match handle_connection(client_stream, backend_addr).await {
                Ok(_) => println!("connection result ok"),
                Err(e) => println!("{:?}", e),
            }
        });
    }
}

async fn handle_connection(
    mut client_stream: TcpStream,
    backend_addr: SocketAddr,
) -> Result<(), anyhow::Error> {
    let mut backend_stream = TcpStream::connect(backend_addr).await?;

    match copy_bidirectional(&mut client_stream, &mut backend_stream).await {
        Ok((from_client, from_server)) => {
            println!("{:?}:{:?}", from_client, from_server);
            Ok(())
        }
        Err(e) => {
            println!("{:?}", e);
            Err(e.into())
        }
    }
}
