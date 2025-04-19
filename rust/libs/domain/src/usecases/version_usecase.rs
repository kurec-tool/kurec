use crate::models::version::{Version, VersionStatus};
use crate::ports::repositories::version_repository::VersionRepository;
use anyhow::Result;

/// mirakcバージョン確認ユースケース
pub struct VersionUseCase<R: VersionRepository> {
    repository: R,
}

impl<R: VersionRepository> VersionUseCase<R> {
    /// 新しいVersionUseCaseを作成
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// mirakcのバージョン状態を取得
    pub async fn get_version_status(&self) -> Result<(Version, VersionStatus)> {
        let version = self.repository.get_version().await?;
        let status = version.version_status()?;
        Ok((version, status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::Mutex;

    // モックリポジトリ
    struct MockVersionRepository {
        version: Arc<Mutex<Version>>,
    }

    impl MockVersionRepository {
        fn new(current: &str, latest: &str) -> Self {
            Self {
                version: Arc::new(Mutex::new(Version {
                    current: current.to_string(),
                    latest: latest.to_string(),
                })),
            }
        }
    }

    #[async_trait]
    impl VersionRepository for MockVersionRepository {
        async fn get_version(&self) -> Result<Version> {
            let version = self.version.lock().unwrap().clone();
            Ok(version)
        }
    }

    #[tokio::test]
    async fn test_get_version_status_up_to_date() {
        let repo = MockVersionRepository::new("1.0.0", "1.0.0");
        let usecase = VersionUseCase::new(repo);

        let (version, status) = usecase.get_version_status().await.unwrap();

        assert_eq!(version.current, "1.0.0");
        assert_eq!(version.latest, "1.0.0");
        assert_eq!(status, VersionStatus::UpToDate);
    }

    #[tokio::test]
    async fn test_get_version_status_update_available() {
        let repo = MockVersionRepository::new("1.0.0", "2.0.0");
        let usecase = VersionUseCase::new(repo);

        let (version, status) = usecase.get_version_status().await.unwrap();

        assert_eq!(version.current, "1.0.0");
        assert_eq!(version.latest, "2.0.0");
        assert_eq!(status, VersionStatus::MajorUpdate);
    }

    #[tokio::test]
    async fn test_get_version_status_dev_version() {
        let repo = MockVersionRepository::new("2.0.0-dev.1", "1.9.0");
        let usecase = VersionUseCase::new(repo);

        let (version, status) = usecase.get_version_status().await.unwrap();

        assert_eq!(version.current, "2.0.0-dev.1");
        assert_eq!(version.latest, "1.9.0");
        assert_eq!(status, VersionStatus::Development);
    }
}
