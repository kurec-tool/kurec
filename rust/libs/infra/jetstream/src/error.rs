use async_nats::jetstream::{
    context::{CreateStreamError, GetStreamError, PublishError, UpdateStreamError},
    stream::InfoError, // stream モジュールから InfoError をインポート
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JetstreamError {
    #[error("NATS client error: {0}")]
    Nats(#[from] async_nats::Error),

    // JetStream API エラーを個別に定義
    #[error("JetStream get stream error: {0}")]
    GetStream(GetStreamError),

    #[error("JetStream create stream error: {0}")]
    CreateStream(CreateStreamError),

    #[error("JetStream update stream error: {0}")]
    UpdateStream(UpdateStreamError),

    #[error("JetStream get stream info error: {0}")]
    GetStreamInfo(#[from] InfoError), // From<InfoError> を実装 (これは衝突しないはず)

    #[error("JetStream publish error: {0}")]
    Publish(#[from] PublishError),

    #[error("Stream configuration error: {0}")]
    StreamConfig(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscribe error: {0}")]
    Subscribe(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

// 衝突するため、GetStreamError, CreateStreamError, UpdateStreamError の From 実装は削除
// impl From<GetStreamError> for JetstreamError { ... }
// impl From<CreateStreamError> for JetstreamError { ... }
// impl From<UpdateStreamError> for JetstreamError { ... }
