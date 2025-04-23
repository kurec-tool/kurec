use std::time::Duration;

use domain::event::Event; // 新しい Event トレイトをインポート
use infra_jetstream::{setup_all_streams, EventStream}; // EventStream をインポート
use infra_nats::connect as nats_connect;
use serde::{Deserialize, Serialize};
use testcontainers::{core::WaitFor, runners::AsyncRunner, GenericImage, ImageExt};

// イベント型の定義
#[derive(Serialize, Deserialize, Debug)]
struct TestEvent;

// 新しい Event トレイトを実装
impl Event for TestEvent {}

// テスト用のストリーム定義
const TEST_STREAM_NAME: &str = "test-stream";

async fn ensure_docker() {
    for _ in 0..20 {
        if std::process::Command::new("docker")
            .arg("info")
            .output()
            .is_ok()
        {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    panic!("Docker daemon not ready");
}

#[tokio::test]
async fn stream_defs_are_applied() -> anyhow::Result<()> {
    ensure_docker().await;

    // ---- Spin‑up test JetStream -------------------------------------------
    let container = GenericImage::new("nats", "latest")
        .with_exposed_port(4222u16.into())
        .with_wait_for(WaitFor::message_on_stderr("Server is ready"))
        .with_cmd(vec!["--js"])
        .with_env_var("NATS_SERVER", "jetstream")
        .with_env_var("NATS_JETSTREAM", "true")
        .with_env_var("NATS_STREAMS", "test-stream")
        .with_env_var("NATS_STREAMS_TEST_STREAM", "test-stream")
        .with_env_var("NATS_STREAMS_TEST_SUBJECT", "test.subject")
        .with_env_var("NATS_STREAMS_TEST_MAX_DELIVER", "5")
        .with_env_var("NATS_STREAMS_TEST_ACK_WAIT", "1m")
        .with_env_var("NATS_STREAMS_TEST_MAX_AGE", "1h")
        .start()
        .await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(4222u16).await?;
    let url = format!("nats://{}:{}", host, port);

    // ---- Use infra_nats::connect to get NatsClient ----------------------
    let nats_client = nats_connect(&url).await?;
    let js = nats_client.jetstream_context();

    // テスト用のEventStreamを作成
    let test_stream = EventStream::new(
        TEST_STREAM_NAME,
        infra_jetstream::config::StreamConfig {
            max_age: Some(Duration::from_secs(3600)), // 1時間
            max_messages: None,
            max_bytes: None,
            max_message_size: None,
            storage: None,
            retention: None,
            discard: None,
            duplicate_window: None,
            allow_rollup: None,
            deny_delete: None,
            deny_purge: None,
            description: None,
        },
    );

    // ---- Apply all StreamDefs ---------------------------------------------
    println!("setup_all_streams calling...");
    setup_all_streams(js).await?;
    println!("setup_all_streams done");

    // ---- Assert Stream exists --------------------------------
    assert!(
        js.get_stream_no_info(test_stream.stream_name())
            .await
            .is_ok(),
        "stream {} should exist",
        test_stream.stream_name()
    );

    Ok(())
}

#[tokio::test]
async fn idempotend_check() -> anyhow::Result<()> {
    ensure_docker().await;

    // ---- Spin‑up test JetStream -------------------------------------------
    let container = GenericImage::new("nats", "latest")
        .with_exposed_port(4222u16.into())
        .with_wait_for(WaitFor::message_on_stderr("Server is ready"))
        .with_cmd(vec!["--js"])
        .with_env_var("NATS_SERVER", "jetstream")
        .with_env_var("NATS_JETSTREAM", "true")
        .with_env_var("NATS_STREAMS", "test-stream")
        .with_env_var("NATS_STREAMS_TEST_STREAM", "test-stream")
        .with_env_var("NATS_STREAMS_TEST_SUBJECT", "test.subject")
        .with_env_var("NATS_STREAMS_TEST_MAX_DELIVER", "5")
        .with_env_var("NATS_STREAMS_TEST_ACK_WAIT", "1m")
        .with_env_var("NATS_STREAMS_TEST_MAX_AGE", "1h")
        .start()
        .await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(4222u16).await?;
    let url = format!("nats://{}:{}", host, port);

    // ---- Use infra_nats::connect to get NatsClient ----------------------
    let nats_client = nats_connect(&url).await?;
    let js = nats_client.jetstream_context();

    // テスト用のEventStreamを作成
    let test_stream = EventStream::new(
        TEST_STREAM_NAME,
        infra_jetstream::config::StreamConfig {
            max_age: Some(Duration::from_secs(3600)), // 1時間
            max_messages: None,
            max_bytes: None,
            max_message_size: None,
            storage: None,
            retention: None,
            discard: None,
            duplicate_window: None,
            allow_rollup: None,
            deny_delete: None,
            deny_purge: None,
            description: None,
        },
    );

    // ---- Apply all StreamDefs ---------------------------------------------
    setup_all_streams(js).await?;
    setup_all_streams(js).await?;

    // ---- Assert Stream exists --------------------------------
    assert!(
        js.get_stream_no_info(test_stream.stream_name())
            .await
            .is_ok(),
        "stream {} should exist",
        test_stream.stream_name()
    );

    Ok(())
}
