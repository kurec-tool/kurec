// use domain::event::Event; // domain への依存を削除
use serde::{de::DeserializeOwned, Serialize}; // 必要なトレイト境界を直接指定

/// EventSink: イベントを発行するトレイト
#[async_trait::async_trait]
pub trait EventSink<E>: Send + Sync + 'static
where
    E: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// イベントを発行する
    async fn publish(&self, event: E) -> anyhow::Result<()>;
}
