use anyhow::Result;
use async_trait::async_trait;
use domain::ports::EventSource;
use futures::future::BoxFuture;
use futures::StreamExt;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;

use serde::{de::DeserializeOwned, Serialize}; // 追加

/// ワーカーのミドルウェアトレイト
/// イベント処理の前後に処理を挟むことができる
#[async_trait]
pub trait Middleware<E, Ctx>: Send + Sync + 'static
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
    Ctx: Clone + Send + Sync + 'static,
{
    async fn handle(&self, event: E, ctx: Ctx, next: Next<'_, E, Ctx>) -> Result<()>;
}

/// ミドルウェアチェーンの次の処理を表す構造体
pub struct Next<'a, E, Ctx>
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
    Ctx: Clone + Send + Sync + 'static,
{
    pub(crate) handler:
        Arc<dyn Fn(E, Ctx) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static>,
    _phantom: PhantomData<&'a ()>,
}

impl<E, Ctx> Next<'_, E, Ctx>
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
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
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
    Ctx: Clone + Send + Sync + 'static,
{
    async fn handle(&self, event: E, ctx: Ctx) -> Result<()>;
}

/// 関数をハンドラとして扱うためのラッパー
pub struct FnHandler<E, Ctx, F>
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
    Ctx: Clone + Send + Sync + 'static,
    F: Fn(E, Ctx) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
{
    f: F,
    _phantom: PhantomData<(E, Ctx)>,
}

impl<E, Ctx, F> FnHandler<E, Ctx, F>
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
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
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
    Ctx: Clone + Send + Sync + 'static,
    F: Fn(E, Ctx) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
{
    async fn handle(&self, event: E, ctx: Ctx) -> Result<()> {
        (self.f)(event, ctx).await
    }
}

impl<E, Ctx> Clone for Next<'_, E, Ctx>
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
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
pub struct WorkerBuilder<E, H, Ctx, Src>
// EventSource をジェネリックに
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
    H: Handler<E, Ctx>,
    Ctx: Clone + Send + Sync + 'static,
    Src: EventSource<E> + 'static, // EventSource トレイト境界を追加
{
    source: Arc<Src>, // Arc<dyn EventSource<E>> -> Arc<Src>
    handler: H,
    context: Ctx,
    middlewares: Vec<Arc<dyn Middleware<E, Ctx>>>,
    durable_name: Option<String>,
}

impl<E, H, Ctx, Src> WorkerBuilder<E, H, Ctx, Src>
// ジェネリックパラメータ追加
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
    H: Handler<E, Ctx> + Clone,
    Ctx: Clone + Send + Sync + 'static,
    Src: EventSource<E> + 'static, // EventSource トレイト境界を追加
{
    /// 新しいWorkerBuilderを作成
    pub fn new(source: Arc<Src>, handler: H, context: Ctx) -> Self {
        // 引数型変更
        Self {
            source,
            handler,
            context,
            middlewares: Vec::new(),
            durable_name: None,
        }
    }

    /// ミドルウェアを追加
    pub fn with_middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware<E, Ctx> + 'static, // Middleware の E 境界は修正済みのはず
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
        // EventSource::subscribe の戻り値型が仮実装 (Result<BoxStream<'static, Result<E, anyhow::Error>>>) であることに注意
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
                message_result = stream.next() => { // 変数名を変更
                    match message_result {
                        // subscribe の仮実装に合わせる (Result<E, anyhow::Error>)
                        Some(Ok(event)) => {
                            // ミドルウェアチェーンを実行
                            let result = Self::execute_middleware_chain(
                                Arc::clone(&handler),
                                &middlewares,
                                event, // event を直接渡す
                                context.clone(),
                            ).await;

                            // 結果に基づいてログ出力 (Ack/Nack は EventSource 側で行う想定)
                            if let Err(e) = result {
                                // TODO: エラー処理の詳細化 (ClassifyError を使うなど)
                                eprintln!("Worker handler error: {:?}", e);
                            }
                            // TODO: 本来はここで Ack/Nack を行う必要があるが、
                            //       EventSource のシグネチャが仮実装のため、一旦何もしない。
                        }
                        Some(Err(e)) => {
                            // subscribe ストリーム自体のエラー
                            eprintln!("Subscription error: {:?}", e);
                            // エラーによってはリトライや終了処理が必要かもしれない
                        }
                        None => {
                            // ストリームが終了したらループを抜ける
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
