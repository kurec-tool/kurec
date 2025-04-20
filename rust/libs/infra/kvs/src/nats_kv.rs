use anyhow::{Context, Result}; // anyhow::Result を使用
use async_nats::jetstream::kv::Store; // Entry を削除
use async_trait::async_trait;
use bytes::Bytes;
use tracing::{debug, error, instrument}; // anyhow::Result を使用

use domain::models::epg::KurecProgram;
use domain::ports::repositories::KurecProgramRepository;

// KvsError は anyhow::Error に変換されるため、このファイル内での Result<T, KvsError> は不要
// use crate::error::KvsError; // KvsError は使用しない

/// NATS KVストアを使用して `KurecProgramRepository` を実装する構造体。
#[derive(Debug, Clone)]
pub struct NatsKvProgramRepository {
    store: Store,
}

impl NatsKvProgramRepository {
    /// 新しい `NatsKvProgramRepository` を作成する。
    ///
    /// # Arguments
    ///
    /// * `store` - 使用するNATS KVストア (`async_nats::kv::Store`)
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// KVSで使用するキーを生成する。
    /// キーは `epg:{mirakc_url}:{service_id}` の形式。
    fn generate_key(mirakc_url: &str, service_id: i32) -> String {
        // URLに含まれる可能性のある特殊文字をエスケープまたは置換する必要があるか検討
        // NATSのキーとして安全な形式にする
        // 簡単のため、ここでは単純な結合のみ行う
        format!("epg:{}:{}", mirakc_url, service_id)
    }
}

#[async_trait]
impl KurecProgramRepository for NatsKvProgramRepository {
    #[instrument(skip(self, programs), fields(key = %Self::generate_key(mirakc_url, service_id), num_programs = programs.len()))]
    async fn save_service_programs(
        &self,
        mirakc_url: &str,
        service_id: i32,
        programs: Vec<KurecProgram>,
    ) -> Result<()> {
        // 戻り値は anyhow::Result<()> でOK
        let key = Self::generate_key(mirakc_url, service_id);
        debug!("Saving programs to NATS KV");

        // Vec<KurecProgram> をJSON文字列にシリアライズ
        let json_data =
            serde_json::to_string(&programs).context("Failed to serialize programs to JSON")?; // anyhow::Context を使用
        let bytes_data: Bytes = json_data.into(); // Bytesに変換

        // NATS KVに保存 (put)
        // 既存のキーがあれば上書きされる
        match self.store.put(&key, bytes_data).await {
            Ok(revision) => {
                debug!(revision, "Successfully saved programs to NATS KV");
                Ok(())
            }
            Err(e) => {
                error!("Failed to save programs to NATS KV: {}", e);
                Err(e).context("NATS KV put operation failed") // anyhow::Error に変換
            }
        }
    }

    #[instrument(skip(self), fields(key = %Self::generate_key(mirakc_url, service_id)))]
    async fn get_service_programs(
        &self,
        mirakc_url: &str,
        service_id: i32,
    ) -> Result<Option<Vec<KurecProgram>>> {
        // 戻り値は anyhow::Result<...> でOK
        let key = Self::generate_key(mirakc_url, service_id);
        debug!("Getting programs from NATS KV");

        // NATS KVから取得 (entry メソッドを使用)
        match self.store.entry(&key).await {
            // get -> entry に変更
            // ↓↓↓ entry メソッドの戻り値に合わせる
            Ok(Some(entry)) => {
                // entry の型は async_nats::jetstream::kv::Entry
                let json_data = String::from_utf8(entry.value.to_vec()) // entry.value フィールドを使用
                    .context("Failed to convert NATS KV value to UTF-8 string")?;
                let programs: Vec<KurecProgram> = serde_json::from_str(&json_data)
                    .context("Failed to deserialize programs from JSON")?;
                debug!(
                    revision = entry.revision, // entry.revision フィールドを使用
                    "Successfully got programs from NATS KV"
                );
                Ok(Some(programs))
            }
            Ok(None) => {
                // キーが存在しない場合 (get自体は成功) -> これが NotFound 相当
                debug!("No programs found in NATS KV for the key");
                Ok(None)
            }
            Err(e) => {
                // Ok(None) 以外のエラーは anyhow::Error として処理
                error!("Failed to get programs from NATS KV: {}", e);
                // エラーを anyhow::Error に変換して返す
                Err(anyhow::Error::new(e).context("NATS KV get operation failed"))
            }
        }
    }
} // impl ブロックの正しい閉じ括弧

// --- 単体テスト ---
#[cfg(test)]
mod tests {
    use super::*;
    use async_nats::connect;
    use chrono::{TimeZone, Utc}; // chrono を use
    use domain::models::epg::KurecProgram; // KurecSeriesInfo を削除
                                           // use rand; // rand は setup_test_kv 内で直接 ::random を使うので不要

