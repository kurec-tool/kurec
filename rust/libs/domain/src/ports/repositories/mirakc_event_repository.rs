//! mirakcイベントリポジトリのインターフェース

use async_trait::async_trait;
use futures::stream::BoxStream;
use shared_core::dtos::mirakc_event::MirakcEventDto;

/// mirakcイベントリポジトリのインターフェース
#[async_trait]
pub trait MirakcEventRepository: Send + Sync + 'static {
    /// mirakcサーバーからイベントストリームを取得
    async fn get_event_stream(&self) -> anyhow::Result<BoxStream<'static, MirakcEventDto>>;
}
