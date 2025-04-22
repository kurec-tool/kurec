//! JetStream メッセージ関連の型定義

use crate::error::AckError; // 後で定義するエラー型
use async_nats::jetstream;
use domain::event::Event; // ドメイン層の Event トレイト
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use tracing::error; // エラーログ用

/// イベントペイロードと元の JetStream メッセージを保持する構造体。
/// Ack 機能を提供する。
#[derive(Debug)]
pub struct EventMessage<E: Event> {
    /// デシリアライズされたイベントペイロード
    payload: E,
    /// 元の JetStream メッセージ (Ack/Nack用)
    message: jetstream::Message,
}

impl<E> EventMessage<E>
where
    E: Event + DeserializeOwned + Debug + Send + Sync + 'static,
{
    /// 新しい EventMessage を作成する (内部用)。
    pub(crate) fn new(payload: E, message: jetstream::Message) -> Self {
        Self { payload, message }
    }

    /// イベントペイロードへの参照を取得する。
    pub fn payload(&self) -> &E {
        &self.payload
    }

    /// メッセージを Ack する。
    pub async fn ack(&self) -> Result<(), AckError> {
        self.message.ack().await.map_err(|e| {
            error!(error = %e, "Failed to ack message");
            AckError::NatsError(e)
        })?;
        Ok(())
    }

    // 必要に応じて Nack や Term などのメソッドも追加可能
    // pub async fn nack(&self) -> Result<(), AckError> { ... }
    // pub async fn term(&self) -> Result<(), AckError> { ... }
}
