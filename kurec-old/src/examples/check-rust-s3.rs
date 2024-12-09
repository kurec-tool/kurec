use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let ret = main1().await;
    match ret {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

async fn main1() -> Result<(), anyhow::Error> {
    let bucket_name = "test";
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

    // let path = "path";
    // let test: Vec<u8> = (0..1000).map(|_| 42).collect();
    // let mut file = File::create(path).unwrap();
    // file.write_all(&test).unwrap();

    let mut async_input_file = tokio::fs::File::open(".devcontainer/recorded/323912360850852.ts")
        .await
        .expect("Unable to open file");
    dbg!(&async_input_file);

    // Async variant with `tokio` or `async-std` features
    // Generic over std::io::Read
    let status_code = bucket
        .put_object_stream_with_content_type(&mut async_input_file, "path2", "video/MP2T")
        .await
        .unwrap();

    // `sync` feature will produce an identical method
    #[cfg(feature = "sync")]
    // Generic over std::io::Read
    let status_code = bucket
        .put_object_stream_with_content_type(&mut path, "/path", "application/octet-stream")
        .unwrap();

    // Blocking variant, generated with `blocking` feature in combination
    // with `tokio` or `async-std` features.
    #[cfg(feature = "blocking")]
    let status_code = bucket
        .put_object_stream_with_content_type_blocking(
            &mut path,
            "/path",
            "application/octet-stream",
        )
        .unwrap();

    Ok(())
}
