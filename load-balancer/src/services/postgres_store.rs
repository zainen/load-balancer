use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PostgresWorkerStore {
    pool: PgPool,
}

impl PostgresWorkerStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_workers(&self) -> Result<Vec<String>, &str> {
        let workers: Vec<String> = sqlx::query!("Select worker_address from workers")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| "failed to get records")?
            .iter()
            .map(|row| row.worker_address.clone())
            .collect();

        Ok(workers)
    }
}
