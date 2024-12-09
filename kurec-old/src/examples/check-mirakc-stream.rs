use anyhow::Result;
use bytes::Bytes;
use futures::{
    io,
    stream::{self, StreamExt},
    Stream, TryStreamExt,
};
use s3::Bucket;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let bucket_name = "test";
    let credentials =
        s3::creds::Credentials::new(Some("minio"), Some("minio123"), None, None, None)?;
    let bucket = s3::Bucket::new(
        bucket_name,
        s3::Region::Custom {
            region: "".to_owned(),
            endpoint: "http://minio:9000".to_owned(),
        },
        credentials,
    )?
    .with_path_style();

    match get_dummy_record_id_stream().await {
        Ok(mut stream) => {
            while let Some(record_id) = stream.next().await {
                println!("{}", record_id);

                save_record_to_s3(tuner_url, &bucket, &record_id).await?;
            }
            error!("mirakc events stream ended");
        }
        Err(e) => {
            error!("Error: {:?}", e);
            panic!("Error");
        }
    };
    Ok(())
}

async fn save_record_to_s3(
    tuner_url: &str,
    bucket: &Bucket,
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
    let stream = resp.bytes_stream().map_err(std::io::Error::other);
    let mut reader = StreamReader::new(stream);
    let mut writer = tokio::fs::File::create(format!("{}.ts", record_id)).await?;

    let s3_path = format!("/{}.ts", record_id);
    let content_type = "video/MP2T";
    let resp = bucket
        .put_object_stream_with_content_type(&mut reader, s3_path, content_type)
        .await?;
    println!("save to s3 done resp:{:?}", resp);
    Ok(())
}

async fn get_dummy_record_id_stream() -> Result<impl Stream<Item = String>, std::io::Error> {
    let record_ids = vec![
        "000001939DE15AD7000129BDCAB62D56".to_string(),
        "000001939DE0F3D000012698B934F5A4".to_string(),
        "000001939DCB8CDA000129BDCAB62D53".to_string(),
    ];

    Ok(stream::iter(record_ids.into_iter()))
}
