use domain::event::Event; // 新しい Event トレイトをインポート
use domain::ports::{event_sink::EventSink, event_source::EventSource}; // パス修正
use futures::StreamExt;
use infra_jetstream::{EventStream, JsPublisher, JsSubscriber};
use infra_nats::connect as nats_connect;
use serde::{Deserialize, Serialize};
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent {
    pub id: usize,
    pub message: String,
}

// 新しい Event トレイトを実装
impl Event for TestEvent {}

// テスト用のストリーム定義
const TEST_STREAM_NAME: &str = "test-pubsub-stream";

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

    // EventStream を作成
    let event_stream = EventStream::new(
        TEST_STREAM_NAME,
        infra_jetstream::config::StreamConfig {
            max_age: None,
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

    // NatsClient と EventStream を渡す
    let publisher = JsPublisher::<TestEvent>::new(nats_client.clone(), event_stream.clone());
    let subscriber = JsSubscriber::<TestEvent>::new(nats_client.clone(), event_stream);

    let test_event = TestEvent {
        id: 1,
        message: "Hello, JetStream!".to_string(),
    };

    publisher.publish(test_event.clone()).await?;

    let mut stream = subscriber.subscribe().await?;

    let received = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        if let Some(Ok(event)) = stream.next().await {
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
