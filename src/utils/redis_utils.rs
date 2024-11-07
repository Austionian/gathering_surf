use bb8::Pool;
use bb8_redis::RedisConnectionManager;

pub async fn get(key: &str, pool: &Pool<RedisConnectionManager>) -> Option<String> {
    let mut conn = pool.get().await.ok()?;

    redis::cmd("GET")
        .arg(key)
        .query_async(&mut *conn)
        .await
        .expect("failed to execute GET")
}

pub async fn set(key: &str, data: &str, pool: &Pool<RedisConnectionManager>) -> anyhow::Result<()> {
    let mut conn = pool.get().await?;

    tracing::info!("setting redis value");
    let _: () = redis::cmd("PSETEX")
        .arg(key)
        .arg("100000")
        .arg(data)
        .query_async(&mut *conn)
        .await
        .expect("failed to execute SET");

    Ok(())
}
