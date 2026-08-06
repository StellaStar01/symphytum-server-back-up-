pub mod account;
pub mod inventory;
pub mod jump_rope;
pub mod live;
pub mod models;
pub mod user_data;

use parking_lot::RwLock;
use sqlx::Error;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use types::rpc::api::common::UserData;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
    cache: Arc<RwLock<HashMap<String, Arc<UserData>>>>,
}

impl Database {
    pub async fn open(path: &Path) -> Result<Self, Error> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// full snapshot
    pub async fn user_data(&self, uid: &str) -> Result<Arc<UserData>, Error> {
        if let Some(cached) = self.cache.read().get(uid) {
            return Ok(cached.clone());
        }
        let data = self.build_user_data(uid).await?;
        let arc = Arc::new(data);
        self.cache.write().insert(uid.to_string(), arc.clone());
        Ok(arc)
    }

    pub fn invalidate(&self, uid: &str) {
        self.cache.write().remove(uid);
    }
}

pub fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before epoch")
        .as_secs() as i64
}
