//! MirakcApi トレイトの実装

use anyhow::{Context, Result};
use async_trait::async_trait;
use domain::ports::mirakc_api::MirakcApi;
use mirakc_client::apis::configuration::Configuration; // mirakc_client:: を使用
use mirakc_client::apis::{programs_api, services_api}; // mirakc_client:: を使用
use reqwest::Client;
// use mirakc_client::models::{MirakurunProgram, MirakurunService}; // 直接返さないので不要
use serde_json::Value;
use url::Url;

/// reqwest を使用した MirakcApi の実装
#[derive(Clone)] // Clone を追加
pub struct MirakcApiClientImpl {
    client: Client,
    // base_url はメソッド呼び出し時に渡されるため、フィールドとして保持しない
}

impl MirakcApiClientImpl {
    /// 新しい MirakcApiClientImpl を作成する。
    pub fn new() -> Self {
        Self {
            client: Client::new(), // reqwest::Client を初期化
        }
    }

    /// ベースURLとパスから完全なURLを構築するヘルパー関数
    fn build_url(base_url: &str, path: &str) -> Result<Url> {
        let base = Url::parse(base_url).context("Invalid base URL")?;
        base.join(path).context("Failed to join URL path")
    }
}

#[async_trait]
impl MirakcApi for MirakcApiClientImpl {
    async fn get_service(&self, mirakc_url: &str, service_id: i64) -> Result<Value> {
        // u64 -> i64 に戻す
        // mirakc_client の Configuration を動的に作成
        let config = Configuration {
            base_path: mirakc_url.to_string(),
            user_agent: Some("kurec/0.1.0".to_string()),
            client: self.client.clone(),
            ..Default::default()
        };

        let service = services_api::get_service(&config, service_id) // キャスト不要
            .await
            .context(format!(
                "Failed to get service {} from {}",
                service_id, mirakc_url
            ))?;

        // MirakurunService を serde_json::Value に変換
        serde_json::to_value(service).context("Failed to serialize service to JSON Value")
    }

    async fn get_programs_of_service(
        &self,
        mirakc_url: &str,
        service_id: i64, // u64 -> i64 に戻す
    ) -> Result<Vec<Value>> {
        // mirakc_client の Configuration を動的に作成
        let config = Configuration {
            base_path: mirakc_url.to_string(),
            user_agent: Some("kurec/0.1.0".to_string()),
            client: self.client.clone(),
            ..Default::default()
        };

        // services_api を使うように修正
        let programs = services_api::get_programs_of_service(&config, service_id) // キャスト不要
            .await
            .context(format!(
                "Failed to get programs for service {} from {}",
                service_id, mirakc_url
            ))?;

        // Vec<MirakurunProgram> を Vec<serde_json::Value> に変換
        programs
            .into_iter()
            .map(|p| serde_json::to_value(p).context("Failed to serialize program to JSON Value"))
            .collect()
    }
}
