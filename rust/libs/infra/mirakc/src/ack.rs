//! SSE用のAck実装を提供するモジュール

use anyhow::Result;
use async_trait::async_trait;
use infra_common::ack::Ack;

/// SSE用のAck実装
pub struct SseAck {}

impl SseAck {
    /// 新しいSseAckを作成
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Ack for SseAck {
    async fn ack(&self) -> Result<()> {
        // SSEはAck機能がないので何もしない
        Ok(())
    }
}
