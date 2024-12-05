use anyhow::Result;
use async_nats::jetstream::consumer::PullConsumer;
use futures::StreamExt;
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

    let stream_name = "kurec-epg"; // TODO: ユーザがPrefix変えられるようにする

    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream_name.to_string(),
            max_messages: 10_000_000,
            ..Default::default()
        })
        .await?;

    let consumer: PullConsumer = stream
        .get_or_create_consumer(
            "kurec-epg-rule-meilisearch", // この名前の意味は？
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("kurec-epg-rule-meilisearch".to_string()),
                // TODO: Config調整
                ..Default::default()
            },
        )
        .await?;

    let mut messages = consumer.messages().await?;
    loop {
        match messages.next().await {
            Some(Ok(msg)) => {
                let value = String::from_utf8_lossy(msg.payload.as_ref());
                msg.ack().await.unwrap();
            }
            Some(Err(e)) => {
                error!("Error when reading messages: {:?}", e);
            }
            None => {
                error!("No more messages");
                break;
            }
        }
    }

    error!("End of the program");

    Ok(())
}
