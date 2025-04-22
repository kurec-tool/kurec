use anyhow::Result;
use async_trait::async_trait;
use shared_core::event::Event; // shared_core の Event トレイトを使用
use std::sync::Arc;

/// ドメインイベントを発行するためのインターフェース (ポート)
///
/// このトレイトはインフラ層 (例: JetStream) で実装され、
/// ドメイン層やアプリケーション層からイベントを発行するために使用されます。
#[async_trait]
pub trait DomainEventSink<E: Event>: Send + Sync {
    /// イベントを発行します。
    async fn publish(&self, event: E) -> Result<()>;
}

// Arc<dyn DomainEventSink<E>> も DomainEventSink<E> として扱えるようにする
#[async_trait]
impl<E: Event> DomainEventSink<E> for Arc<dyn DomainEventSink<E>> {
    async fn publish(&self, event: E) -> Result<()> {
        self.as_ref().publish(event).await
    }
}
