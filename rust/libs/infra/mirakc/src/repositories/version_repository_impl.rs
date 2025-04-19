use anyhow::Result;
use async_trait::async_trait;

use shared_core::dtos::version_dto::VersionDto;
use shared_core::repositories::version_repository::VersionRepository;

use crate::mirakc_client::MirakcClient;

/// VersionRepositoryの実装
pub struct VersionRepositoryImpl {
    client: MirakcClient,
}

impl VersionRepositoryImpl {
    /// 新しいVersionRepositoryImplを作成
    pub fn new(base_url: &str) -> Self {
        Self {
            client: MirakcClient::new(base_url),
        }
    }
}

#[async_trait]
impl VersionRepository for VersionRepositoryImpl {
    async fn get_version(&self) -> Result<VersionDto> {
        let mirakc_version = self.client.get_version().await?;

        // mirakcのモデルからDTOに変換
        Ok(VersionDto {
            current: mirakc_version.current,
            latest: mirakc_version.latest,
        })
    }
}
