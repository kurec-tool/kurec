use std::sync::Arc;
// use std::sync::Arc; // 重複削除
use std::time::Duration;

use futures::StreamExt;
use infra_jetstream::{JsPublisher, JsSubscriber}; // connect を削除
use infra_nats::connect as nats_connect; // infra_nats::connect をインポート
use serde::{Deserialize, Serialize};
use shared_core::event_metadata::Event;
use shared_core::event_sink::EventSink; // リネーム
use shared_core::event_source::EventSource; // リネーム
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt}; // ContainerAsyncをインポート

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent {
    pub id: usize,
    pub message: String,
}

impl Event for TestEvent {
    fn stream_name() -> &'static str {
        "test-pubsub-stream"
    }

    fn event_name() -> &'static str {
        "test-event"
    }
}

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
async fn test_publisher_subscriber() -> anyhow::Result<()> {
    let (_container, url) = setup_nats().await?;

    // infra_nats::connect を使用
    let nats_client = nats_connect(&url).await?;
    let js = nats_client.jetstream_context(); // NatsClient から JetStream Context を取得

    let _stream = js
        .create_stream(&async_nats::jetstream::stream::Config {
            name: TestEvent::stream_name().to_string(),
            subjects: vec![TestEvent::stream_subject().to_string()],
            ..Default::default()
        })
        .await?;

    // Removed manual consumer creation to let JsSubscriber create its own consumer.

    // NatsClient を渡すように変更
    let publisher = JsPublisher::<TestEvent>::from_event_type(nats_client.clone());
    let subscriber = JsSubscriber::<TestEvent>::from_event_type(nats_client.clone());

    let test_event = TestEvent {
        id: 1,
        message: "Hello, JetStream!".to_string(),
    };

    publisher.publish(test_event.clone()).await?;

    let mut stream = subscriber.subscribe().await?;

    let received = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        if let Some((event, ack)) = stream.next().await {
            ack.ack().await?;
            Ok::<_, anyhow::Error>(event)
        } else {
            Err(anyhow::anyhow!("Stream ended unexpectedly"))
        }
    })
    .await??;

    assert_eq!(received.id, test_event.id);
    assert_eq!(received.message, test_event.message);

    Ok(())
}

#[tokio::test]
async fn test_from_event_type() -> anyhow::Result<()> {
    let (_container, url) = setup_nats().await?;

    // infra_nats::connect を使用
    let nats_client = nats_connect(&url).await?;
    let js = nats_client.jetstream_context(); // NatsClient から JetStream Context を取得

    let _stream = js
        .create_stream(&async_nats::jetstream::stream::Config {
            name: TestEvent::stream_name().to_string(),
            subjects: vec![TestEvent::stream_subject().to_string()],
            ..Default::default()
        })
        .await?;

    // NatsClient を渡すように変更
    let publisher = JsPublisher::<TestEvent>::from_event_type(nats_client.clone());
    let subscriber = JsSubscriber::<TestEvent>::from_event_type(nats_client.clone());

    let test_event = TestEvent {
        id: 2,
        message: "Hello from from_event_type!".to_string(),
    };

    publisher.publish(test_event.clone()).await?;

    let mut stream = subscriber.subscribe().await?;

    let received = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        if let Some((event, ack)) = stream.next().await {
            ack.ack().await?;
            Ok::<_, anyhow::Error>(event)
        } else {
            Err(anyhow::anyhow!("Stream ended unexpectedly"))
        }
    })
    .await??;

    assert_eq!(received.id, test_event.id);
    assert_eq!(received.message, test_event.message);

    Ok(())
}

async fn setup_nats() -> anyhow::Result<(ContainerAsync<GenericImage>, String)> {
    ensure_docker().await;
    // ---- Spin‑up test JetStream -------------------------------------------
    let container = GenericImage::new("nats", "latest")
        .with_exposed_port(4222u16.into())
        .with_wait_for(WaitFor::message_on_stderr("Server is ready"))
        .with_cmd(vec!["--js"])
        .start()
        .await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(4222u16).await?;
    let url = format!("nats://{}:{}", host, port); // スキームを追加
    Ok((container, url))
}
