use crate::event::Event; // domain クレート内の Event トレイトを使用
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
// AckHandle は削除

// TODO: 戻り値の型を infra 層に依存しない形にする必要がある。
//       例えば、ペイロードと Ack/Nack 機能を持つジェネリックなトレイトを定義するなど。
//       現状は infra_jetstream の型を使う仮実装とする。
// use infra_jetstream::error::SubscribeError;
// use infra_jetstream::message::EventMessage;

use serde::de::DeserializeOwned; // 追加

/// ドメインイベントを購読するためのインターフェース (ポート)
#[async_trait]
pub trait EventSource<E>: Send + Sync + 'static
where
    E: DeserializeOwned + Send + Sync + 'static,
{
    /// イベントを購読し、Ack/Nack 可能なメッセージのストリームを返す。
    /// TODO: 戻り値の型を修正する (infra に依存しないように)
    async fn subscribe(
        &self,
    ) -> Result<
        BoxStream<'static, Result</* EventMessage<E> */ E, /* SubscribeError */ anyhow::Error>>,
    >;
    // 仮実装: Result<E, anyhow::Error> を返すようにしておく
}
