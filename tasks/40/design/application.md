# Issue #40: Ack/Nack機能の修正設計 - アプリケーション層

アプリケーション層では、EventHandlerトレイトとStreamWorkerの更新を行います。

## EventHandler

```rust
// app/src/handler/event_handler.rs
use anyhow::Result;
use async_trait::async_trait;
use domain::event::Event;

/// イベントハンドラーのトレイト
#[async_trait]
pub trait EventHandler<I, O, E>: Send + Sync + 'static
where
    I: Event,
    O: Event,
    E: shared_core::error_handling::ClassifyError + Send + Sync + 'static,
{
    /// イベントを処理する
    async fn handle(&self, event: I) -> Result<Option<O>, E>;
}

/// 関数をハンドラとして扱うためのラッパー
pub struct FnEventHandler<I, O, E, F>
where
    I: Event,
    O: Event,
    E: shared_core::error_handling::ClassifyError + Send + Sync + 'static,
    F: Fn(I) -> futures::future::BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static,
{
    f: F,
    _phantom: std::marker::PhantomData<(I, O, E)>,
}

impl<I, O, E, F> FnEventHandler<I, O, E, F>
where
    I: Event,
    O: Event,
    E: shared_core::error_handling::ClassifyError + Send + Sync + 'static,
    F: Fn(I) -> futures::future::BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<I, O, E, F> EventHandler<I, O, E> for FnEventHandler<I, O, E, F>
where
    I: Event,
    O: Event,
    E: shared_core::error_handling::ClassifyError + Send + Sync + 'static,
    F: Fn(I) -> futures::future::BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static,
{
    async fn handle(&self, event: I) -> Result<Option<O>, E> {
        (self.f)(event).await
    }
}
```

## StreamWorker

