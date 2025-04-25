//! MirakcApi トレイトの実装

use anyhow::{Context, Result};
use async_trait::async_trait;
use domain::ports::mirakc_api::MirakcApi;
use mirakc_client::apis::configuration::Configuration; // mirakc_client:: を使用
use mirakc_client::apis::services_api; // mirakc_client:: を使用
use reqwest::Client;
use serde_json::Value;

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
}

impl Default for MirakcApiClientImpl {
    fn default() -> Self {
        Self::new()
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
