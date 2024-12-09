use anyhow::Result;
use futures::StreamExt;
use kurec::adapter::{mirakc::get_record_stream_reader, sse_stream::get_sse_record_id_stream};
use s3::Bucket;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let tuner_url = "http://tuner:40772";
    let ts_stream = get_record_stream_reader(tuner_url, "000001939DCB8CDA000129BDCAB62D53");
    Ok(())
}

async fn save_record_to_s3(
    tuner_url: &str,
    bucket: &Bucket,
    record_id: &str,
) -> Result<(), anyhow::Error> {
    let mut ts_stream = get_record_stream_reader(tuner_url, record_id).await?;
    let s3_path = format!("{}.ts", record_id);
    let content_type = "video/MP2T";
    let resp = bucket
        .put_object_stream_with_content_type(&mut ts_stream, s3_path, content_type)
        .await?;
    println!("save to s3 done resp:{:?}", resp);
    Ok(())
}
