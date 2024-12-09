use futures::stream::TryStreamExt;
use futures_util::StreamExt;
use reqwest::Client;
use s3::creds::Credentials;
use s3::Bucket;
use s3::Region;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // HTTPクライアントとURL設定
    let client = Client::new();
    let url = "http://tuner:40772/api/recording/records/000001939DE15AD7000129BDCAB62D56";

    // S3バケットの設定
    let bucket_name = "test";
    let region = Region::Custom {
        region: "".into(),
        endpoint: "http://minio:9000".into(),
    };
    let credentials = Credentials::new(Some("minio"), Some("minio123"), None, None, None)?;
    let bucket = Bucket::new(bucket_name, region, credentials)?.with_path_style();

    // HTTPレスポンスを取得
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(format!("Failed to fetch file: {}", response.status()).into());
    }

    // HTTPレスポンスボディをストリーム化
    let stream = response
        .bytes_stream()
        .map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));

    // StreamをAsyncReadに変換
    let mut stream_reader = StreamReader::new(stream);

    // S3にアップロード
    let key = "000001939DE15AD7000129BDCAB62D56.ts";
    let result = bucket.put_object_stream(&mut stream_reader, key).await?;

    println!("Upload completed: {:?}", result);
    Ok(())
}
