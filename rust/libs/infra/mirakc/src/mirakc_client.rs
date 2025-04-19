use anyhow::{Context, Result};
use mirakc_client::apis::{configuration::Configuration, version_api};
use std::sync::Arc;

/// mirakcクライアントのラッパー
#[derive(Clone)]
pub struct MirakcClient {
    config: Arc<Configuration>,
}

impl MirakcClient {
    /// 新しいMirakcClientを作成
    pub fn new(base_url: &str) -> Self {
        let mut config = Configuration::new();
        // base_pathはデフォルトで "/api" なので、完全なURLを構築
        config.base_path = format!("{}/api", base_url);
        Self { config: Arc::new(config) }
    }

    /// バージョン情報を取得
    pub async fn get_version(&self) -> Result<mirakc_client::models::Version> {
        version_api::check_version(&self.config)
            .await
            .context("Failed to get mirakc version")
            .map_err(Into::into)
    }

    // 将来的に他のAPIメソッドを追加...
}
