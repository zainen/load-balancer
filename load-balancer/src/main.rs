use std::{net::SocketAddr, str::FromStr};

use load_balancer::{
    proxy::load_balancer::LoadBalancer,
    services::postgres_store::PostgresWorkerStore,
    utils::{constants::DATABSE_URL, tracing::init_tracing},
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tracing::{event, Level};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing().expect("Failed to initialize tracing");

    let pool = configure_postgres().await;
    let db = PostgresWorkerStore::new(pool.clone());

    let workers: Vec<String> = match db.get_workers().await {
        Ok(workers) => workers,
        Err(e) => {
            event!(Level::WARN, e);
            vec![]
        }
    };

    let mut lb = LoadBalancer::new(workers, db);

    let addr = SocketAddr::from_str("127.0.0.1:3000").unwrap();

    let listener = TcpListener::bind(addr).await?;

    event!(Level::INFO, "Listening at addr: {}", addr);

    lb.run(listener).await
}

pub async fn get_postgres_pool(url: String) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().max_connections(5).connect(&url).await
}

async fn configure_postgres() -> PgPool {
    let pool = get_postgres_pool(DATABSE_URL.to_string())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migration");

    pool
}