```rust
// app/src/worker/stream_worker.rs
use anyhow::Result;
use async_trait::async_trait;
use domain::event::Event;
use domain::ports::event_sink::EventSink;
use futures::future::BoxFuture;
use futures::StreamExt;
use infra_common::ackable_event::AckableEvent;
use infra_common::event_source::EventSource;
use shared_core::error_handling::ClassifyError;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::handler::event_handler::EventHandler;

/// ストリームワーカーのミドルウェアトレイト
#[async_trait]
pub trait StreamMiddleware<I, O, E>: Send + Sync + 'static
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    async fn handle(&self, event: I, next: StreamNext<'_, I, O, E>) -> Result<Option<O>, E>;
}

/// ミドルウェアチェーンの次の処理を表す構造体
pub struct StreamNext<'a, I, O, E>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    pub(crate) handler:
        Arc<dyn Fn(I) -> BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static>,
    _phantom: PhantomData<&'a ()>,
}

impl<I, O, E> StreamNext<'_, I, O, E>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    pub fn new(
        handler: Arc<dyn Fn(I) -> BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }

    pub async fn run(&self, event: I) -> Result<Option<O>, E> {
        (self.handler)(event).await
    }
}

impl<I, O, E> Clone for StreamNext<'_, I, O, E>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            _phantom: PhantomData,
        }
    }
}

/// ストリームワーカー
pub struct StreamWorker<I, O, IErr, E>
where
    I: Event,
    O: Event,
    IErr: Send + Sync + 'static,
    E: ClassifyError + Send + Sync + 'static,
{
    source: Arc<dyn EventSource<I, IErr>>,
    sink: Arc<dyn EventSink<O>>,
    handler: Arc<dyn EventHandler<I, O, E>>,
    middlewares: Vec<Arc<dyn StreamMiddleware<I, O, E>>>,
    durable_name: Option<String>,
}

impl<I, O, IErr, E> StreamWorker<I, O, IErr, E>
where
    I: Event,
    O: Event,
    IErr: Send + Sync + 'static,
    E: ClassifyError + Send + Sync + 'static,
{
    /// 新しいStreamWorkerを作成
    pub fn new(
        source: Arc<dyn EventSource<I, IErr>>,
        sink: Arc<dyn EventSink<O>>,
        handler: Arc<dyn EventHandler<I, O, E>>,
    ) -> Self {
        Self {
            source,
            sink,
            handler,
            middlewares: Vec::new(),
            durable_name: None,
        }
    }

    /// ミドルウェアを追加
    pub fn with_middleware<M>(mut self, middleware: M) -> Self
    where
        M: StreamMiddleware<I, O, E> + 'static,
    {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    /// durable名を設定
    pub fn durable(mut self, name: &str) -> Self {
        self.durable_name = Some(name.to_string());
        self
    }

    /// durable名を自動生成
    pub fn durable_auto(mut self) -> Self {
        let type_name = std::any::type_name::<I>();
        let durable_name = format!("consumer_{}", type_name.replace("::", "_").to_lowercase());
        self.durable_name = Some(durable_name);
        self
    }

    /// ミドルウェアチェーンを実行
    async fn execute_middleware_chain(
        handler: Arc<dyn EventHandler<I, O, E>>,
        middlewares: &[Arc<dyn StreamMiddleware<I, O, E>>],
        event: I,
    ) -> Result<Option<O>, E> {
        // ミドルウェアがない場合は直接ハンドラを実行
        if middlewares.is_empty() {
            return handler.handle(event).await;
        }

        // ミドルウェアチェーンを構築
        let mut chain = Vec::with_capacity(middlewares.len());
        for (i, middleware) in middlewares.iter().enumerate() {
            let next_handler: Arc<dyn Fn(I) -> BoxFuture<'static, Result<Option<O>, E>> + Send + Sync> =
                if i == middlewares.len() - 1 {
                    // 最後のミドルウェアの次はハンドラ
                    Arc::new(move |e| {
                        let handler = handler.clone();
                        Box::pin(async move { handler.handle(e).await })
                    })
                } else {
                    // 次のミドルウェア
                    let next_middleware = middlewares[i + 1].clone();
                    let next_chain = chain.clone();
                    Arc::new(move |e| {
                        let next_middleware = next_middleware.clone();
                        let next = StreamNext::new(next_chain[i + 1].clone());
                        Box::pin(async move { next_middleware.handle(e, next).await })
                    })
                };
            chain.push(next_handler);
        }

        // 最初のミドルウェアから実行
        let first_middleware = middlewares[0].clone();
        let next = StreamNext::new(chain[0].clone());
        first_middleware.handle(event, next).await
    }

    /// ワーカーを実行
    pub async fn run(self, shutdown: CancellationToken) -> Result<()> {
        // sourceからメッセージストリームを取得
        let mut stream = self.source.subscribe().await?;
        let shutdown_token = shutdown.clone();

        // ハンドラとミドルウェアをArcでラップ
        let handler = self.handler;
        let middlewares: Vec<Arc<dyn StreamMiddleware<I, O, E>>> = self.middlewares;
        let sink = self.sink;

        // メッセージ処理ループ
        loop {
            select! {
                // シャットダウントークンが発火したら終了
                _ = shutdown_token.cancelled() => {
                    break;
                }
                // メッセージを受信したら処理
                message = stream.next() => {
                    match message {
                        Some(Ok(mut ackable_event)) => {
                            // 通常の処理
                            let event = ackable_event.event().clone(); // イベントをクローン
                            let result = Self::execute_middleware_chain(
                                handler.clone(),
                                &middlewares,
                                event
                            ).await;
                            
                            match result {
                                Ok(Some(output_event)) => {
                                    // 出力イベントをSinkに発行
                                    if let Err(e) = sink.publish(output_event).await {
                                        error!(error = %e, "Failed to publish output event");
                                        // パブリッシュに失敗した場合はAckしない（再試行）
                                        continue;
                                    }
                                    
                                    // 処理完了後に明示的にAck
                                    if let Err(e) = ackable_event.ack().await {
                                        error!(error = %e, "Failed to ack message after successful processing");
                                    }
                                }
                                Ok(None) => {
                                    // 出力イベントがない場合も処理は完了したので明示的にAck
                                    if let Err(e) = ackable_event.ack().await {
                                        error!(error = %e, "Failed to ack message after processing with no output");
                                    }
                                }
                                Err(e) => {
                                    // エラーアクションに基づいて処理
                                    match e.error_action() {
                                        shared_core::error_handling::ErrorAction::Retry => {
                                            // 再試行の場合はAckしない
                                            warn!(error = %e, "Error processing event, will retry");
                                            continue;
                                        }
                                        shared_core::error_handling::ErrorAction::Ignore => {
                                            // エラーを無視する場合はAck
                                            warn!(error = %e, "Error processing event, ignoring");
                                            if let Err(ack_err) = ackable_event.ack().await {
                                                error!(error = %ack_err, "Failed to ack message after ignored error");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(error)) => {
                            // エラーの処理
                            // ログは既にソース側で記録済み
                            
                            // 必要に応じて追加の処理
                            // metrics::increment_counter!("event_errors", "type" => error.type_str());
                        }
                        None => {
                            // ストリームが終了
                            info!("Event stream ended");
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

## 設計のポイント

1. **StreamWorker**:
   - AckableEventを使用して、イベントとAck機能を扱う
   - イベント処理が成功した場合は明示的にAckを送信
   - エラーアクションに基づいて、Ackするかどうかを決定
     - Retry: Ackしない（再試行）
     - Ignore: Ackする（エラーを無視）

2. **処理フロー**:
   1. イベントソースからAckableEventを取得
   2. イベントをハンドラで処理
   3. 処理結果に基づいて、Ackするかどうかを決定
      - 成功した場合は、Sinkへの発行が完了した後にAck
      - エラーの場合は、エラーアクションに基づいてAckするかどうかを決定

3. **エラー処理**:
   - エラーアクションに基づいて、Ackするかどうかを決定
   - Retry: Ackしない（再試行）
   - Ignore: Ackする（エラーを無視）

4. **シャットダウン処理**:
   - シャットダウントークンが発火したら、ループを終了
   - 処理中のイベントは完了まで待機

## 変更点

以前の実装と比較して、以下の変更点があります：

1. **明示的なAck**:
   - 以前の実装では、自動的にすべてのメッセージにAckしていた
   - 新しい実装では、処理が完了した後に明示的にAckを送信

2. **エラー処理の改善**:
   - エラーアクションに基づいて、Ackするかどうかを決定
   - エラーの詳細をログに記録

3. **処理フローの改善**:
   - Sinkへの発行が完了した後にAckを送信
   - エラーの場合は、エラーアクションに基づいてAckするかどうかを決定
