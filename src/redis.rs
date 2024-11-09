use crate::configuration::Settings;
use fred::clients::RedisPool;
use fred::error::RedisError;
use fred::interfaces::ClientLike;
use fred::types::{Builder, RedisConfig};
use std::time::Duration;

pub async fn get_redis_connection(config: &Settings) -> Result<RedisPool, RedisError> {
    let redis_config = RedisConfig::from_url(&config.redis.url)?;

    let pool = Builder::from_config(redis_config)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(5);
        })
        .build_pool(config.redis.pool_size)?;

    pool.init().await?;

    Ok(pool)
}
