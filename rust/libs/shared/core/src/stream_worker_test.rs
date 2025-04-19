use crate::error_handling::{ClassifyError, ErrorAction};
use crate::event_metadata::Event;
use crate::event_publisher::EventPublisher;
use crate::event_subscriber::{AckHandle, EventSubscriber};
use crate::stream_worker::{
    FnStreamHandler, StreamHandler, StreamMiddleware, StreamNext, StreamWorker,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::stream::{self, BoxStream};
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// テスト用の入力イベント型
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct InputEvent {
    pub id: usize,
    pub data: String,
}

impl Event for InputEvent {
    fn stream_name() -> &'static str {
        "test_input_stream"
    }

    fn event_name() -> &'static str {
        "input_event"
    }
}

// テスト用の出力イベント型
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct OutputEvent {
    pub id: usize,
    pub data: String,
    pub processed: bool,
}

impl Event for OutputEvent {
    fn stream_name() -> &'static str {
        "test_output_stream"
    }

    fn event_name() -> &'static str {
        "output_event"
    }
}

// テスト用のエラー型
#[derive(Debug)]
struct TestError {
    pub message: String,
    pub should_retry: bool,
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError: {}", self.message)
    }
}

impl std::error::Error for TestError {}

impl ClassifyError for TestError {
    fn error_action(&self) -> ErrorAction {
        if self.should_retry {
            ErrorAction::Retry
        } else {
            ErrorAction::Ignore
        }
    }
}

// テスト用のサブスクライバー
struct TestSubscriber {
    events: Vec<InputEvent>,
    ack_called: Arc<AtomicBool>,
}

impl TestSubscriber {
    fn new(events: Vec<InputEvent>, ack_called: Arc<AtomicBool>) -> Self {
        Self { events, ack_called }
    }
}

#[async_trait]
impl EventSubscriber<InputEvent> for TestSubscriber {
    async fn subscribe(&self) -> Result<BoxStream<'static, (InputEvent, AckHandle)>> {
        let events = self.events.clone();
        let ack_called = self.ack_called.clone();

        let stream = stream::iter(events.into_iter().map(move |event| {
            let ack_called = ack_called.clone();
            let ack_handle = AckHandle::new(Box::new(move || {
                let ack_called = ack_called.clone();
                Box::pin(async move {
                    ack_called.store(true, Ordering::SeqCst);
                    Ok(())
                })
            }));
            (event, ack_handle)
        }));

        Ok(Box::pin(stream))
    }
}

// テスト用のパブリッシャー
struct TestPublisher {
    published: Arc<AtomicUsize>,
    last_event: Arc<std::sync::Mutex<Option<OutputEvent>>>,
}

impl TestPublisher {
    fn new(
        published: Arc<AtomicUsize>,
        last_event: Arc<std::sync::Mutex<Option<OutputEvent>>>,
    ) -> Self {
        Self {
            published,
            last_event,
        }
    }
}

#[async_trait]
impl EventPublisher<OutputEvent> for TestPublisher {
    async fn publish(&self, event: OutputEvent) -> Result<()> {
        self.published.fetch_add(1, Ordering::SeqCst);
        let mut last_event = self.last_event.lock().unwrap();
        *last_event = Some(event);
        Ok(())
    }
}

// テスト用のミドルウェア
struct TestMiddleware {
    counter: Arc<AtomicUsize>,
}

#[async_trait]
impl StreamMiddleware<InputEvent, OutputEvent, TestError> for TestMiddleware {
    async fn handle(
        &self,
        event: InputEvent,
        next: StreamNext<'_, InputEvent, OutputEvent, TestError>,
    ) -> Result<OutputEvent, TestError> {
        // イベント処理前にカウンターをインクリメント
        self.counter.fetch_add(1, Ordering::SeqCst);

        // 次のミドルウェアまたはハンドラを呼び出す
        let mut result = next.run(event).await;

        // イベント処理後にカウンターをインクリメント
        self.counter.fetch_add(1, Ordering::SeqCst);

        // 出力イベントを変更
        if let Ok(ref mut output) = result {
            output.data = format!("{}:middleware", output.data);
        }

        result
    }
}

