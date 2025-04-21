//! NATS 接続と関連リソース管理のためのインフラクレート
//!
//! このクレートは NATS サーバーへの接続を確立し、
//! JetStream コンテキストや KV ストアへのアクセスを提供します。

use anyhow::{Context, Result};
use async_nats::{
    self,
    client::Client,
    connect_with_options,
    jetstream,
    jetstream::kv, // kv を jetstream モジュールからインポート
    ConnectOptions,
    // ServerAddr, // 未使用のため削除
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tracing::{debug, info, warn};

#[derive(Error, Debug)]
pub enum NatsInfraError {
    #[error("NATS 接続に失敗しました: {0}")]
    ConnectionFailed(#[from] async_nats::Error), // async_nats::Error をラップ

    #[error("JetStream コンテキストの取得に失敗しました: {0}")]
    JetStreamContextFailed(async_nats::Error),

    // bucket_name を名前で参照するように修正
    #[error("KV ストア '{bucket_name}' の作成/取得に失敗しました: {source}")]
    KvStoreFailed {
        bucket_name: String,
        source: async_nats::Error,
    },

    #[error("その他のエラー: {0}")]
    Other(#[from] anyhow::Error), // anyhow::Error をラップ
}

/// NATS クライアントと関連コンテキストを保持するラッパー構造体。
/// Arc でラップして共有可能にします。
#[derive(Clone)]
pub struct NatsClient {
    client: Client,
    js_context: jetstream::context::Context, // JetStream コンテキストをキャッシュ
}

impl NatsClient {
    /// 新しい NatsClient インスタンスを作成します (内部利用)。
    fn new(client: Client) -> Self {
        let js_context = jetstream::new(client.clone());
        Self { client, js_context }
    }

    /// 接続済みの NATS クライアントを取得します。
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// 接続済みの JetStream コンテキストを取得します。
    pub fn jetstream_context(&self) -> &jetstream::context::Context {
        &self.js_context
    }

    /// 指定された設定で KV ストアを取得または作成します。
    ///
    /// 冪等性を持ちます。ストアが既に存在する場合はそれを返し、
    /// 存在しない場合は作成します。
    pub async fn get_or_create_kv_store(
        &self,
        kv_config: kv::Config,
    ) -> Result<kv::Store, NatsInfraError> {
        let bucket_name = kv_config.bucket.clone();
        debug!(bucket_name = %bucket_name, "KV ストアを取得または作成します...");

        match self.js_context.get_key_value(&bucket_name).await {
            Ok(store) => {
                info!(bucket_name = %bucket_name, "既存の KV ストアを取得しました。");
                // TODO: 設定が異なる場合に update_key_value を呼ぶべきか検討
                Ok(store)
            }
            // ErrorKind::NotFound または "stream not found" の場合に作成を試みる
            Err(err)
                if err.to_string().contains("no key value store named")
                    || err.to_string().contains("stream not found") =>
            {
                info!(bucket_name = %bucket_name, "KV ストアが存在しないか、関連ストリームが見つからないため、新規作成します。");
                self.js_context
                    .create_key_value(kv_config) // create_key_value は Result<Store, Error<CreateKeyValueErrorKind>> を返す
                    .await
                    .map_err(|e| {
                        // CreateKeyValueErrorKind を async_nats::Error に変換 (into() を試す)
                        NatsInfraError::KvStoreFailed {
                            bucket_name: bucket_name.clone(),
                            source: e.into(), // .into() を追加
                        }
                    })
            }
            Err(e) => {
                // その他の予期せぬエラー
                warn!(bucket_name = %bucket_name, error = %e, "KV ストアの取得中に予期せぬエラーが発生しました。");
                Err(NatsInfraError::KvStoreFailed {
                    bucket_name,
                    source: e.into(), // .into() を追加
                })
            }
        }
        // create_key_value は冪等ではないため、get -> create の順で試す
        // match self.js_context.create_key_value(kv_config).await {
        //     Ok(store) => {
        //         info!(bucket_name = %bucket_name, "KV ストアを新規作成しました。");
        //         Ok(store)
        //     }
        //     Err(e) => {
        //         // すでに存在する場合のエラーは無視して取得を試みる
        //         // TODO: async-nats のエラー型をちゃんと確認する
        //         if e.to_string().contains("bucket already exists") {
        //             info!(bucket_name = %bucket_name, "KV ストアは既に存在します。取得を試みます。");
        //             self.js_context.get_key_value(&bucket_name).await.map_err(|e_get| {
        //                 NatsInfraError::KvStoreFailed {
        //                     bucket_name: bucket_name.clone(),
        //                     source: e_get, // get のエラーを返す
        //                 }
        //             })
        //         } else {
        //             warn!(bucket_name = %bucket_name, error = %e, "KV ストアの作成中にエラーが発生しました。");
        //             Err(NatsInfraError::KvStoreFailed { bucket_name, source: e })
        //         }
        //     }
        // }
    }
}

/// 指定された URL で NATS サーバーに接続し、`NatsClient` を返します。
///
/// 接続オプションには、再接続試行などのデフォルト設定が含まれます。
pub async fn connect(nats_url: &str) -> Result<Arc<NatsClient>, NatsInfraError> {
    info!(url = %nats_url, "NATS サーバーへの接続を開始します...");

    // TODO: 設定ファイルから読み込むなど、より柔軟なオプション設定を検討
    let options = ConnectOptions::new()
        .retry_on_initial_connect()
        .connection_timeout(Duration::from_secs(10)) // メソッド名を修正
        .max_reconnects(None) // 無制限に再接続試行
        .reconnect_delay_callback(|attempts| {
            // 再接続試行回数に応じて遅延時間を調整 (例: 指数バックオフ)
            let delay = Duration::from_millis(100 * 2u64.pow(attempts.min(8) as u32)); // 最大約25秒
            debug!(attempts, delay = ?delay, "NATS 再接続試行...");
            delay
        });

    let client = connect_with_options(nats_url, options)
        .await
        .context("NATS 接続オプション付きでの接続に失敗しました")?; // anyhow::Error に変換

    info!(url = %nats_url, "NATS サーバーへの接続が成功しました。");
    Ok(Arc::new(NatsClient::new(client)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use async_nats::jetstream::kv; // kv をインポート
    use testcontainers::{
        core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt,
    };
    use tokio;
    use tracing::debug; // debug をインポート

    // テスト終了時に自動的にコンテナを停止・削除するための構造体
    struct TestNatsContainer {
        _container: ContainerAsync<GenericImage>, // コンテナの所有権を保持
    }

    // Docker が利用可能かチェック
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

    // テスト用の NATS サーバーを起動し、接続 URL とコンテナハンドラを返す
    async fn setup_nats() -> Result<(TestNatsContainer, String)> {
        ensure_docker().await;
        debug!("Starting NATS container for testing...");
        let container = GenericImage::new("nats", "latest")
            .with_exposed_port(4222u16.into())
            .with_wait_for(WaitFor::message_on_stderr("Server is ready"))
            .with_cmd(vec!["--js", "--debug"]) // JetStream を有効化
            .start()
            .await?;
        let host = container.get_host().await?;
        let port = container.get_host_port_ipv4(4222u16).await?;
        let url = format!("nats://{}:{}", host, port); // スキームを nats:// に修正
        debug!(url = %url, "NATS container started.");

        // NATSサーバーが完全に起動するまで少し待機
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok((
            TestNatsContainer {
                _container: container,
            },
            url,
        ))
    }

    // テスト用の NatsClient をセットアップするヘルパー関数
    async fn setup_test_client() -> Result<(TestNatsContainer, Arc<NatsClient>)> {
        let (container_handler, url) = setup_nats().await?;
        let client = connect(&url).await?;
        Ok((container_handler, client))
    }

    #[tokio::test]
    async fn test_connect_success() -> Result<()> {
        let (_container_handler, client) = setup_test_client().await?;
        // flush() を呼び出して接続を確立させる
        client.client().flush().await?;
        // 接続状態を確認
        assert_eq!(
            client.client().connection_state(),
            async_nats::connection::State::Connected
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_jetstream_context() -> Result<()> {
        let (_container_handler, nats_client) = setup_test_client().await?;
        let js_ctx = nats_client.jetstream_context();
        // 簡単な操作を試す (例: アカウント情報取得 - query_account())
        let result = js_ctx.query_account().await;
        assert!(result.is_ok(), "JetStream 操作に失敗: {:?}", result.err());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_create_kv_store_basic() -> Result<()> {
        let (_container_handler, nats_client) = setup_test_client().await?;

        let bucket_name = format!("test_kv_basic_{}", rand::random::<u32>());
        let kv_config = kv::Config {
            bucket: bucket_name.clone(),
            ..Default::default()
        };

        // 1回目の呼び出し (作成されるはず)
        let result1 = nats_client.get_or_create_kv_store(kv_config.clone()).await;
        assert!(
            result1.is_ok(),
            "KVストアの初回作成/取得に失敗: {:?}",
            result1.err()
        );
        let store1 = result1.unwrap();
        // bucket() メソッドは存在しないため、直接比較はしない (作成成功で確認)
        // assert_eq!(store1.bucket(), bucket_name);

        // 簡単な書き込み・読み込みテスト
        let key = "test_key";
        let value = b"test_value";
        // put は Bytes を取るので、&[u8] から .into() で変換
        assert!(store1.put(key, value.as_ref().into()).await.is_ok());
        let entry = store1
            .entry(key)
            .await
            .expect("KV get failed")
            .expect("KV entry not found");
        // entry の結果は Option<Entry> で、Entry.payload は Bytes
        assert_eq!(entry.value.as_ref(), value); // value -> payload

        // 2回目の呼び出し (取得)
        let result2 = nats_client.get_or_create_kv_store(kv_config).await;
        assert!(
            result2.is_ok(),
            "KVストアの2回目の取得に失敗: {:?}",
            result2.err()
        );
        let store2 = result2.unwrap();
        // bucket() メソッドは存在しないため、直接比較はしない (取得成功で確認)
        // assert_eq!(store2.bucket(), bucket_name);

        // ストアが同じものであることを確認 (簡単な操作で)
        let entry_again = store2
            .entry(key)
            .await
            .expect("KV get failed again")
            .expect("KV entry not found again");
        assert_eq!(entry_again.value.as_ref(), value); // value -> payload

        // クリーンアップはコンテナハンドラの Drop で自動的に行われる
        // 必要であれば明示的に削除テストを追加
        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_create_kv_store_already_exists() -> Result<()> {
        let (_container_handler, nats_client) = setup_test_client().await?;
        let js_ctx = nats_client.jetstream_context(); // 事前作成用にコンテキスト取得

        let bucket_name = format!("test_kv_exists_{}", rand::random::<u32>());
        let kv_config = kv::Config {
            bucket: bucket_name.clone(),
            ..Default::default()
        };

        // 事前に KV ストアを作成しておく
        let initial_store = js_ctx
            .create_key_value(kv_config.clone())
            .await
            .expect("テスト用KVストアの事前作成に失敗");
        assert!(initial_store
            // put は Bytes を取るので、&str から .into() で変換
            .put("pre_key", "pre_value".into())
            .await
            .is_ok());

        // get_or_create_kv_store を呼び出す (既存ストアが取得されるはず)
        let result = nats_client.get_or_create_kv_store(kv_config).await;
        assert!(
            result.is_ok(),
            "既存KVストアの取得に失敗: {:?}",
            result.err()
        );
        let store = result.unwrap();
        // bucket() メソッドは存在しないため、直接比較はしない (取得成功で確認)
        // assert_eq!(store.bucket(), bucket_name);

        // 事前に入れた値が読めるか確認
        let entry = store
            .entry("pre_key")
            .await
            .expect("KV get failed")
            .expect("KV entry not found");
        assert_eq!(entry.value.as_ref(), b"pre_value"); // value -> payload

        // クリーンアップはコンテナハンドラの Drop で自動的に行われる
        Ok(())
    }
}
