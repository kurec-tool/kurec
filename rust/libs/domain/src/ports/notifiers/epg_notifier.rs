use anyhow::Result;
use async_trait::async_trait;

use crate::events::kurec_events::EpgStoredEvent;

/// EPG情報がKVSに保存されたことを通知するためのトレイト。
/// 主にJetStreamへのイベント発行を想定。
#[async_trait]
pub trait EpgNotifier: Send + Sync {
    /// `EpgStoredEvent` を通知する。
    ///
    /// # Arguments
    ///
    /// * `event` - 通知するイベント (`EpgStoredEvent`)
    ///
    /// # Returns
    ///
    /// 通知に成功した場合は `Ok(())`、失敗した場合は `Err`。
    async fn notify_epg_stored(&self, event: EpgStoredEvent) -> Result<()>;
}
