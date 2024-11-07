use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, SetExpiry, SetOptions};

/// Gets the value from the Redis cache if it exists.
pub async fn get(key: &str, pool: &Pool<RedisConnectionManager>) -> Option<String> {
    pool.get().await.ok()?.get(key).await.ok()
}

/// Sets the given k,v pair for 5 minutes in the Redis cache.
pub async fn set(
    key: &str,
    value: &str,
    pool: &Pool<RedisConnectionManager>,
) -> anyhow::Result<()> {
    let opts = SetOptions::default().with_expiration(SetExpiry::EX(300));
    pool.get()
        .await?
        .set_options::<&str, &str, ()>(key, value, opts)
        .await?;

    Ok(())
}
