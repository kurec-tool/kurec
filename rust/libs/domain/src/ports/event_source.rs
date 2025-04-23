use crate::event::Event;
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;

/// ドメインイベントを購読するためのインターフェース (ポート)
///
/// このトレイトは、イベントを購読するためのインターフェースを提供します。
/// 具体的な実装は、JetStreamやSSEなどのメッセージングシステムに対して行われます。
#[async_trait]
pub trait EventSource<E, Err>: Send + Sync + 'static
where
    E: Event,
    Err: Send + Sync + 'static,
{
    /// イベントを購読する
    ///
    /// # Returns
    /// - `Ok(stream)`: 購読に成功した場合、イベントのストリームを返す
    /// - `Err(e)`: 購読に失敗した場合
    async fn subscribe(&self) -> Result<BoxStream<'static, Result<E, Err>>>;
}
