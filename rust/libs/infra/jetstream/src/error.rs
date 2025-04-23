//! infra/jetstream クレートのエラー定義

use thiserror::Error;
use tracing::{error, warn};

/// イベント発行時のエラー
#[derive(Error, Debug)]
pub enum PublishError {
    #[error("Failed to serialize event: {0}")]
    SerializationError(serde_json::Error),
    #[error("Failed to get stream info: {0}")]
    GetStreamError(async_nats::Error),
    #[error("Failed to create stream: {0}")]
    CreateStreamError(async_nats::Error),
    #[error("Failed to publish message: {0}")]
    PublishError(async_nats::jetstream::context::PublishError),
    #[error("NATS client error: {0}")]
    NatsClientError(anyhow::Error),
    #[error("Internal error: {0}")]
    Internal(anyhow::Error),
}

/// イベント購読時のエラー
#[derive(Error, Debug)]
pub enum SubscribeError {
    #[error("Failed to get stream info: {0}")]
    GetStreamError(async_nats::Error),
    #[error("Failed to create consumer: {0}")]
    CreateConsumerError(async_nats::Error),
    #[error("Failed to get messages from consumer: {0}")]
    MessagesError(async_nats::Error),
    #[error("NATS client error: {0}")]
    NatsClientError(anyhow::Error),
    #[error("Internal error: {0}")]
    Internal(anyhow::Error),
    #[error("Failed to deserialize message: {0}")]
    DeserializationError(serde_json::Error),
}

/// メッセージ Ack 時のエラー
#[derive(Error, Debug)]
pub enum AckError {
    #[error("Failed to send ack to NATS: {0}")]
    NatsError(async_nats::Error),
}

/// JetStreamイベント処理時のエラー
#[derive(Debug, Error)]
pub enum JsEventError {
    #[error("Failed to deserialize message: {source}")]
    Deserialize {
        #[source]
        source: serde_json::Error,
        payload: Vec<u8>,
    },

    #[error("Connection error: {source}")]
    Connection {
        #[source]
        source: async_nats::Error,
        endpoint: String,
    },

    #[error("Stream error: {source}")]
    Stream {
        #[source]
        source: async_nats::jetstream::consumer::pull::MessagesError,
    },

    #[error("Get consumer error: {source}")]
    GetConsumer {
        #[source]
        source: async_nats::Error,
    },

    #[error("Create consumer error: {source}")]
    CreateConsumer {
        #[source]
        source: async_nats::Error,
    },

    #[error("Ack error: {source}")]
    Ack {
        #[source]
        source: async_nats::Error,
    },
}

impl JsEventError {
    /// エラーが再試行可能かどうかを判断する
    pub fn should_retry(&self) -> bool {
        match self {
            Self::Deserialize { .. } => false, // デシリアライズエラーは再試行しても同じ結果になる
            Self::Connection { .. } => true,   // 接続エラーは再試行する価値がある
            Self::Stream { .. } => true,       // ストリームエラーは再試行する価値がある
            Self::GetConsumer { .. } => true,  // コンシューマ取得エラーは再試行する価値がある
            Self::CreateConsumer { .. } => true, // コンシューマ作成エラーは再試行する価値がある
            Self::Ack { .. } => true,          // Ackエラーは再試行する価値がある
        }
    }

    /// エラーの詳細をログに記録する
    pub fn log(&self) {
        match self {
            Self::Deserialize { payload, .. } => {
                error!(
                    error = %self,
                    payload_size = payload.len(),
                    payload_preview = %String::from_utf8_lossy(&payload[..std::cmp::min(100, payload.len())]),
                    "Failed to deserialize message"
                );
            }
            Self::Connection { endpoint, .. } => {
                warn!(
                    error = %self,
                    endpoint = %endpoint,
                    "Connection error"
                );
            }
            Self::Stream { .. } => {
                warn!(
                    error = %self,
                    "Stream error"
                );
            }
            Self::GetConsumer { .. } => {
                warn!(
                    error = %self,
                    "Get consumer error"
                );
            }
            Self::CreateConsumer { .. } => {
                warn!(
                    error = %self,
                    "Create consumer error"
                );
            }
            Self::Ack { .. } => {
                warn!(
                    error = %self,
                    "Ack error"
                );
            }
        }
    }
}
