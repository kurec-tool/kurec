use crate::error_handling::ClassifyError;
use crate::event_metadata::Event;
use crate::event_publisher::EventPublisher;
use crate::event_subscriber::EventSubscriber;
use anyhow::Result;
use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::StreamExt;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;

/// ストリームワーカーのミドルウェアトレイト
/// 入力イベントを処理して出力イベントを生成する前後に処理を挟むことができる
#[async_trait]
pub trait StreamMiddleware<I, O, E>: Send + Sync + 'static
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    async fn handle(&self, event: I, next: StreamNext<'_, I, O, E>) -> Result<O, E>;
}

/// ミドルウェアチェーンの次の処理を表す構造体
pub struct StreamNext<'a, I, O, E>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    pub(crate) handler: Arc<dyn Fn(I) -> BoxFuture<'static, Result<O, E>> + Send + Sync + 'static>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, I, O, E> StreamNext<'a, I, O, E>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    pub fn new(
        handler: Arc<dyn Fn(I) -> BoxFuture<'static, Result<O, E>> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }

    pub async fn run(&self, event: I) -> Result<O, E> {
        (self.handler)(event).await
    }
}

impl<'a, I, O, E> Clone for StreamNext<'a, I, O, E>
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

/// イベントハンドラトレイト
#[async_trait]
pub trait StreamHandler<I, O, E>: Send + Sync + 'static
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    async fn handle(&self, event: I) -> Result<O, E>;
}

/// 関数をハンドラとして扱うためのラッパー
pub struct FnStreamHandler<I, O, E, F>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
    F: Fn(I) -> BoxFuture<'static, Result<O, E>> + Send + Sync + 'static,
{
    f: F,
    _phantom: PhantomData<(I, O, E)>,
}

impl<I, O, E, F> FnStreamHandler<I, O, E, F>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
    F: Fn(I) -> BoxFuture<'static, Result<O, E>> + Send + Sync + 'static,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<I, O, E, F> StreamHandler<I, O, E> for FnStreamHandler<I, O, E, F>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
    F: Fn(I) -> BoxFuture<'static, Result<O, E>> + Send + Sync + 'static,
{
    async fn handle(&self, event: I) -> Result<O, E> {
        (self.f)(event).await
    }
}

/// ストリームワーカー
/// 入力イベントを処理して出力イベントを生成するワーカー
pub struct StreamWorker<I, O, E, F>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
    F: Fn(I) -> BoxFuture<'static, Result<O, E>> + Send + Sync + 'static + Clone,
{
    subscriber: Arc<dyn EventSubscriber<I>>,
    publisher: Arc<dyn EventPublisher<O>>,
    handler: F,
    middlewares: Vec<Arc<dyn StreamMiddleware<I, O, E>>>,
    durable_name: Option<String>,
}

impl<I, O, E, F> StreamWorker<I, O, E, F>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
    F: Fn(I) -> BoxFuture<'static, Result<O, E>> + Send + Sync + 'static + Clone,
{
    /// 新しいStreamWorkerを作成
    pub fn new(
        subscriber: Arc<dyn EventSubscriber<I>>,
        publisher: Arc<dyn EventPublisher<O>>,
        handler: F,
    ) -> Self {
        Self {
            subscriber,
            publisher,
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
        let last_segment = type_name.split("::").last().unwrap_or(type_name);
        self.durable_name = Some(format!("worker_{}", last_segment));
        self
    }

    /// ミドルウェアチェーンを構築して実行
    async fn execute_middleware_chain(
        handler: F,
        middlewares: &[Arc<dyn StreamMiddleware<I, O, E>>],
        event: I,
    ) -> Result<O, E> {
        // ミドルウェアがない場合は直接ハンドラを実行
        if middlewares.is_empty() {
            return handler(event).await;
        }

        // ミドルウェアチェーンを構築
        let mut chain = Vec::new();
        for middleware in middlewares {
            chain.push(Arc::clone(middleware));
        }

        // 最後のミドルウェアの次の処理はハンドラ
        let handler_clone = handler.clone();
        let handler_fn = Arc::new(move |e: I| -> BoxFuture<'static, Result<O, E>> {
            let handler_inner = handler_clone.clone();
            Box::pin(async move { handler_inner(e).await })
        });

        // ミドルウェアチェーンを逆順に実行
        let mut next = StreamNext::new(handler_fn);
        for middleware in chain.into_iter().rev() {
            let prev_next = next.clone();
            let middleware_clone = Arc::clone(&middleware);
            let next_handler = Arc::new(move |e: I| -> BoxFuture<'static, Result<O, E>> {
                let pn = prev_next.clone();
                let middleware_inner = Arc::clone(&middleware_clone);
                Box::pin(async move { middleware_inner.handle(e, pn).await })
            });
            next = StreamNext::new(next_handler);
        }

        // 最初のミドルウェアを実行
        next.run(event).await
    }

    /// ワーカーを実行
    pub async fn run(self, shutdown: CancellationToken) -> Result<()> {
        // サブスクライバーからメッセージストリームを取得
        let mut stream = self.subscriber.subscribe().await?;
        let shutdown_token = shutdown.clone();

        // ハンドラとミドルウェアをArcでラップ
        let handler = self.handler.clone();
        let middlewares: Vec<Arc<dyn StreamMiddleware<I, O, E>>> = self.middlewares;
        let publisher = self.publisher;

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
                        Some((event, ack)) => {
                            // ミドルウェアチェーンを実行
                            let result = Self::execute_middleware_chain(
                                handler.clone(),
                                &middlewares,
                                event,
                            ).await;

                            match result {
                                Ok(output_event) => {
                                    // 出力イベントをパブリッシュ
                                    publisher.publish(output_event).await?;
                                    ack.ack().await?;
                                }
                                Err(e) => {
                                    // エラーアクションに基づいて処理
                                    match e.error_action() {
                                        crate::error_handling::ErrorAction::Retry => {
                                            // nack（再試行）
                                            // JetStreamの場合、ackしないと自動的に再配信される
                                        }
                                        crate::error_handling::ErrorAction::Ignore => {
                                            // エラーを無視してack
                                            ack.ack().await?;
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            // ストリームが終了したら終了
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
