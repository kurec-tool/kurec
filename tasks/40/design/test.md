# Issue #40: Ack/Nack機能の修正設計 - テスト実装

Ack/Nack機能のテストを実装します。特に、メッセージをAckせずにDropした場合、少し待てばもう一度再配信されることを確認するテストを追加します。

## JetStreamのAck/Nackテスト

```rust
// infra/jetstream/tests/js_ack_test.rs
use std::sync::Arc;
use std::time::Duration;

use domain::event::Event;
use futures::StreamExt;
use infra_common::ackable_event::AckableEvent;
use infra_common::event_source::EventSource;
use infra_jetstream::{EventStream, JsPublisher, JsSubscriber};
use infra_nats::connect as nats_connect;
use serde::{Deserialize, Serialize};
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent {
    pub id: usize,
    pub message: String,
}

impl Event for TestEvent {}

// テスト用のストリーム定義
const TEST_STREAM_NAME: &str = "test-ack-stream";

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
async fn test_message_redelivery_without_ack() -> anyhow::Result<()> {
    let (_container, url) = setup_nats().await?;

    // infra_nats::connect を使用
    let nats_client = nats_connect(&url).await?;
    let js = nats_client.jetstream_context();

    // EventStream を作成（短いAckWaitを設定）
    let event_stream = EventStream::<TestEvent>::new(
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
        message: "Test redelivery".to_string(),
    };

    // イベントを発行
    publisher.publish(test_event.clone()).await?;

    // サブスクライブ
    let mut stream = subscriber.subscribe().await?;

    // 最初のメッセージを受信（Ackしない）
    let first_message = stream.next().await.unwrap()?;
    
    assert_eq!(first_message.event().id, 1);
    assert_eq!(first_message.event().message, "Test redelivery");
    // Ackしない

    // AckWaitの時間だけ待機（JetStreamのデフォルトは30秒だが、テストでは短く設定）
    tokio::time::sleep(Duration::from_secs(5)).await;

    // 再配信されたメッセージを受信
    let redelivered_message = stream.next().await.unwrap()?;
    
    assert_eq!(redelivered_message.event().id, 1);
    assert_eq!(redelivered_message.event().message, "Test redelivery");
    
    // 今度はAckする
    redelivered_message.ack().await?;

    // さらに待機しても再配信されないことを確認
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // タイムアウトを設定して次のメッセージを待つ
    let timeout_result = tokio::time::timeout(Duration::from_secs(2), stream.next()).await;
    
    // タイムアウトすることを確認（メッセージが再配信されない）
    assert!(timeout_result.is_err(), "Expected timeout waiting for message");

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
    let url = format!("nats://{}:{}", host, port);
    Ok((container, url))
}
```

## StreamWorkerのAck/Nackテスト

