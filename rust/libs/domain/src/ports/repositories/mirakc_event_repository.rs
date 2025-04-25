//! mirakcイベントリポジトリのインターフェース

use crate::events::MirakcEventInput; // domain::events::MirakcEventInput をインポート
use async_trait::async_trait;
use futures::stream::BoxStream;
// 不要な DTO インポートを削除: use shared_core::dtos::mirakc_event::MirakcEventDto;

/// mirakcイベントリポジトリのインターフェース
#[async_trait]
pub trait MirakcEventRepository: Send + Sync + 'static {
    /// mirakcサーバーからイベントストリームを取得
    // 返り値の型を MirakcEventInput に変更
    async fn get_event_stream(&self) -> anyhow::Result<BoxStream<'static, MirakcEventInput>>;
}
