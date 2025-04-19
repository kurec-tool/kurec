use std::time::Duration;

use infra_jetstream::{connect, setup_all_streams};
use serde::{Deserialize, Serialize};
use shared_core::streams::get_all_stream_configs;
use shared_macros::{define_streams, event};
use testcontainers::{core::WaitFor, runners::AsyncRunner, GenericImage, ImageExt};

// ストリーム定義
define_streams! {
    stream test_stream {
        max_age: "1h",
        max_deliver: 5,
        ack_wait: "1m",
    }
}

// イベント型の定義
#[event(stream = "test_stream")]
#[derive(Serialize, Deserialize, Debug)]
struct TestEvent;

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
    let url = format!("{}:{}", host, port);

    // ---- Use our helper to connect ----------------------------------------
    let ctx = connect(&url).await?;
    let js = &ctx.js;

    // ---- Apply all StreamDefs ---------------------------------------------
    println!("setup_all_streams calling...");
    setup_all_streams(js).await?;
    println!("setup_all_streams done");

    // 登録されたストリーム設定を取得
    let configs = get_all_stream_configs();
    assert_eq!(configs.len(), 1, "should have exactly one stream config");

    // ---- Assert every Stream now exists --------------------------------
    for config in configs {
        assert!(
            js.get_stream_no_info(&config.name).await.is_ok(),
            "stream {} should exist",
            config.name
        );
        assert_eq!(
            js.get_stream(&config.name)
                .await?
                .info()
                .await?
                .config
                .max_age,
            Duration::from_secs(3600),
        );
    }

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
    let url = format!("{}:{}", host, port);

    // ---- Use our helper to connect ----------------------------------------
    let ctx = connect(&url).await?;
    let js = &ctx.js;

    // ---- Apply all StreamDefs ---------------------------------------------
    setup_all_streams(js).await?;
    setup_all_streams(js).await?;

    // ---- Assert every Stream now exists --------------------------------
    let configs = get_all_stream_configs();
    for config in configs {
        assert!(
            js.get_stream_no_info(&config.name).await.is_ok(),
            "stream {} should exist",
            config.name
        );
    }

    Ok(())
}
