use anyhow::Result;
use async_trait::async_trait;
use domain::models::version::Version;
use domain::ports::repositories::version_repository::VersionRepository;

use crate::mirakc_client::MirakcClient;

/// domainのVersionRepositoryの実装
pub struct DomainVersionRepositoryImpl {
    client: MirakcClient,
}

impl DomainVersionRepositoryImpl {
    /// 新しいDomainVersionRepositoryImplを作成
    pub fn new(base_url: &str) -> Self {
        Self {
            client: MirakcClient::new(base_url),
        }
    }
}

#[async_trait]
impl VersionRepository for DomainVersionRepositoryImpl {
    async fn get_version(&self) -> Result<Version> {
        let mirakc_version = self.client.get_version().await?;

        // mirakcのモデルからVersionに変換
        Ok(Version {
            current: mirakc_version.current,
            latest: mirakc_version.latest,
        })
    }
}
