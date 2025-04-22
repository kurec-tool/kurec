use shared_core::error_handling::{ClassifyError, ErrorAction}; // crate:: -> shared_core::
                                                               // use crate::event_metadata::Event; // 削除
use anyhow::Result;
use async_trait::async_trait;
use domain::event::Event;
use domain::ports::event_source::EventSource;
use futures::future::BoxFuture;
use futures::StreamExt;
use serde::de::DeserializeOwned; // DeserializeOwned をインポート
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;

/// ワーカーのミドルウェアトレイト
/// イベント処理の前後に処理を挟むことができる
#[async_trait]
pub trait Middleware<E, Ctx>: Send + Sync + 'static
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event -> DeserializeOwned
    Ctx: Clone + Send + Sync + 'static,
{
    async fn handle(&self, event: E, ctx: Ctx, next: Next<'_, E, Ctx>) -> Result<()>;
}

/// ミドルウェアチェーンの次の処理を表す構造体
pub struct Next<'a, E, Ctx>
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event 境界を追加
    Ctx: Clone + Send + Sync + 'static,
{
    pub(crate) handler:
        Arc<dyn Fn(E, Ctx) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static>,
    _phantom: PhantomData<&'a ()>,
}

impl<E, Ctx> Next<'_, E, Ctx>
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event 境界を追加
    Ctx: Clone + Send + Sync + 'static,
{
    pub fn new(
        handler: Arc<dyn Fn(E, Ctx) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }

    pub async fn run(&self, event: E, ctx: Ctx) -> Result<()> {
        (self.handler)(event, ctx).await
    }
}

/// イベントハンドラトレイト
#[async_trait]
pub trait Handler<E, Ctx>: Send + Sync + 'static
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event 境界を追加
    Ctx: Clone + Send + Sync + 'static,
{
    async fn handle(&self, event: E, ctx: Ctx) -> Result<()>;
}

/// 関数をハンドラとして扱うためのラッパー
pub struct FnHandler<E, Ctx, F>
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event 境界を追加
    Ctx: Clone + Send + Sync + 'static,
    F: Fn(E, Ctx) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
{
    f: F,
    _phantom: PhantomData<(E, Ctx)>,
}

impl<E, Ctx, F> FnHandler<E, Ctx, F>
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event 境界を追加
    Ctx: Clone + Send + Sync + 'static,
    F: Fn(E, Ctx) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<E, Ctx, F> Handler<E, Ctx> for FnHandler<E, Ctx, F>
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event 境界を追加
    Ctx: Clone + Send + Sync + 'static,
    F: Fn(E, Ctx) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
{
    async fn handle(&self, event: E, ctx: Ctx) -> Result<()> {
        (self.f)(event, ctx).await
    }
}

impl<E, Ctx> Clone for Next<'_, E, Ctx>
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event 境界を追加
    Ctx: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            _phantom: PhantomData,
        }
    }
}

/// ワーカービルダー
/// イベントの購読と処理を行うワーカーを構築する
pub struct WorkerBuilder<E, H, Ctx>
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event -> DeserializeOwned
    H: Handler<E, Ctx>,
    Ctx: Clone + Send + Sync + 'static,
{
    source: Arc<dyn EventSource<E>>, // subscriber -> source にリネーム
    handler: H,
    context: Ctx,
    middlewares: Vec<Arc<dyn Middleware<E, Ctx>>>,
    durable_name: Option<String>,
}

impl<E, H, Ctx> WorkerBuilder<E, H, Ctx>
where
    E: DeserializeOwned + Send + Sync + 'static + Event, // Event -> DeserializeOwned
    H: Handler<E, Ctx> + Clone,
    Ctx: Clone + Send + Sync + 'static,
{
    /// 新しいWorkerBuilderを作成
    pub fn new(source: Arc<dyn EventSource<E>>, handler: H, context: Ctx) -> Self {
        // subscriber -> source
        Self {
            source, // subscriber -> source
            handler,
            context,
            middlewares: Vec::new(),
            durable_name: None,
        }
    }

    /// ミドルウェアを追加
    pub fn with_middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware<E, Ctx> + 'static,
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
        let type_name = std::any::type_name::<E>();
        let last_segment = type_name.split("::").last().unwrap_or(type_name);
        self.durable_name = Some(format!("worker_{}", last_segment));
        self
    }

    /// ミドルウェアチェーンを構築して実行
    async fn execute_middleware_chain(
        handler: Arc<H>,
        middlewares: &[Arc<dyn Middleware<E, Ctx>>],
        event: E,
        context: Ctx,
    ) -> Result<()> {
        // ミドルウェアがない場合は直接ハンドラを実行
        if middlewares.is_empty() {
            return handler.handle(event, context).await;
        }

        // ミドルウェアチェーンを構築
        let mut chain = Vec::new();
        for middleware in middlewares {
            chain.push(Arc::clone(middleware));
        }

        // 最後のミドルウェアの次の処理はハンドラ
        let handler_clone = Arc::clone(&handler);
        let handler_fn = Arc::new(move |e: E, c: Ctx| -> BoxFuture<'static, Result<()>> {
            let handler_inner = Arc::clone(&handler_clone);
            Box::pin(async move { handler_inner.handle(e, c).await })
        });

        // ミドルウェアチェーンを逆順に実行
        let mut next = Next::new(handler_fn);
        for middleware in chain.into_iter().rev() {
            let prev_next = next.clone();
            let middleware_clone = Arc::clone(&middleware);
            let next_handler = Arc::new(move |e: E, c: Ctx| -> BoxFuture<'static, Result<()>> {
                let pn = prev_next.clone();
                let middleware_inner = Arc::clone(&middleware_clone);
                Box::pin(async move { middleware_inner.handle(e, c, pn).await })
            });
            next = Next::new(next_handler);
        }

        // 最初のミドルウェアを実行
        next.run(event, context).await
    }

    /// ワーカーを実行
    pub async fn run(self, shutdown: CancellationToken) -> Result<()> {
        // source からメッセージストリームを取得 (subscriber -> source)
        let mut stream = self.source.subscribe().await?;
        let shutdown_token = shutdown.clone();

        // ハンドラとミドルウェアをArcでラップ
        let handler = Arc::new(self.handler);
        let middlewares: Vec<Arc<dyn Middleware<E, Ctx>>> = self.middlewares.into_iter().collect();
        let context = self.context;

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
                                Arc::clone(&handler),
                                &middlewares,
                                event,
                                context.clone(),
                            ).await;

                            // 結果に基づいてack/nackを行う
                            match result {
                                Ok(_) => {
                                    ack.ack().await?;
                                }
                                Err(e) => {
                                    // エラーがClassifyErrorを実装している場合は、エラーアクションに基づいて処理
                                    if let Some(classify_error) = e.downcast_ref::<Box<dyn ClassifyError>>() {
                                        match classify_error.error_action() {
                                            shared_core::error_handling::ErrorAction::Retry => {
                                                // nack（再試行）
                                                // JetStreamの場合、ackしないと自動的に再配信される
                                            }
                                            shared_core::error_handling::ErrorAction::Ignore => {
                                                // エラーを無視してack
                                                ack.ack().await?;
                                            }
                                        }
                                    } else {
                                        // ClassifyErrorを実装していない場合はデフォルトでRetry
                                        // nack（再試行）
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
