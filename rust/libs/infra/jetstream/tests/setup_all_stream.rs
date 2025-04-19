use std::time::Duration;

use infra_jetstream::{connect, setup_all_streams};
use shared_core::event_metadata::StreamDef;
use shared_macros::event;
use testcontainers::{core::WaitFor, runners::AsyncRunner, GenericImage, ImageExt};

#[event(
    stream = "test-stream",
    subject = "test.subject",
    max_deliver = 5,
    ack_wait = "1m",
    max_age = "1h"
)]
struct _TestEvent;

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

    let defs: Vec<_> = shared_core::event_metadata::inventory::iter::<StreamDef>
        .into_iter()
        .collect();
    assert_eq!(defs.len(), 1, "should have exactly one stream def");

    // ---- Assert every StreamDef now exists --------------------------------
    for def in defs {
        assert!(
            js.get_stream_no_info(def.name).await.is_ok(),
            "stream {} should exist",
            def.name
        );
        assert_eq!(
            js.get_stream(def.name).await?.info().await?.config.max_age,
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

    // ---- Assert every StreamDef now exists --------------------------------
    for def in shared_core::event_metadata::inventory::iter::<StreamDef> {
        assert!(
            js.get_stream_no_info(def.name).await.is_ok(),
            "stream {} should exist",
            def.name
        );
    }

    Ok(())
}