// テスト用のハンドラ
#[derive(Clone)]
struct TestHandler {
    processed: Arc<AtomicUsize>,
    should_fail: bool,
    should_retry: bool,
}

#[async_trait]
impl StreamHandler<InputEvent, OutputEvent, TestError> for TestHandler {
    async fn handle(&self, event: InputEvent) -> Result<OutputEvent, TestError> {
        self.processed.fetch_add(1, Ordering::SeqCst);

        if self.should_fail {
            Err(TestError {
                message: format!("Failed to process event {}", event.id),
                should_retry: self.should_retry,
            })
        } else {
            Ok(OutputEvent {
                id: event.id,
                data: format!("{}:processed", event.data),
                processed: true,
            })
        }
    }
}

#[tokio::test]
async fn test_stream_worker_success() -> Result<()> {
    // テスト用のイベント
    let events = vec![
        InputEvent {
            id: 1,
            data: "test1".to_string(),
        },
        InputEvent {
            id: 2,
            data: "test2".to_string(),
        },
    ];

    // ackが呼ばれたかを追跡
    let ack_called = Arc::new(AtomicBool::new(false));

    // サブスクライバーを作成
    let subscriber = Arc::new(TestSubscriber::new(events, ack_called.clone()));

    // パブリッシュされたイベントをカウント
    let published = Arc::new(AtomicUsize::new(0));
    let last_event = Arc::new(std::sync::Mutex::new(None));

    // パブリッシャーを作成
    let publisher = Arc::new(TestPublisher::new(published.clone(), last_event.clone()));

    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // ハンドラを作成
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: false,
        should_retry: false,
    };

    // ミドルウェアのカウンター
    let middleware_counter = Arc::new(AtomicUsize::new(0));

    // ミドルウェアを作成
    let middleware = TestMiddleware {
        counter: middleware_counter.clone(),
    };

    // ハンドラ関数をラップ
    let handler_fn =
        move |event: InputEvent| -> BoxFuture<'static, Result<OutputEvent, TestError>> {
            let handler_clone = handler.clone();
            Box::pin(async move { handler_clone.handle(event).await })
        };

    // キャンセルトークン
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // ストリームワーカーを構築して実行
    let worker_task = tokio::spawn(async move {
        StreamWorker::new(subscriber, publisher, handler_fn)
            .with_middleware(middleware)
            .durable("test_stream_worker")
            .run(token_clone)
            .await
    });

    // 少し待ってからキャンセル
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    token.cancel();

    // ワーカーの終了を待つ
    worker_task.await??;

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 2);

    // パブリッシュされたイベント数を確認
    assert_eq!(published.load(Ordering::SeqCst), 2);

    // ミドルウェアが各イベントの前後で呼ばれたことを確認（2イベント × 前後2回 = 4回）
    assert_eq!(middleware_counter.load(Ordering::SeqCst), 4);

    // ackが呼ばれたことを確認
    assert!(ack_called.load(Ordering::SeqCst));

    // 最後のイベントを確認
    let last = last_event.lock().unwrap();
    assert!(last.is_some());
    let last = last.as_ref().unwrap();
    assert_eq!(last.id, 2);
    assert_eq!(last.data, "test2:processed:middleware");
    assert!(last.processed);

    Ok(())
}

