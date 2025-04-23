use anyhow::{anyhow, Result};
use async_trait::async_trait;
use domain::event::Event; // domain::event::Event を使用
use domain::ports::event_source::EventSource; // domain::ports::event_source を使用
use futures::future::BoxFuture;
use futures::stream::{self, BoxStream};
use kurec_app::worker::builder::{FnHandler, Handler, Middleware, Next, WorkerBuilder}; // kurec_app::worker::builder を使用
use shared_core::error_handling::{ClassifyError, ErrorAction}; // shared_core からインポート
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// テスト用のイベント型
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TestEvent {
    pub id: usize,
    pub data: String,
}

// Event トレイトを実装（stream_name と event_name メソッドは削除）
impl Event for TestEvent {}

// テスト用のトレイト
pub trait TestEventTrait: Send + Sync + 'static {}
impl TestEventTrait for TestEvent {}

// テスト用のエラー型
#[derive(Debug)]
pub struct TestError {
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
    events: Vec<TestEvent>,
    ack_called: Arc<AtomicBool>,
}

impl TestSubscriber {
    fn new(events: Vec<TestEvent>, ack_called: Arc<AtomicBool>) -> Self {
        Self { events, ack_called }
    }
}

#[async_trait]
impl EventSource<TestEvent, anyhow::Error> for TestSubscriber {
    async fn subscribe(&self) -> Result<BoxStream<'static, Result<TestEvent, anyhow::Error>>> {
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

// テスト用のミドルウェア
pub struct TestMiddleware {
    pub counter: Arc<AtomicUsize>,
}

#[async_trait]
impl Middleware<TestEvent, ()> for TestMiddleware {
    async fn handle(&self, event: TestEvent, ctx: (), next: Next<'_, TestEvent, ()>) -> Result<()> {
        // イベント処理前にカウンターをインクリメント
        self.counter.fetch_add(1, Ordering::SeqCst);

        // 次のミドルウェアまたはハンドラを呼び出す
        let result = next.run(event, ctx).await;

        // イベント処理後にカウンターをインクリメント
        self.counter.fetch_add(1, Ordering::SeqCst);

        result
    }
}

// テスト用のハンドラ
pub struct TestHandler {
    pub processed: Arc<AtomicUsize>,
    pub should_fail: bool,
    pub should_retry: bool,
}

#[async_trait]
impl Handler<TestEvent, ()> for TestHandler {
    async fn handle(&self, event: TestEvent, _ctx: ()) -> Result<()> {
        self.processed.fetch_add(1, Ordering::SeqCst);

        if self.should_fail {
            Err(anyhow!(TestError {
                message: format!("Failed to process event {}", event.id),
                should_retry: self.should_retry,
            }))
        } else {
            Ok(())
        }
    }
}

impl Clone for TestHandler {
    fn clone(&self) -> Self {
        Self {
            processed: self.processed.clone(),
            should_fail: self.should_fail,
            should_retry: self.should_retry,
        }
    }
}

#[tokio::test]
async fn test_middleware_success() -> Result<()> {
    // ミドルウェアのカウンター
    let middleware_counter = Arc::new(AtomicUsize::new(0));

    // ミドルウェアを作成
    let middleware = TestMiddleware {
        counter: middleware_counter.clone(),
    };

    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));
    let processed_clone = processed.clone();

    // ハンドラ関数
    let handler_fn = move |_: TestEvent, _: ()| -> BoxFuture<'static, Result<()>> {
        let counter = processed_clone.clone();
        Box::pin(async move {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    };

    // Next構造体を作成
    let handler = Arc::new(handler_fn);
    let next = Next::new(handler);

    // イベントを処理
    let event = TestEvent {
        id: 1,
        data: "test1".to_string(),
    };

    // ミドルウェアを実行
    middleware.handle(event, (), next).await?;

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 1);

    // ミドルウェアが前後で呼ばれたことを確認（前後2回）
    assert_eq!(middleware_counter.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn test_handler_success() -> Result<()> {
    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // ハンドラを作成
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: false,
        should_retry: false,
    };

    // イベントを処理
    let event = TestEvent {
        id: 1,
        data: "test1".to_string(),
    };

    // ハンドラを実行
    handler.handle(event, ()).await?;

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 1);

    Ok(())
}

#[tokio::test]
async fn test_handler_error_ignore() -> Result<()> {
    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // エラーを返すハンドラを作成（リトライしない）
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: true,
        should_retry: false, // Ignoreアクションを返す
    };

    // イベントを処理
    let event = TestEvent {
        id: 1,
        data: "test1".to_string(),
    };

    // ハンドラを実行
    let result = handler.handle(event, ()).await;

    // エラーが返されることを確認
    assert!(result.is_err());

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 1);

    // エラーがIgnoreアクションを返すことを確認
    if let Err(e) = result {
        if let Some(test_error) = e.downcast_ref::<TestError>() {
            assert_eq!(test_error.error_action(), ErrorAction::Ignore);
        } else {
            panic!("Expected TestError");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_handler_error_retry() -> Result<()> {
    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));

    // エラーを返すハンドラを作成（リトライする）
    let handler = TestHandler {
        processed: processed.clone(),
        should_fail: true,
        should_retry: true, // Retryアクションを返す
    };

    // イベントを処理
    let event = TestEvent {
        id: 1,
        data: "test1".to_string(),
    };

    // ハンドラを実行
    let result = handler.handle(event, ()).await;

    // エラーが返されることを確認
    assert!(result.is_err());

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 1);

    // エラーがRetryアクションを返すことを確認
    if let Err(e) = result {
        if let Some(test_error) = e.downcast_ref::<TestError>() {
            assert_eq!(test_error.error_action(), ErrorAction::Retry);
        } else {
            panic!("Expected TestError");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_fn_handler() -> Result<()> {
    // 処理されたイベントをカウント
    let processed = Arc::new(AtomicUsize::new(0));
    let processed_clone = processed.clone();

    // 関数ハンドラを作成
    let handler = FnHandler::new(
        move |event: TestEvent, _: ()| -> BoxFuture<'static, Result<()>> {
            let counter = processed_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                println!("Processing event: {}", event.id);
                Ok(())
            })
        },
    );

    // イベントを処理
    let event = TestEvent {
        id: 1,
        data: "test".to_string(),
    };
    handler.handle(event, ()).await?;

    // 処理されたイベント数を確認
    assert_eq!(processed.load(Ordering::SeqCst), 1);

    Ok(())
}