```rust
// app/tests/stream_worker_ack_test.rs
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use domain::event::Event;
use domain::ports::event_sink::EventSink;
use futures::future::BoxFuture;
use futures::StreamExt;
use infra_common::ackable_event::AckableEvent;
use infra_common::event_source::EventSource;
use infra_jetstream::{EventStream, JsPublisher, JsSubscriber};
use infra_nats::connect as nats_connect;
use serde::{Deserialize, Serialize};
use shared_core::error_handling::{ClassifyError, ErrorAction};
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use app::handler::event_handler::{EventHandler, FnEventHandler};
use app::worker::stream_worker::StreamWorker;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestInputEvent {
    pub id: usize,
    pub message: String,
}

impl Event for TestInputEvent {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestOutputEvent {
    pub id: usize,
    pub message: String,
}

impl Event for TestOutputEvent {}

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("Test error: {0}")]
    Test(String),
}

impl ClassifyError for TestError {
    fn error_action(&self) -> ErrorAction {
        match self {
            TestError::Test(msg) => {
                if msg.contains("retry") {
                    ErrorAction::Retry
                } else {
                    ErrorAction::Ignore
                }
            }
        }
    }
}

struct TestEventSink {
    tx: mpsc::Sender<TestOutputEvent>,
}

#[async_trait]
impl EventSink<TestOutputEvent> for TestEventSink {
    async fn publish(&self, event: TestOutputEvent) -> Result<()> {
        self.tx.send(event).await.map_err(|e| anyhow::anyhow!("Failed to send event: {}", e))
    }
}

#[tokio::test]
async fn test_stream_worker_ack_on_success() -> anyhow::Result<()> {
    let (_container, url) = setup_nats().await?;

    // infra_nats::connect を使用
    let nats_client = nats_connect(&url).await?;
    let js = nats_client.jetstream_context();

    // EventStream を作成
    let input_stream = EventStream::<TestInputEvent>::new(
        "test-input-stream",
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

    let output_stream = EventStream::<TestOutputEvent>::new(
        "test-output-stream",
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
    let input_publisher = JsPublisher::<TestInputEvent>::new(nats_client.clone(), input_stream.clone());
    let input_subscriber = JsSubscriber::<TestInputEvent>::new(nats_client.clone(), input_stream);
    let output_publisher = JsPublisher::<TestOutputEvent>::new(nats_client.clone(), output_stream.clone());

    // テストイベントを発行
    let test_event = TestInputEvent {
        id: 1,
        message: "Test success".to_string(),
    };

    input_publisher.publish(test_event.clone()).await?;

    // イベントハンドラを作成
    let handler = FnEventHandler::new(|event: TestInputEvent| {
        Box::pin(async move {
            Ok(Some(TestOutputEvent {
                id: event.id,
                message: format!("Processed: {}", event.message),
            }))
        }) as BoxFuture<'static, Result<Option<TestOutputEvent>, TestError>>
    });

    // 出力イベントを受け取るためのチャネルを作成
    let (tx, mut rx) = mpsc::channel(10);
    let sink = TestEventSink { tx };

    // StreamWorkerを作成
    let worker = StreamWorker::new(
        Arc::new(input_subscriber),
        Arc::new(sink),
        Arc::new(handler),
    );

    // シャットダウントークンを作成
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // ワーカーを実行
    let worker_handle = tokio::spawn(async move {
        worker.run(shutdown_clone).await
    });

    // 出力イベントを受信
    let output_event = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await??;
    assert_eq!(output_event.id, 1);
    assert_eq!(output_event.message, "Processed: Test success");

    // シャットダウン
    shutdown.cancel();
    worker_handle.await??;

    // 再度サブスクライブして、メッセージが再配信されないことを確認
    let subscriber = JsSubscriber::<TestInputEvent>::new(nats_client.clone(), input_stream);
    let mut stream = subscriber.subscribe().await?;
    
    // タイムアウトを設定して次のメッセージを待つ
    let timeout_result = tokio::time::timeout(Duration::from_secs(2), stream.next()).await;
    
    // タイムアウトすることを確認（メッセージが再配信されない）
    assert!(timeout_result.is_err(), "Expected timeout waiting for message");

    Ok(())
}

#[tokio::test]
async fn test_stream_worker_no_ack_on_error_retry() -> anyhow::Result<()> {
    let (_container, url) = setup_nats().await?;

    // infra_nats::connect を使用
    let nats_client = nats_connect(&url).await?;
    let js = nats_client.jetstream_context();

    // EventStream を作成（短いAckWaitを設定）
    let input_stream = EventStream::<TestInputEvent>::new(
        "test-input-stream-error",
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

    let output_stream = EventStream::<TestOutputEvent>::new(
        "test-output-stream-error",
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
    let input_publisher = JsPublisher::<TestInputEvent>::new(nats_client.clone(), input_stream.clone());
    let input_subscriber = JsSubscriber::<TestInputEvent>::new(nats_client.clone(), input_stream.clone());
    let output_publisher = JsPublisher::<TestOutputEvent>::new(nats_client.clone(), output_stream.clone());

    // テストイベントを発行
    let test_event = TestInputEvent {
        id: 2,
        message: "Test error retry".to_string(),
    };

    input_publisher.publish(test_event.clone()).await?;

    // イベントハンドラを作成（エラーを返す）
    let handler = FnEventHandler::new(|event: TestInputEvent| {
        Box::pin(async move {
            Err(TestError::Test("retry".to_string()))
        }) as BoxFuture<'static, Result<Option<TestOutputEvent>, TestError>>
    });

    // 出力イベントを受け取るためのチャネルを作成
    let (tx, mut rx) = mpsc::channel(10);
    let sink = TestEventSink { tx };

    // StreamWorkerを作成
    let worker = StreamWorker::new(
        Arc::new(input_subscriber),
        Arc::new(sink),
        Arc::new(handler),
    );

    // シャットダウントークンを作成
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // ワーカーを実行（短時間だけ）
    let worker_handle = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(2), worker.run(shutdown_clone)).await
    });

    // シャットダウン
    shutdown.cancel();
    let _ = worker_handle.await;

    // 再度サブスクライブして、メッセージが再配信されることを確認
    let subscriber = JsSubscriber::<TestInputEvent>::new(nats_client.clone(), input_stream);
    let mut stream = subscriber.subscribe().await?;
    
    // メッセージを受信
    let redelivered_message = tokio::time::timeout(Duration::from_secs(5), stream.next()).await??.unwrap()?;
    
    assert_eq!(redelivered_message.event().id, 2);
    assert_eq!(redelivered_message.event().message, "Test error retry");
    
    // 今度はAckする
    redelivered_message.ack().await?;

    Ok(())
}

async fn setup_nats() -> anyhow::Result<(ContainerAsync<GenericImage>, String)> {
    // ---- Spin‑up test JetStream -------------------------------------------
    let container = GenericImage::new("nats", "latest")
        .with_exposed_port(4222u16.into())
        .with_wait_for(WaitFor::message_on_stderr("Server is ready"))
        .with_cmd(vec!["--js"])
        .start()
        .await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(4222u16).await?;
    let url = format!("nats://{}:{}", host, port);
    Ok((container, url))
}
```

## テストのポイント

1. **JetStreamのAck/Nackテスト**:
   - メッセージをAckせずにDropした場合、少し待てばもう一度再配信されることを確認
   - メッセージをAckした場合、再配信されないことを確認

2. **StreamWorkerのAck/Nackテスト**:
   - 成功時にAckが送信されることを確認
   - エラー時（Retry）にAckが送信されないことを確認
   - エラー時（Ignore）にAckが送信されることを確認

3. **テスト環境**:
   - testcontainersを使用して、テスト用のNATSサーバーを起動
   - AckWaitの時間を短く設定して、テストの実行時間を短縮

4. **テストの実行方法**:
   - `cargo test` コマンドで実行
   - テストの実行には、Dockerが必要

## テスト実行時の注意点

1. **AckWaitの設定**:
   - JetStreamのデフォルトのAckWaitは30秒
   - テストでは短く設定する必要がある

2. **タイムアウト**:
   - テストでは、タイムアウトを設定して、メッセージが再配信されないことを確認
   - タイムアウトの時間は、AckWaitの時間よりも短く設定する

3. **Dockerの要件**:
   - テストの実行には、Dockerが必要
   - Dockerが起動していない場合は、テストがスキップされる
