use crate::models::version::Version;
use anyhow::Result;
use async_trait::async_trait;

/// mirakcバージョンリポジトリのインターフェース
#[async_trait]
pub trait VersionRepository: Send + Sync + 'static {
    /// バージョン情報を取得
    async fn get_version(&self) -> Result<Version>;
}
