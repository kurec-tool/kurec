use anyhow::Result;
use bytes::Bytes;
use futures::stream::{IntoAsyncRead, StreamExt, TryStreamExt};
use kurec::{adapter::sse_stream::get_sse_record_id_stream, config::KurecConfig};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

pub async fn run_stream_record(config: KurecConfig) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let bucket_name = "kurec-record"; // TODO: ユーザがPrefix変えられるようにする
    let stream_name = "kurec-encode"; // TODO: ユーザがPrefix変えられるようにする
    let tuner_url = "http://tuner:40772";
    let nats_url = "nats:4222";
    let client = async_nats::connect(nats_url).await?;
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

    let region = "";
    let endpoint_url = "http://minio:9000";
    let access_key = "minio";
    let secret_key = "minio123";

    let cred = s3::creds::Credentials::new(
        config.minio_access_key.as_deref(),
        config.minio_secret_key.as_deref(),
        None,
        None,
        None,
    )?;

    let mut bucket = s3::Bucket::new(
        &config.minio_record_bucket_name,
        s3::Region::Custom {
            region: config.minio_region.clone(),
            endpoint: config.minio_url.clone(),
        },
        cred,
    )?
    .with_path_style();

    info!("start loop");

    match get_sse_record_id_stream(tuner_url).await {
        Ok(mut stream) => {
            while let Some(record_id) = stream.next().await {
                info!("record received: {}", record_id);

                save_record_to_s3(tuner_url, &bucket, &record_id).await?;
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

async fn save_record_to_s3(
    tuner_url: &str,
    bucket: &s3::Bucket,
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
    let s3_path = format!("record/{}.ts", record_id);
    let content_type = "video/MP2T";
    let mut reader = tokio_util::io::StreamReader::new(stream);
    let resp = bucket
        .put_object_stream_with_content_type(&mut reader, s3_path, content_type)
        .await?;
    debug!("put_object_stream_with_content_type: {:?}", resp);
    Ok(())
}
