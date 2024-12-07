use anyhow::Result;
use futures::stream::StreamExt;
use kurec::{
    adapter::sse_stream::get_sse_record_id_stream, config::KurecConfig,
    message::jetstream_message::OnRecordingFinished,
};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

pub async fn run_sse_record(config: KurecConfig, tuner_url: &str) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let stream_name = "kurec-record"; // TODO: ユーザがPrefix変えられるようにする
    let tuner_url = "http://tuner:40772";
    let nats_url = "nats:4222";
    let client = async_nats::connect(nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);
    // consumer作るときにはstreamいるけど、publishだけならいらないが、
    // Config設定する必要があるのでget_or_create_streamを使う
    let _ = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream_name.to_string(),
            max_messages: 10_000_000,
            // TODO: Config調整
            ..Default::default()
        })
        .await?;

    info!("start loop");

    match get_sse_record_id_stream(tuner_url).await {
        Ok(mut stream) => {
            while let Some(record_id) = stream.next().await {
                info!("record received: {}", record_id);
                let message = OnRecordingFinished {
                    tuner_url: tuner_url.to_string(),
                    record_id,
                };
                let message = serde_json::to_string(&message)?;
                jetstream.publish(stream_name, message.into()).await?;
            }
            error!("mirakc events stream ended");

            Err(anyhow::anyhow!("mirakc events stream ended"))
        }
        Err(e) => {
            error!("Error: {:?}", e);
            Err(e)
        }
    }
}
