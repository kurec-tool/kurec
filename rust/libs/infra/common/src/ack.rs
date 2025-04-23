//! イベントのAck（確認応答）機能を提供するモジュール

use anyhow::Result;
use async_trait::async_trait;

/// イベントのAck（確認応答）機能を提供するトレイト
#[async_trait]
pub trait Ack: Send + Sync + 'static {
    /// イベントを確認応答（Ack）する
    ///
    /// # Returns
    /// - `Ok(())`: Ackが成功した場合
    /// - `Err(e)`: Ackが失敗した場合
    async fn ack(&self) -> Result<()>;
}
