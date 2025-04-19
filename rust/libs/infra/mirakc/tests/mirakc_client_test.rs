use anyhow::Result;
use serde_json::json;
use wiremock::{matchers::path, Mock, MockServer, ResponseTemplate};

use infra_mirakc::MirakcClient;

#[tokio::test]
async fn test_get_version() -> Result<()> {
    // モックサーバーを起動
    let mock_server = MockServer::start().await;
    
    // モックレスポンスを設定
    Mock::given(path("/api/version"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "current": "1.0.0",
                "latest": "1.0.0"
            })))
        .mount(&mock_server)
        .await;
    
    // テスト対象のクライアントを作成
    let client = MirakcClient::new(&mock_server.uri());
    
    // バージョン情報を取得
    let version = client.get_version().await?;
    
    // 結果を検証
    assert_eq!(version.current, "1.0.0");
    assert_eq!(version.latest, "1.0.0");
    
    Ok(())
}
