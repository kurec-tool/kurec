//! JetStream用のAck実装を提供するモジュール

use anyhow::Result;
use async_nats::jetstream;
use async_trait::async_trait;
use infra_common::ack::Ack;
use tracing::error;

use crate::error::JsEventError;

/// JetStream用のAck実装
pub struct JsAck {
    message: jetstream::Message,
}

impl JsAck {
    /// 新しいJsAckを作成
    ///
    /// # Arguments
    /// * `message` - JetStreamのメッセージ
    pub fn new(message: jetstream::Message) -> Self {
        Self { message }
    }
}

#[async_trait]
impl Ack for JsAck {
    async fn ack(&self) -> Result<()> {
        self.message.ack().await.map_err(|e| {
            error!(error = %e, "Failed to ack message");
            JsEventError::Ack { source: e }.into()
        })
    }
}
