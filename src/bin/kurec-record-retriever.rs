use anyhow::Result;
use futures::stream::StreamExt;
use kurec::adapter::{mirakc::get_record_stream_reader, sse_stream::get_sse_record_id_stream};
use s3::Bucket;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
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

    let credentials =
        s3::creds::Credentials::new(Some("minio"), Some("minio123"), None, None, None).unwrap();
    let bucket = s3::Bucket::new(
        bucket_name,
        s3::Region::Custom {
            region: "".to_owned(),
            endpoint: "http://minio:9000".to_owned(),
        },
        credentials,
    )
    .unwrap()
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
    bucket: &Bucket,
    record_id: &str,
) -> Result<(), anyhow::Error> {
    let mut ts_stream = get_record_stream_reader(tuner_url, record_id).await?;
    let s3_path = format!("{}.ts", record_id);
    let content_type = "video/MP2T";
    debug!(
        "start save to s3 host:{} bucket:{} path:{}",
        bucket.host(),
        bucket.name(),
        s3_path
    );
    let resp = bucket
        .put_object_stream_with_content_type(&mut ts_stream, &s3_path, content_type)
        .await?;
    debug!("save to s3 done resp:{:?}", resp);
    Ok(())
}
