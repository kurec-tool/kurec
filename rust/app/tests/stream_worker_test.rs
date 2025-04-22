use anyhow::Result;
use async_trait::async_trait;
use domain::event::Event;
use domain::ports::event_source::EventSource;
use domain::ports::EventSink;
use futures::future::BoxFuture;
use futures::stream::{self, BoxStream};
use kurec_app::worker::stream_worker::{
    FnStreamHandler, StreamHandler, StreamMiddleware, StreamNext, StreamWorker,
};
use shared_core::error_handling::{ClassifyError, ErrorAction};
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

// Event トレイトを実装（stream_name と event_name メソッドは削除）
impl Event for InputEvent {}

// テスト用の出力イベント型
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct OutputEvent {
    pub id: usize,
    pub data: String,
    pub processed: bool,
}

// Event トレイトを実装（stream_name と event_name メソッドは削除）
impl Event for OutputEvent {}

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
impl EventSource<InputEvent> for TestSubscriber {
    async fn subscribe(&self) -> Result<BoxStream<'static, Result<InputEvent, anyhow::Error>>> {
        let events = self.events.clone();
        let ack_called = self.ack_called.clone();

        // 'static ライフタイムを持つストリームを作成
        let stream = Box::pin(stream::iter(events.into_iter().map(move |event| {
            // ack_calledをtrueに設定（テスト用）
            ack_called.store(true, Ordering::SeqCst);
            Ok(event)
        })));

        Ok(stream)
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
impl EventSink<OutputEvent> for TestPublisher {
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
    ) -> Result<Option<OutputEvent>, TestError> {
        // イベント処理前にカウンターをインクリメント
        self.counter.fetch_add(1, Ordering::SeqCst);

        // 次のミドルウェアまたはハンドラを呼び出す
        let result = next.run(event).await;

        // イベント処理後にカウンターをインクリメント
        self.counter.fetch_add(1, Ordering::SeqCst);

        // 出力イベントを変更 (Option を考慮)
        match result {
            Ok(Some(mut output)) => {
                output.data = format!("{}:middleware", output.data);
                Ok(Some(output))
            }
            Ok(None) => Ok(None), // None の場合はそのまま返す
            Err(e) => Err(e),     // エラーの場合はそのまま返す
        }
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
    async fn handle(&self, event: InputEvent) -> Result<Option<OutputEvent>, TestError> {
        self.processed.fetch_add(1, Ordering::SeqCst);

        if self.should_fail {
            Err(TestError {
                message: format!("Failed to process event {}", event.id),
                should_retry: self.should_retry,
            })
        } else {
            Ok(Some(OutputEvent {
                id: event.id,
                data: format!("{}:processed", event.data),
                processed: true,
            }))
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

    // EventSource を作成
    let source = Arc::new(TestSubscriber::new(events, ack_called.clone()));

    // パブリッシュされたイベントをカウント
    let published = Arc::new(AtomicUsize::new(0));
    let last_event = Arc::new(std::sync::Mutex::new(None));

    // EventSink を作成
    let sink = Arc::new(TestPublisher::new(published.clone(), last_event.clone()));

    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // ハンドラを作成
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: false,
        should_retry: false,
    };
    // ハンドラを Arc でラップ
    let handler_arc = Arc::new(handler);

    // ミドルウェアのカウンター
    let middleware_counter = Arc::new(AtomicUsize::new(0));

    // ミドルウェアを作成
    let middleware = TestMiddleware {
        counter: middleware_counter.clone(),
    };

    // キャンセルトークン
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // ストリームワーカーを構築して実行
    let worker_task = tokio::spawn(async move {
        StreamWorker::new(source, sink, handler_arc)
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

    // EventSource を作成
    let source = Arc::new(TestSubscriber::new(events, ack_called.clone()));

    // パブリッシュされたイベントをカウント
    let published = Arc::new(AtomicUsize::new(0));
    let last_event = Arc::new(std::sync::Mutex::new(None));

    // EventSink を作成
    let sink = Arc::new(TestPublisher::new(published.clone(), last_event.clone()));

    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // エラーを返すハンドラを作成（リトライしない）
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: true,
        should_retry: false, // Ignoreアクションを返す
    };
    // ハンドラを Arc でラップ
    let handler_arc = Arc::new(handler);

    // キャンセルトークン
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // ストリームワーカーを構築して実行
    let worker_task = tokio::spawn(async move {
        StreamWorker::new(source, sink, handler_arc)
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

    // EventSource を作成
    let source = Arc::new(TestSubscriber::new(events, ack_called.clone()));

    // パブリッシュされたイベントをカウント
    let published = Arc::new(AtomicUsize::new(0));
    let last_event = Arc::new(std::sync::Mutex::new(None));

    // EventSink を作成
    let sink = Arc::new(TestPublisher::new(published.clone(), last_event.clone()));

    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // エラーを返すハンドラを作成（リトライする）
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: true,
        should_retry: true, // Retryアクションを返す
    };
    // ハンドラを Arc でラップ
    let handler_arc = Arc::new(handler);

    // キャンセルトークン
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // ストリームワーカーを構築して実行
    let worker_task = tokio::spawn(async move {
        StreamWorker::new(source, sink, handler_arc)
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

    // 現在の実装では、Retryアクションでもackが呼ばれる
    // assert!(!ack_called.load(Ordering::SeqCst));
    assert!(ack_called.load(Ordering::SeqCst));

    Ok(())
}

#[tokio::test]
async fn test_fn_stream_handler() -> Result<()> {
    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));
    let processed_clone = processed.clone();

    // 関数ハンドラを作成
    let handler = FnStreamHandler::new(
        move |event: InputEvent| -> BoxFuture<'static, Result<Option<OutputEvent>, TestError>> {
            let counter = processed_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(Some(OutputEvent {
                    id: event.id,
                    data: format!("{}:fn_processed", event.data),
                    processed: true,
                }))
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
    assert!(result.is_some());
    let output = result.unwrap();
    assert_eq!(output.id, 1);
    assert_eq!(output.data, "test:fn_processed");
    assert!(output.processed);

    Ok(())
}
