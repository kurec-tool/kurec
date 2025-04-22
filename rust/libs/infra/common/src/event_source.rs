//! イベントソースの抽象化を提供するモジュール

use anyhow::Result;
use async_trait::async_trait;
use domain::event::Event;
use domain::ports::event_source::EventSource as DomainEventSource;
use futures::{stream::BoxStream, StreamExt};

use crate::ackable_event::AckableEvent;

/// イベントソースのトレイト
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
    /// - `Ok(stream)`: 購読に成功した場合、AckableEventのストリームを返す
    /// - `Err(e)`: 購読に失敗した場合
    async fn subscribe(&self) -> Result<BoxStream<'static, Result<AckableEvent<E>, Err>>>;
}

/// ドメイン層のEventSourceトレイトを実装するためのアダプター関数
///
/// この関数は、インフラ層のEventSourceをドメイン層のEventSourceに変換します。
/// AckableEventを自動的にAckし、イベントのみを返すようにします。
pub async fn adapt_event_source<S, E, Err>(source: &S) -> Result<BoxStream<'static, Result<E, Err>>>
where
    S: EventSource<E, Err> + Send + Sync + 'static,
    E: Event + Clone,
    Err: Send + Sync + 'static,
{
    let ackable_stream = source.subscribe().await?;

    // AckableEventストリームをイベントストリームに変換
    let event_stream = ackable_stream
        .map(|result| {
            match result {
                Ok(mut ackable_event) => {
                    let event = ackable_event.event().clone();
                    // 自動的にAck
                    tokio::spawn(async move {
                        if let Err(e) = ackable_event.ack().await {
                            tracing::error!(error = %e, "Failed to ack event automatically");
                        }
                    });
                    Ok(event)
                }
                Err(e) => Err(e),
            }
        })
        .boxed();

    Ok(event_stream)
}
