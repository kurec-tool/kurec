use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use shared_core::event_metadata::{Event, HasStreamDef};
use shared_core::event_publisher::EventPublisher;
use shared_core::event_subscriber::{AckHandle, EventSubscriber};

// テスト用のイベント型1
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent1 {
    pub id: usize,
    pub message: String,
}

impl HasStreamDef for TestEvent1 {
    fn stream_name() -> &'static str {
        "test-event1-stream"
    }

    fn stream_subject() -> &'static str {
        "test.event1.subject"
    }
}

impl Event for TestEvent1 {}

// テスト用のイベント型2（異なるストリームとサブジェクト）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent2 {
    pub id: usize,
    pub data: Vec<u8>,
}

impl HasStreamDef for TestEvent2 {
    fn stream_name() -> &'static str {
        "test-event2-stream"
    }

    fn stream_subject() -> &'static str {
        "test.event2.subject"
    }
}

impl Event for TestEvent2 {}

// モック用のJetStreamコンテキスト
#[derive(Clone)]
struct MockJetStreamCtx {
    published_events: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
}

impl MockJetStreamCtx {
    fn new() -> Self {
        Self {
            published_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_published_events(&self) -> Vec<(String, Vec<u8>)> {
        self.published_events.lock().unwrap().clone()
    }
}

// モック用のパブリッシャー
struct MockPublisher<E: Event> {
    ctx: MockJetStreamCtx,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: Event> MockPublisher<E> {
    fn new(ctx: MockJetStreamCtx) -> Self {
        Self {
            ctx,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<E: Event> EventPublisher<E> for MockPublisher<E> {
    async fn publish(&self, event: E) -> Result<()> {
        let payload = serde_json::to_vec(&event)?;
        self.ctx
            .published_events
            .lock()
            .unwrap()
            .push((E::stream_subject().to_string(), payload));
        Ok(())
    }
}

// モック用のサブスクライバー
struct MockSubscriber<E: Event + Clone> {
    events: Vec<E>,
}

impl<E: Event + Clone> MockSubscriber<E> {
    fn new(events: Vec<E>) -> Self {
        Self { events }
    }
}

#[async_trait]
impl<E: Event + Clone> EventSubscriber<E> for MockSubscriber<E> {
    async fn subscribe(&self) -> Result<BoxStream<'static, (E, AckHandle)>> {
        let events = self.events.clone();
        let stream = stream::iter(events.into_iter().map(move |event| {
            let ack_handle = AckHandle::new(Box::new(move || Box::pin(async move { Ok(()) })));
            (event, ack_handle)
        }));

        Ok(Box::pin(stream))
    }
}

#[tokio::test]
async fn test_publisher_with_mock() -> Result<()> {
    // モックコンテキストを作成
    let ctx = MockJetStreamCtx::new();

    // パブリッシャーを作成
    let publisher = MockPublisher::<TestEvent1>::new(ctx.clone());

    // テストイベントを作成
    let test_event = TestEvent1 {
        id: 1,
        message: "Test message".to_string(),
    };

    // イベントを発行
    publisher.publish(test_event.clone()).await?;

    // 発行されたイベントを確認
    let published = ctx.get_published_events();
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].0, TestEvent1::stream_subject());

    // 発行されたペイロードをデシリアライズして確認
    let deserialized: TestEvent1 = serde_json::from_slice(&published[0].1)?;
    assert_eq!(deserialized, test_event);

    Ok(())
}

#[tokio::test]
async fn test_subscriber_with_mock() -> Result<()> {
    // テストイベントを作成
    let test_events = vec![
        TestEvent1 {
            id: 1,
            message: "Test message 1".to_string(),
        },
        TestEvent1 {
            id: 2,
            message: "Test message 2".to_string(),
        },
    ];

    // サブスクライバーを作成
    let subscriber = MockSubscriber::new(test_events.clone());

    // サブスクライブしてイベントを受信
    let mut stream = subscriber.subscribe().await?;

    // 最初のイベントを受信
    if let Some((event, ack)) = stream.next().await {
        assert_eq!(event, test_events[0]);
        ack.ack().await?;
    } else {
        panic!("Expected event not received");
    }

    // 2番目のイベントを受信
    if let Some((event, ack)) = stream.next().await {
        assert_eq!(event, test_events[1]);
        ack.ack().await?;
    } else {
        panic!("Expected event not received");
    }

    // これ以上イベントがないことを確認
    assert!(stream.next().await.is_none());

    Ok(())
}

#[tokio::test]
async fn test_different_event_types() -> Result<()> {
    // モックコンテキストを作成
    let ctx = MockJetStreamCtx::new();

    // 異なるイベント型のパブリッシャーを作成
    let publisher1 = MockPublisher::<TestEvent1>::new(ctx.clone());
    let publisher2 = MockPublisher::<TestEvent2>::new(ctx.clone());

    // テストイベントを作成
    let test_event1 = TestEvent1 {
        id: 1,
        message: "Test message".to_string(),
    };

    let test_event2 = TestEvent2 {
        id: 2,
        data: vec![1, 2, 3, 4],
    };

    // イベントを発行
    publisher1.publish(test_event1.clone()).await?;
    publisher2.publish(test_event2.clone()).await?;

    // 発行されたイベントを確認
    let published = ctx.get_published_events();
    assert_eq!(published.len(), 2);

    // 異なるサブジェクトに発行されていることを確認
    assert_eq!(published[0].0, TestEvent1::stream_subject());
    assert_eq!(published[1].0, TestEvent2::stream_subject());

    // 発行されたペイロードをデシリアライズして確認
    let deserialized1: TestEvent1 = serde_json::from_slice(&published[0].1)?;
    assert_eq!(deserialized1, test_event1);

    let deserialized2: TestEvent2 = serde_json::from_slice(&published[1].1)?;
    assert_eq!(deserialized2, test_event2);

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    // エラーを返すモックパブリッシャー
    struct ErrorPublisher<E: Event> {
        _phantom: std::marker::PhantomData<E>,
    }

    #[async_trait]
    impl<E: Event> EventPublisher<E> for ErrorPublisher<E> {
        async fn publish(&self, _event: E) -> Result<()> {
            Err(anyhow::anyhow!("Simulated publish error"))
        }
    }

    // エラーを返すモックサブスクライバー
    struct ErrorSubscriber<E: Event> {
        _phantom: std::marker::PhantomData<E>,
    }

    #[async_trait]
    impl<E: Event> EventSubscriber<E> for ErrorSubscriber<E> {
        async fn subscribe(&self) -> Result<BoxStream<'static, (E, AckHandle)>> {
            Err(anyhow::anyhow!("Simulated subscribe error"))
        }
    }

    // パブリッシャーとサブスクライバーを作成
    let publisher = ErrorPublisher::<TestEvent1> {
        _phantom: std::marker::PhantomData,
    };

    let subscriber = ErrorSubscriber::<TestEvent1> {
        _phantom: std::marker::PhantomData,
    };

    // テストイベントを作成
    let test_event = TestEvent1 {
        id: 1,
        message: "Test message".to_string(),
    };

    // 発行時にエラーが発生することを確認
    let publish_result = publisher.publish(test_event).await;
    assert!(publish_result.is_err());

    // 購読時にエラーが発生することを確認
    let subscribe_result = subscriber.subscribe().await;
    assert!(subscribe_result.is_err());

    Ok(())
}
