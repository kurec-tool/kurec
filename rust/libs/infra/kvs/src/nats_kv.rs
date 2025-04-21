use anyhow::{Context, Result}; // anyhow::Result を使用
use async_nats::jetstream::kv::{Config as KvConfig, Store}; // Config をインポート
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc; // Arc をインポート
use tracing::{debug, error, info, instrument}; // info を追加

// infra_nats クレートの NatsClient をインポート
use infra_nats::NatsClient;

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
    /// このリポジトリは "kurec_epg" KV バケットを使用します。
    /// バケットが存在しない場合は作成されます。
    ///
    /// # Arguments
    ///
    /// * `nats_client` - 接続済みの `NatsClient`
    pub async fn new(nats_client: Arc<NatsClient>) -> Result<Self> {
        // "kurec_epg" バケットの設定を定義
        // TODO: バケット名を定数化または設定から取得する
        let kv_config = KvConfig {
            bucket: "kurec_epg".to_string(),
            // 必要に応じて他の設定 (TTL, history など) を追加
            ..Default::default()
        };

        info!(bucket_name = %kv_config.bucket, "EPG 用 KV ストアを取得または作成します...");
        // NatsClient から JetStream Context を取得し、KV ストアを作成/取得
        let js_ctx = nats_client.jetstream_context();
        let store = match js_ctx.get_key_value(&kv_config.bucket).await {
            Ok(store) => {
                info!(bucket_name = %kv_config.bucket, "既存の EPG 用 KV ストアを取得しました。");
                // TODO: 設定が異なる場合に update_key_value を呼ぶべきか検討
                store
            }
            Err(err)
                if err.to_string().contains("no key value store named")
                    || err.to_string().contains("stream not found") =>
            {
                info!(bucket_name = %kv_config.bucket, "EPG 用 KV ストアが存在しないか、関連ストリームが見つからないため、新規作成します。");
                js_ctx
                    .create_key_value(kv_config)
                    .await
                    .context("EPG 用 KV ストアの作成に失敗しました")?
            }
            Err(e) => {
                // その他の予期せぬエラー
                return Err(anyhow::Error::new(e).context(format!(
                    "EPG 用 KV ストア '{}' の取得中にエラーが発生しました",
                    kv_config.bucket
                )));
            }
        };

        Ok(Self { store })
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
// --- 単体テスト ---
#[cfg(test)]
mod tests {
    use super::*;
    // use async_nats::connect; // infra_nats::connect を使うので不要
    use chrono::{TimeZone, Utc};
    use domain::models::epg::KurecProgram;
    use futures::StreamExt;
    use infra_nats::connect as nats_connect; // infra_nats::connect をインポート
    use testcontainers::{
        core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt,
    };

    // テスト終了時に自動的にバケットを削除するための構造体
    struct TestBucketCleaner {
        context: Option<async_nats::jetstream::Context>,
        bucket_name: String,
    }

    impl Drop for TestBucketCleaner {
        fn drop(&mut self) {
            if let Some(context) = self.context.take() {
                // バケット名をクローンして所有権を移動
                let bucket_name = self.bucket_name.clone();

                // 非同期処理を実行するためのタスクを作成
                // 注意: これはテスト終了時に実行されるため、結果を待機しない
                eprintln!("Cleaning up test bucket: {}", bucket_name);

                // async-natsのAPIを使用してバケットを削除
                tokio::spawn(async move {
                    match context.delete_key_value(&bucket_name).await {
                        Ok(_) => {
                            eprintln!("Successfully cleaned up test bucket: {}", bucket_name);
                        }
                        Err(e) => {
                            eprintln!("Failed to clean up test bucket {}: {}", bucket_name, e);
                        }
                    }
                });
            }
        }
    }

    async fn ensure_docker() {
        for _ in 0..20 {
            if std::process::Command::new("docker")
                .arg("info")
                .output()
                .is_ok()
            {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        panic!("Docker daemon not ready");
    }

    async fn setup_nats() -> anyhow::Result<(ContainerAsync<GenericImage>, String)> {
        ensure_docker().await;
        // ---- Spin‑up test JetStream -------------------------------------------
        let container = GenericImage::new("nats", "latest")
            .with_exposed_port(4222u16.into())
            .with_wait_for(WaitFor::message_on_stderr("Server is ready"))
            .with_cmd(vec!["--js", "--debug"])
            .start()
            .await?;
        let host = container.get_host().await?;
        let port = container.get_host_port_ipv4(4222u16).await?;
        let url = format!("{}:{}", host, port);

        // NATSサーバーが完全に起動するまで少し待機
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok((container, url))
    }

    // Helper function to create a test NATS client and KV store
    async fn setup_test_kv() -> anyhow::Result<(
        ContainerAsync<GenericImage>,
        Arc<NatsClient>,
        Store,
        TestBucketCleaner,
    )> {
        let (container, url) = setup_nats().await?;

        // infra_nats::connect を使用して NatsClient を取得
        let nats_client = nats_connect(&url)
            .await
            .context("テスト用 NATS クライアントの接続に失敗")?;
        let context = nats_client.jetstream_context().clone(); // クリーナー用にコンテキストをクローン
        let bucket_name = format!("test_kurec_epg_{}", rand::random::<u32>());

        debug!("Creating test bucket: {}", bucket_name);

        // KVストアの設定 (テスト用)
        let kv_config = async_nats::jetstream::kv::Config {
            bucket: bucket_name.clone(),
            history: 1,
            max_value_size: 1024 * 1024, // 1MB
            ..Default::default()
        };

        // NatsClient から JetStream Context を取得し、テスト用 KV ストアを作成
        let js_ctx = nats_client.jetstream_context();
        let store = match js_ctx.get_key_value(&kv_config.bucket).await {
            Ok(store) => store, // 既存ストアを返す
            Err(err)
                if err.to_string().contains("no key value store named")
                    || err.to_string().contains("stream not found") =>
            {
                // ストアが存在しないか、関連ストリームが見つからない場合は作成
                js_ctx
                    .create_key_value(kv_config)
                    .await
                    .context("テスト用 KV ストアの作成に失敗")?
            }
            Err(e) => {
                // その他のエラー
                return Err(anyhow::Error::new(e).context(format!(
                    "テスト用 KV ストア '{}' の取得中にエラーが発生しました",
                    kv_config.bucket
                )));
            }
        };

        // クリーナーを作成 (JetStream Context を渡す)
        let cleaner = TestBucketCleaner {
            context: Some(context), // クローンしたコンテキストを渡す
            bucket_name,
        };

        // 少し待機してKVストアが完全に初期化されるのを待つ
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // NatsClient と Store の両方を返す (テスト内容に応じて使い分ける)
        Ok((container, nats_client.clone(), store, cleaner))
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
    async fn test_repository_new() -> anyhow::Result<()> {
        let (_container, nats_client, _store, _cleaner) = setup_test_kv().await?;

        // NatsKvProgramRepository::new をテスト
        let repository_result = NatsKvProgramRepository::new(nats_client.clone()).await;
        assert!(
            repository_result.is_ok(),
            "Repository 作成に失敗: {:?}",
            repository_result.err()
        );

        // 作成されたリポジトリが動作するか簡単な確認 (バケット名など)
        let repository = repository_result.unwrap();
        assert_eq!(repository.store.name, "kurec_epg"); // ハードコードされたバケット名を確認

        Ok(())
    }

    #[tokio::test]
    async fn test_save_and_get_programs() -> anyhow::Result<()> {
        // NatsClient を取得するが、リポジトリ作成には store を直接使う (new のテストではないため)
        let (_container, nats_client, store, _cleaner) = setup_test_kv().await?;
        // let repository = NatsKvProgramRepository::new(nats_client.clone()).await?; // new を使う場合
        let repository = NatsKvProgramRepository {
            store: store.clone(),
        }; // store を直接設定

        let mirakc_url = "http://test-mirakc:1234";
        let service_id = 101i64;
        let programs_to_save = create_dummy_programs(mirakc_url, service_id, 3);

        // 保存テスト
        repository
            .save_service_programs(mirakc_url, service_id, programs_to_save.clone())
            .await?;

        // 取得テスト
        let retrieved_programs = repository
            .get_service_programs(mirakc_url, service_id)
            .await?;
        assert!(retrieved_programs.is_some());
        assert_eq!(retrieved_programs.unwrap(), programs_to_save);

        // 存在しないキーの取得テスト
        let get_none_result = repository.get_service_programs(mirakc_url, 999i64).await?;
        assert!(get_none_result.is_none());

        // クリーンアップは _cleaner のドロップ時に自動的に行われる
        Ok(())
    }

    #[tokio::test]
    async fn test_save_overwrite() -> anyhow::Result<()> {
        let (_container, _nats_client, store, _cleaner) = setup_test_kv().await?;
        // let repository = NatsKvProgramRepository::new(_nats_client.clone()).await?; // new を使う場合
        let repository = NatsKvProgramRepository {
            store: store.clone(),
        }; // store を直接設定

        let mirakc_url = "http://overwrite-test:5678";
        let service_id = 202i64;
        let initial_programs = create_dummy_programs(mirakc_url, service_id, 2);
        let updated_programs = create_dummy_programs(mirakc_url, service_id, 5); // 更新版 (件数変更)

        // 初回保存
        repository
            .save_service_programs(mirakc_url, service_id, initial_programs.clone())
            .await?;
        let retrieved1 = repository
            .get_service_programs(mirakc_url, service_id)
            .await?
            .unwrap();
        assert_eq!(retrieved1, initial_programs);

        // 上書き保存
        repository
            .save_service_programs(mirakc_url, service_id, updated_programs.clone())
            .await?;
        let retrieved2 = repository
            .get_service_programs(mirakc_url, service_id)
            .await?
            .unwrap();
        assert_eq!(retrieved2, updated_programs); // 更新後のデータと一致するか

        // クリーンアップは _cleaner のドロップ時に自動的に行われる
        Ok(())
    }
}
