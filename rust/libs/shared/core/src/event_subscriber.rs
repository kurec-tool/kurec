use anyhow::Result;
use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::stream::BoxStream;

/// AckHandle: 非同期でメッセージの確認応答を行うハンドル
pub struct AckHandle {
    ack_fn: Box<dyn Fn() -> BoxFuture<'static, Result<()>> + Send + Sync>,
}

impl AckHandle {
    /// 新しいAckHandleを作成
    pub fn new(ack_fn: Box<dyn Fn() -> BoxFuture<'static, Result<()>> + Send + Sync>) -> Self {
        Self { ack_fn }
    }

    /// メッセージを確認応答（ack）する
    pub async fn ack(self) -> Result<()> {
        (self.ack_fn)().await
    }
}

use crate::event_metadata::Event;

/// EventSubscriber: 指定された subject を購読し、
/// `(Message, AckHandle)` のストリームを返すトレイト
#[async_trait]
pub trait EventSubscriber<E: Event>: Send + Sync + 'static {
    /// 指定 subject と durable 名で購読し、メッセージと AckHandle のストリームを返す
    async fn subscribe(&self) -> Result<BoxStream<'static, (E, AckHandle)>>;
}
