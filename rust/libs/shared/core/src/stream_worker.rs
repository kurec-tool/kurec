use crate::error_handling::ClassifyError;
use crate::event_metadata::Event;
use crate::event_sink::EventSink; // リネーム
use crate::event_source::EventSource; // リネーム
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
    // 戻り値を Option<O> に変更
    async fn handle(&self, event: I, next: StreamNext<'_, I, O, E>) -> Result<Option<O>, E>;
}

/// ミドルウェアチェーンの次の処理を表す構造体
pub struct StreamNext<'a, I, O, E>
where
    I: Event,
    O: Event,
    E: ClassifyError + Send + Sync + 'static,
{
    // ハンドラの型と戻り値を Option<O> に変更
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
    // ハンドラの型と戻り値を Option<O> に変更
    pub fn new(
        handler: Arc<dyn Fn(I) -> BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }

    // 戻り値を Option<O> に変更
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
    // ハンドラの型を Option<O> に変更
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
    // O: Event, // このトレイト境界を削除 (ハンドラは Event を返さなくても良い)
    E: ClassifyError + Send + Sync + 'static,
{
    // 戻り値を Option<O> に変更
    async fn handle(&self, event: I) -> Result<Option<O>, E>;
}

/// 関数をハンドラとして扱うためのラッパー
pub struct FnStreamHandler<I, O, E, F>
where
    I: Event,
    // O: Event, // このトレイト境界を削除
    E: ClassifyError + Send + Sync + 'static,
    // 関数の戻り値を Option<O> に変更
    F: Fn(I) -> BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static,
{
    f: F,
    _phantom: PhantomData<(I, O, E)>,
}

impl<I, O, E, F> FnStreamHandler<I, O, E, F>
where
    I: Event,
    // O: Event, // このトレイト境界を削除
    E: ClassifyError + Send + Sync + 'static,
    // 関数の戻り値を Option<O> に変更
    F: Fn(I) -> BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static,
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
    O: Send + Sync + 'static, // Send + Sync 境界を追加
    E: ClassifyError + Send + Sync + 'static,
    // 関数の戻り値を Option<O> に変更
    F: Fn(I) -> BoxFuture<'static, Result<Option<O>, E>> + Send + Sync + 'static,
{
    // 戻り値を Option<O> に変更
    async fn handle(&self, event: I) -> Result<Option<O>, E> {
        (self.f)(event).await
    }
}

/// ストリームワーカー
/// 入力イベントを処理して出力イベントを生成するワーカー
// ジェネリック F を削除し、ハンドラをトレイトオブジェクトに変更
pub struct StreamWorker<I, O, E>
where
    I: Event,
    O: Event, // Sink が Event を要求するため、StreamWorker の O には Event 境界が必要
    E: ClassifyError + Send + Sync + 'static,
{
    source: Arc<dyn EventSource<I>>, // subscriber -> source にリネーム
    sink: Arc<dyn EventSink<O>>,     // publisher -> sink にリネーム
    handler: Arc<dyn StreamHandler<I, O, E>>, // F -> Arc<dyn StreamHandler> に変更
    middlewares: Vec<Arc<dyn StreamMiddleware<I, O, E>>>,
    durable_name: Option<String>,
}

// ジェネリック F を削除
impl<I, O, E> StreamWorker<I, O, E>
where
    I: Event,
    O: Event, // Sink が Event を要求するため、StreamWorker の O には Event 境界が必要
    E: ClassifyError + Send + Sync + 'static,
{
    /// 新しいStreamWorkerを作成
    // シグネチャを変更
    pub fn new(
        source: Arc<dyn EventSource<I>>,
        sink: Arc<dyn EventSink<O>>,
        handler: Arc<dyn StreamHandler<I, O, E>>,
    ) -> Self {
        Self {
            source,
            sink,
            handler, // Arc<dyn StreamHandler> を受け取る
            middlewares: Vec::new(),
            durable_name: None,
        }
    }

    /// ミドルウェアを追加
    pub fn with_middleware<M>(mut self, middleware: M) -> Self
    where
        // M の戻り値を Option<O> に変更
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
    // シグネチャと戻り値を変更
    async fn execute_middleware_chain(
        handler: Arc<dyn StreamHandler<I, O, E>>, // F -> Arc<dyn StreamHandler>
        middlewares: &[Arc<dyn StreamMiddleware<I, O, E>>],
        event: I,
    ) -> Result<Option<O>, E> {
        // Result<O, E> -> Result<Option<O>, E>
        // ミドルウェアがない場合は直接ハンドラを実行
        if middlewares.is_empty() {
            return handler.handle(event).await; // handler(event) -> handler.handle(event)
        }

        // ミドルウェアチェーンを構築
        let mut chain = Vec::new();
        for middleware in middlewares {
            chain.push(Arc::clone(middleware));
        }

        // 最後のミドルウェアの次の処理はハンドラ
        let handler_clone = handler.clone(); // handler は Arc なので clone するだけ
                                             // ハンドラの型と戻り値を Option<O> に変更
        let handler_fn = Arc::new(move |e: I| -> BoxFuture<'static, Result<Option<O>, E>> {
            let handler_inner = handler_clone.clone();
            Box::pin(async move { handler_inner.handle(e).await }) // handler_inner(e) -> handler_inner.handle(e)
        });

        // ミドルウェアチェーンを逆順に実行
        let mut next = StreamNext::new(handler_fn);
        for middleware in chain.into_iter().rev() {
            let prev_next = next.clone();
            let middleware_clone = Arc::clone(&middleware);
            // ハンドラの型と戻り値を Option<O> に変更
            let next_handler = Arc::new(move |e: I| -> BoxFuture<'static, Result<Option<O>, E>> {
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
        // source からメッセージストリームを取得 (subscriber -> source)
        let mut stream = self.source.subscribe().await?;
        let shutdown_token = shutdown.clone();

        // ハンドラとミドルウェアをArcでラップ (handler は既に Arc)
        let handler = self.handler; // clone 不要
        let middlewares: Vec<Arc<dyn StreamMiddleware<I, O, E>>> = self.middlewares;
        let sink = self.sink; // publisher -> sink

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
                            // ミドルウェアチェーンを実行 (handler.clone() 不要)
                            let result = Self::execute_middleware_chain(
                                handler.clone(), // handler は Arc なので clone
                                &middlewares,
                                event,
                            ).await;

                            match result {
                                // 戻り値が Option<O> になったので Some の場合のみ publish
                                Ok(Some(output_event)) => {
                                    // 出力イベントを sink に発行 (publisher -> sink)
                                    sink.publish(output_event).await?;
                                    ack.ack().await?;
                                }
                                Ok(None) => {
                                    // 出力イベントがない場合は ack のみ
                                    ack.ack().await?;
                                }
                                Err(e) => {
                                    // エラーアクションに基づいて処理
                                    match e.error_action() {
                                        crate::error_handling::ErrorAction::Retry => {
                                            // nack（再試行）
                                            // JetStreamの場合、ackしないと自動的に再配信される
                                            // 何もしない
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
