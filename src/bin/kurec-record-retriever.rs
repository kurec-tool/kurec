use anyhow::Result;
use bytes::Bytes;
use futures::stream::StreamExt;
use kurec::adapter::sse_stream::get_sse_record_id_stream;
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

    let region = "";
    let endpoint_url = "http://minio:9000";
    let access_key = "minio";
    let secret_key = "minio123";

    let cred_provider = AwsCredentials::new(Some(access_key), Some(secret_key), None, None);

    let s3_config = aws_types::SdkConfig::builder()
        .endpoint_url(endpoint_url)
        .region(aws_sdk_s3::config::Region::from_static(region))
        .credentials_provider(cred_provider)
        .build();
    let s3_client = aws_sdk_s3::Client::new(&s3_config);

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
    client: &aws_sdk_s3::Client,
    bucket_name: &str,
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
    let body = hyper::Body::wrap_stream(stream);

    let s3_path = format!("{}.ts", record_id);
    let content_type = "video/MP2T";
    debug!("start save to s3 bucket:{} path:{}", bucket_name, s3_path);
    let request = client
        .put_object()
        .bucket(bucket_name)
        .key(&s3_path)
        .body(SdkBody::from(body))
        .send()
        .await?;
    Ok(())
}
