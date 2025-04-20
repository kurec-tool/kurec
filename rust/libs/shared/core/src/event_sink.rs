use crate::event_metadata::Event;

/// EventSink: イベントを発行するトレイト
#[async_trait::async_trait]
pub trait EventSink<E: Event>: Send + Sync + 'static {
    /// イベントを発行する
    async fn publish(&self, event: E) -> anyhow::Result<()>;
}
