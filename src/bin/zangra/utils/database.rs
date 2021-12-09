use serenity::prelude::TypeMapKey;
use sqlx::sqlite::{SqlitePoolOptions, SqlitePool, SqliteConnectOptions};
use std::str::FromStr;

pub struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = SqlitePool;
}

pub async fn get_sqlite_pool<S: AsRef<str>>(sqlite_url: S) -> Result<SqlitePool, Box<dyn std::error::Error + Send + Sync>>{
    let connect_options = SqliteConnectOptions::from_str(sqlite_url.as_ref())?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(connect_options).await?;
    Ok(pool)
}