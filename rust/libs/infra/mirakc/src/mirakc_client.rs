use anyhow::{Context, Result};
use mirakc_client::apis::{configuration::Configuration, services_api, version_api};
use mirakc_client::models::{MirakurunProgram, MirakurunService};
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
        Self {
            config: Arc::new(config),
        }
    }

    /// バージョン情報を取得
    pub async fn get_version(&self) -> Result<mirakc_client::models::Version> {
        version_api::check_version(&self.config)
            .await
            .context("Failed to get mirakc version")
    }

    /// サービス情報を取得
    pub async fn get_service(&self, service_id: u64) -> Result<MirakurunService> {
        services_api::get_service(&self.config, service_id as i64)
            .await
            .context("Failed to get service")
    }

    /// サービスのプログラム一覧を取得
    pub async fn get_programs_of_service(&self, service_id: u64) -> Result<Vec<MirakurunProgram>> {
        services_api::get_programs_of_service(&self.config, service_id as i64)
            .await
            .context("Failed to get programs of service")
    }
}
