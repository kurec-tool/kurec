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
    fn generate_key(mirakc_url: &str, service_id: i64) -> String {
        // i32 -> i64
        // URLに含まれる可能性のある特殊文字をエスケープまたは置換する必要があるか検討
        // NATSのキーとして安全な形式にする
        // URLからスキーム (http:// など) を削除し、ホスト部分のみを使用
        // URLからスキーム (http:// など) を削除し、ホスト部分のみを使用
        // また、特殊文字（:や/など）をアンダースコアに置換
        let host = mirakc_url
            .replace("http://", "")
            .replace("https://", "")
            .replace(":", "_")
            .replace("/", "_")
            .trim_end_matches('_')
            .to_string();
        format!("epg_{}_svc_{}", host, service_id)
    }
}

#[async_trait]
impl KurecProgramRepository for NatsKvProgramRepository {
    #[instrument(skip(self, programs), fields(key = %Self::generate_key(mirakc_url, service_id), num_programs = programs.len()))]
    async fn save_service_programs(
        &self,
        mirakc_url: &str,
        service_id: i64, // i32 -> i64
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
                // エラーログを強化
                error!(key = %key, error = %e, "Failed to save programs to NATS KV");
                Err(e).context(format!("NATS KV put operation failed for key '{}'", key))
                // エラーコンテキストにキーを含める
            }
        }
    }

    #[instrument(skip(self), fields(key = %Self::generate_key(mirakc_url, service_id)))]
    async fn get_service_programs(
        &self,
        mirakc_url: &str,
        service_id: i64, // i32 -> i64
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

    // テスト終了時に自動的にバケットを削除するための構造体
    struct TestBucketCleaner {
        context: Option<async_nats::jetstream::Context>,
        bucket_name: String,
    }

    impl Drop for TestBucketCleaner {
        fn drop(&mut self) {
            if let Some(_context) = self.context.take() {
                // バケット名をクローンして所有権を移動
                let bucket_name = self.bucket_name.clone();

                // 非同期処理を実行するためのタスクを作成
                // 注意: これはテスト終了時に実行されるため、結果を待機しない
                // 代わりに、テスト終了時にバケットを削除するコマンドを実行する
                eprintln!("Cleaning up test bucket: {}", bucket_name);

                // 別プロセスでNATSコマンドを実行してバケットを削除
                // これはテスト環境でのみ使用されるため、実運用コードには影響しない
                std::thread::spawn(move || {
                    // nats CLI コマンドを使用してバケットを削除
                    let output = std::process::Command::new("nats")
                        .args(["kv", "rm", "--force", &bucket_name])
                        .output();

                    match output {
                        Ok(output) => {
                            if output.status.success() {
                                eprintln!("Successfully cleaned up test bucket: {}", bucket_name);
                            } else {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                eprintln!(
                                    "Failed to clean up test bucket {}: {}",
                                    bucket_name, stderr
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to execute cleanup command for bucket {}: {}",
                                bucket_name, e
                            );
                        }
                    }
                });
            }
        }
    }

    // Helper function to create a test NATS client and KV store
    // Note: This requires a running NATS server. For true unit tests,
    // consider mocking the Store trait. For integration tests, use testcontainers.
    // 戻り値に cleaner を追加
    async fn setup_test_kv() -> (async_nats::Client, Store, TestBucketCleaner) {
        // 環境変数などからNATS URLを取得 (テスト用に変更可能にする)
        let nats_url =
            std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
        let client = connect(&nats_url).await.expect("Failed to connect to NATS");
        let context = async_nats::jetstream::new(client.clone());
        let bucket_name = format!("test_kurec_epg_{}", rand::random::<u32>()); // バケット名を保存

        debug!("Creating test bucket: {}", bucket_name);

        let store = context
            .create_key_value(async_nats::jetstream::kv::Config {
                // パスはOK
                bucket: bucket_name.clone(), // 保存したバケット名を使用
                // 他の設定 (TTLなど) はデフォルト
                ..Default::default()
            })
            .await
            .expect("Failed to create KV store");

        // クリーナーを作成
        let cleaner = TestBucketCleaner {
            context: Some(context),
            bucket_name,
        };

        (client, store, cleaner) // クリーナーを返す
    }

    // Helper function to create dummy program data
    fn create_dummy_programs(mirakc_url: &str, service_id: i64, count: usize) -> Vec<KurecProgram> {
        // i32 -> i64
        (0..count)
            .map(|i| {
                let dt = Utc
                    .timestamp_millis_opt(1678886400000 + i as i64 * 1000)
                    .unwrap();
                KurecProgram {
                    id: 1000 + i as i64,
                    mirakc_url: mirakc_url.to_string(),
                    service_id,
                    network_id: 1i64,
                    event_id: 5000i64 + i as i64,
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
    // #[ignore] // NATSサーバーが必要なためデフォルトでは無視 <- コメントアウト解除
    async fn test_save_and_get_programs() {
        let (_client, store, _cleaner) = setup_test_kv().await; // クリーナーを受け取る
        let repository = NatsKvProgramRepository::new(store.clone());

        let mirakc_url = "http://test-mirakc:1234";
        let service_id = 101i64;
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
        let get_none_result = repository.get_service_programs(mirakc_url, 999i64).await; // 存在しないservice_id
        assert!(get_none_result.is_ok());
        assert!(get_none_result.unwrap().is_none());

        // クリーンアップは _cleaner のドロップ時に自動的に行われる
    }

    #[tokio::test]
    // #[ignore] // NATSサーバーが必要なためデフォルトでは無視 <- コメントアウト解除
    async fn test_save_overwrite() {
        let (_client, store, _cleaner) = setup_test_kv().await; // クリーナーを受け取る
        let repository = NatsKvProgramRepository::new(store.clone());

        let mirakc_url = "http://overwrite-test:5678";
        let service_id = 202i64;
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

        // クリーンアップは _cleaner のドロップ時に自動的に行われる
    }
}
