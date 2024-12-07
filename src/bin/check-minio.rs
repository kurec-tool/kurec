use s3::{creds::Credentials, Bucket};
use std::error::Error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let bucket_name = "test";
    let credentials = Credentials::new(Some("minio"), Some("minio123"), None, None, None)?;
    let bucket = Bucket::new(
        bucket_name,
        s3::Region::Custom {
            region: "".to_owned(),
            endpoint: "http://minio:9000".to_owned(),
        },
        credentials,
    )?
    .with_path_style();

    bucket
        .put_object("test.txt", "Hello, world!".as_bytes())
        .await?;

    Ok(())
}
