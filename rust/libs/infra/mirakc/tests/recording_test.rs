use anyhow::Result;
use reqwest;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use infra_mirakc::VersionRepositoryImpl;
use shared_core::repositories::version_repository::VersionRepository;

/// APIレスポンスを記録または再生する関数
async fn record_or_replay(endpoint: &str, record: bool) -> Value {
    let path = format!("tests/fixtures/{}.json", endpoint.replace("/", "_"));
    let full_path = format!("rust/libs/infra/mirakc/{}", path);

    if record || !Path::new(&full_path).exists() {
        // 実際のAPIを呼び出し
        let mirakc_url =
            std::env::var("MIRAKC_URL").unwrap_or_else(|_| "http://tuner:40772".to_string());
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}{}", mirakc_url, endpoint))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();

        // レスポンスを保存
        // 親ディレクトリが存在することを確認
        if let Some(parent) = Path::new(&full_path).parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, serde_json::to_string_pretty(&response).unwrap()).unwrap();
        response
    } else if Path::new(&full_path).exists() {
        // 記録したレスポンスを読み込み
        let data = fs::read_to_string(&full_path).unwrap();
        serde_json::from_str(&data).unwrap()
    } else {
        // ファイルが存在しない場合はデフォルト値を返す
        json!({
            "current": "1.0.0",
            "latest": "1.0.0"
        })
    }
}

#[tokio::test]
async fn test_get_version_recording() -> Result<()> {
    // 環境変数でレコーディングモードを制御
    let record = std::env::var("RECORD").is_ok();

    // APIレスポンスを取得（または再生）
    let response = record_or_replay("/api/version", record).await;

    // レスポンスを検証
    assert!(response["current"].is_string());
    assert!(response["latest"].is_string());

    Ok(())
}

#[tokio::test]
#[ignore] // CI環境では実行しない
async fn test_with_real_mirakc() -> Result<()> {
    // 環境変数からmirakcサーバーのURLを取得
    let mirakc_url =
        std::env::var("MIRAKC_URL").unwrap_or_else(|_| "http://tuner:40772".to_string());

    // テスト対象のリポジトリを作成
    let repo = VersionRepositoryImpl::new(&mirakc_url);

    // バージョン情報を取得
    let version = repo.get_version().await?;

    // 結果を検証
    assert!(!version.current.is_empty());
    assert!(!version.latest.is_empty());

    Ok(())
}
