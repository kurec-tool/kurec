use anyhow::Result;
use futures::StreamExt;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let events_url = "http://tuner:40772/events";
    let resp = reqwest::get(events_url).await.unwrap();
    let mut bytes_stream = resp.bytes_stream();

    loop {
        let item = bytes_stream.next().await;
        info!("item: {:?}", item);
    }
}
