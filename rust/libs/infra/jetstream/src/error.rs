//! infra/jetstream クレートのエラー定義

use thiserror::Error;

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
