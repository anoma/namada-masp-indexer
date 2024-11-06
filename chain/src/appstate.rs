use std::env;
use std::time::Duration;

use anyhow::Context;
use deadpool_diesel::postgres::{Object, Pool as DbPool};

#[derive(Clone)]
pub struct AppState {
    db: DbPool,
}

impl AppState {
    pub async fn new(db_url: String) -> anyhow::Result<Self> {
        let max_pool_size = env::var("DATABASE_POOL_SIZE")
            .unwrap_or_else(|_| 8.to_string())
            .parse::<usize>()
            .unwrap_or(8_usize);

        let pool = tryhard::retry_fn(|| async {
            let pool_manager = deadpool_diesel::Manager::from_config(
                db_url.clone(),
                deadpool_diesel::Runtime::Tokio1,
                deadpool_diesel::ManagerConfig {
                    recycling_method:
                        deadpool_diesel::RecyclingMethod::Verified,
                },
            );
            DbPool::builder(pool_manager)
                .max_size(max_pool_size)
                .build()
                .context("Failed to build Postgres db pool")
        })
        .retries(5)
        .exponential_backoff(Duration::from_millis(100))
        .max_delay(Duration::from_secs(5))
        .await?;

        Ok(Self { db: pool })
    }

    pub async fn get_db_connection(&self) -> anyhow::Result<Object> {
        self.db
            .get()
            .await
            .context("Failed to get db connection handle from deadpool")
    }
}
