use std::time::Duration;

use anyhow::Result;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let nats_url = "nats:4222";
    let client = async_nats::connect(nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);
    let kv = match jetstream.get_key_value("test").await {
        Ok(kv) => Ok(kv),
        Err(e) => {
            error!("Error: {:?}", e);
            jetstream
                .create_key_value(async_nats::jetstream::kv::Config {
                    bucket: "test".to_string(),
                    max_age: Duration::from_secs(30),
                    history: 3,
                    ..Default::default()
                })
                .await
        }
    };

    dbg!(&kv);

    Ok(())
}