#[tokio::test]
async fn test_stream_worker_error_handling() -> Result<()> {
    // テスト用のイベント
    let events = vec![InputEvent {
        id: 1,
        data: "test1".to_string(),
    }];

    // ackが呼ばれたかを追跡
    let ack_called = Arc::new(AtomicBool::new(false));

    // サブスクライバーを作成
    let subscriber = Arc::new(TestSubscriber::new(events, ack_called.clone()));

    // パブリッシュされたイベントをカウント
    let published = Arc::new(AtomicUsize::new(0));
    let last_event = Arc::new(std::sync::Mutex::new(None));

    // パブリッシャーを作成
    let publisher = Arc::new(TestPublisher::new(published.clone(), last_event.clone()));

    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // エラーを返すハンドラを作成（リトライしない）
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: true,
        should_retry: false, // Ignoreアクションを返す
    };

    // ハンドラ関数をラップ
    let handler_fn =
        move |event: InputEvent| -> BoxFuture<'static, Result<OutputEvent, TestError>> {
            let handler_clone = handler.clone();
            Box::pin(async move { handler_clone.handle(event).await })
        };

    // キャンセルトークン
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // ストリームワーカーを構築して実行
    let worker_task = tokio::spawn(async move {
        StreamWorker::new(subscriber, publisher, handler_fn)
            .durable("test_stream_worker_error")
            .run(token_clone)
            .await
    });

    // 少し待ってからキャンセル
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    token.cancel();

    // ワーカーの終了を待つ
    worker_task.await??;

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 1);

    // パブリッシュされたイベント数を確認（エラーのため0）
    assert_eq!(published.load(Ordering::SeqCst), 0);

    // エラーはIgnoreアクションを返すので、ackが呼ばれるはず
    assert!(ack_called.load(Ordering::SeqCst));

    Ok(())
}

#[tokio::test]
async fn test_stream_worker_error_retry() -> Result<()> {
    // テスト用のイベント
    let events = vec![InputEvent {
        id: 1,
        data: "test1".to_string(),
    }];

    // ackが呼ばれたかを追跡
    let ack_called = Arc::new(AtomicBool::new(false));

    // サブスクライバーを作成
    let subscriber = Arc::new(TestSubscriber::new(events, ack_called.clone()));

    // パブリッシュされたイベントをカウント
    let published = Arc::new(AtomicUsize::new(0));
    let last_event = Arc::new(std::sync::Mutex::new(None));

    // パブリッシャーを作成
    let publisher = Arc::new(TestPublisher::new(published.clone(), last_event.clone()));

    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // エラーを返すハンドラを作成（リトライする）
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: true,
        should_retry: true, // Retryアクションを返す
    };

    // ハンドラ関数をラップ
    let handler_fn =
        move |event: InputEvent| -> BoxFuture<'static, Result<OutputEvent, TestError>> {
            let handler_clone = handler.clone();
            Box::pin(async move { handler_clone.handle(event).await })
        };

    // キャンセルトークン
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // ストリームワーカーを構築して実行
    let worker_task = tokio::spawn(async move {
        StreamWorker::new(subscriber, publisher, handler_fn)
            .durable("test_stream_worker_retry")
            .run(token_clone)
            .await
    });

    // 少し待ってからキャンセル
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    token.cancel();

    // ワーカーの終了を待つ
    worker_task.await??;

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 1);

    // パブリッシュされたイベント数を確認（エラーのため0）
    assert_eq!(published.load(Ordering::SeqCst), 0);

    // エラーはRetryアクションを返すので、ackは呼ばれないはず
    assert!(!ack_called.load(Ordering::SeqCst));

    Ok(())
}

#[tokio::test]
async fn test_fn_stream_handler() -> Result<()> {
    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));
    let processed_clone = processed.clone();

    // 関数ハンドラを作成
    let handler = FnStreamHandler::new(
        move |event: InputEvent| -> BoxFuture<'static, Result<OutputEvent, TestError>> {
            let counter = processed_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(OutputEvent {
                    id: event.id,
                    data: format!("{}:fn_processed", event.data),
                    processed: true,
                })
            })
        },
    );

    // イベントを処理
    let event = InputEvent {
        id: 1,
        data: "test".to_string(),
    };
    let result = handler.handle(event).await?;

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 1);

    // 結果を確認
    assert_eq!(result.id, 1);
    assert_eq!(result.data, "test:fn_processed");
    assert!(result.processed);

    Ok(())
}
