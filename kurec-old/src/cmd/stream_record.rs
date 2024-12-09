use crate::{
    adapter::{mirakc::delete_record, sse_stream::get_sse_record_id_stream},
    config::KurecConfig,
};
use anyhow::Result;
use async_nats::jetstream::object_store::{self, ObjectMetadata, ObjectStore};
use bytes::Bytes;
use futures::StreamExt;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

pub async fn run_stream_record(config: KurecConfig) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let tuner_url = "http://tuner:40772"; // TODO: メッセージから取得するようにする

    let bucket_name = "kurec-record"; // TODO: ユーザがPrefix変えられるようにする
    let stream_name = "kurec-encode"; // TODO: ユーザがPrefix変えられるようにする
    let client = async_nats::connect(&config.nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);
    let kv = match jetstream.get_key_value(bucket_name.to_string()).await {
        Ok(kv) => kv,
        Err(e) => {
            error!("Error: {:?}", e);
            jetstream
                .create_key_value(async_nats::jetstream::kv::Config {
                    bucket: bucket_name.to_string(),
                    max_age: std::time::Duration::from_secs(30 * 24 * 60 * 60), // 30 days
                    history: 10,                                                // 適当
                    // TODO: パラメータ調整
                    ..Default::default()
                })
                .await?
        }
    };
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
    let object_store = jetstream
        .create_object_store(object_store::Config {
            bucket: config.get_record_object_store_name().clone(),
            ..Default::default()
        })
        .await?;

    info!("start loop");

    match get_sse_record_id_stream(tuner_url).await {
        Ok(mut stream) => {
            while let Some(record_id) = stream.next().await {
                info!("record received: {}", record_id);

                save_record_to_object_store(tuner_url, &object_store, &record_id).await?;
                debug!("save done.");
                delete_record(tuner_url, &record_id).await?;
                debug!("delete done.");
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

async fn save_record_to_object_store(
    tuner_url: &str,
    object_store: &ObjectStore,
    record_id: &str,
) -> Result<(), anyhow::Error> {
    let url = format!("{}/api/recording/records/{}/stream", tuner_url, record_id);
    let resp = reqwest::get(&url).await?;
    if resp.status() != 200 {
        return Err(anyhow::anyhow!(
            "Failed to get record stream response: {}",
            resp.status()
        ));
    }
    let stream = resp.bytes_stream().map(|b| match b {
        Ok(bytes) => Ok::<Bytes, std::io::Error>(bytes),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    });
    let mut reader = tokio_util::io::StreamReader::new(stream);
    let name = format!("record/{}.ts", record_id);
    let metadata = ObjectMetadata {
        name: name.clone(),
        ..Default::default()
    };
    let resp = object_store.put(metadata, &mut reader).await?;
    debug!("put to : {} {:?}", name, resp);
    Ok(())
}