    // Helper function to create a test NATS client and KV store
    // Note: This requires a running NATS server. For true unit tests,
    // consider mocking the Store trait. For integration tests, use testcontainers.
    // 戻り値に context と bucket_name を追加
    async fn setup_test_kv() -> (
        async_nats::Client,
        async_nats::jetstream::Context,
        Store,
        String,
    ) {
        // 環境変数などからNATS URLを取得 (テスト用に変更可能にする)
        let nats_url =
            std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
        let client = connect(&nats_url).await.expect("Failed to connect to NATS");
        let context = async_nats::jetstream::new(client.clone());
        let bucket_name = format!("test_kurec_epg_{}", rand::random::<u32>()); // バケット名を保存
        let store = context
            .create_key_value(async_nats::jetstream::kv::Config {
                // パスはOK
                bucket: bucket_name.clone(), // 保存したバケット名を使用
                // 他の設定 (TTLなど) はデフォルト
                ..Default::default()
            })
            .await
            .expect("Failed to create KV store");
        (client, context, store, bucket_name) // context と bucket_name を返す
    }

    // Helper function to create dummy program data
    fn create_dummy_programs(mirakc_url: &str, service_id: i32, count: usize) -> Vec<KurecProgram> {
        (0..count)
            .map(|i| {
                let dt = Utc
                    .timestamp_millis_opt(1678886400000 + i as i64 * 1000)
                    .unwrap();
                KurecProgram {
                    id: 1000 + i as i64,
                    mirakc_url: mirakc_url.to_string(),
                    service_id,
                    network_id: 1,
                    event_id: 5000 + i as i32,
                    channel_name: format!("チャンネル{}", service_id),
                    channel_type: "GR".to_string(),
                    channel: "27".to_string(),
                    name: Some(format!("番組{}", i)),
                    description: Some(format!("説明{}", i)),
                    extended: None,
                    start_at: dt,
                    duration_millis: 1800000,
                    is_free: true,
                    genres: vec!["ジャンルA".to_string()],
                    video_info: None,
                    audio_infos: vec!["ステレオ".to_string()],
                    series_info: None,
                }
            })
            .collect()
    }

    #[tokio::test]
    #[ignore] // NATSサーバーが必要なためデフォルトでは無視
    async fn test_save_and_get_programs() {
        let (_client, context, store, bucket_name) = setup_test_kv().await; // context と bucket_name を受け取る
        let repository = NatsKvProgramRepository::new(store.clone());

        let mirakc_url = "http://test-mirakc:1234";
        let service_id = 101;
        let programs_to_save = create_dummy_programs(mirakc_url, service_id, 3);

        // 保存テスト
        let save_result = repository
            .save_service_programs(mirakc_url, service_id, programs_to_save.clone())
            .await;
        assert!(save_result.is_ok());

        // 取得テスト
        let get_result = repository
            .get_service_programs(mirakc_url, service_id)
            .await;
        assert!(get_result.is_ok());
        let retrieved_programs = get_result.unwrap();
        assert!(retrieved_programs.is_some());
        assert_eq!(retrieved_programs.unwrap(), programs_to_save);

        // 存在しないキーの取得テスト
        let get_none_result = repository.get_service_programs(mirakc_url, 999).await; // 存在しないservice_id
        assert!(get_none_result.is_ok());
        assert!(get_none_result.unwrap().is_none());

        // クリーンアップ (テストバケット削除)
        context // context を使用
            .delete_key_value(&bucket_name) // delete_key_value を使用
            .await
            .expect("Failed to delete test KV bucket");
    }

    #[tokio::test]
    #[ignore] // NATSサーバーが必要なためデフォルトでは無視
    async fn test_save_overwrite() {
        let (_client, context, store, bucket_name) = setup_test_kv().await; // context と bucket_name を受け取る
        let repository = NatsKvProgramRepository::new(store.clone());

        let mirakc_url = "http://overwrite-test:5678";
        let service_id = 202;
        let initial_programs = create_dummy_programs(mirakc_url, service_id, 2);
        let updated_programs = create_dummy_programs(mirakc_url, service_id, 5); // 更新版 (件数変更)

        // 初回保存
        repository
            .save_service_programs(mirakc_url, service_id, initial_programs.clone())
            .await
            .unwrap();
        let retrieved1 = repository
            .get_service_programs(mirakc_url, service_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved1, initial_programs);

        // 上書き保存
        repository
            .save_service_programs(mirakc_url, service_id, updated_programs.clone())
            .await
            .unwrap();
        let retrieved2 = repository
            .get_service_programs(mirakc_url, service_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved2, updated_programs); // 更新後のデータと一致するか

        // クリーンアップ
        context // context を使用
            .delete_key_value(&bucket_name) // delete_key_value を使用
            .await
            .expect("Failed to delete test KV bucket");
    }
}
