# mirakc-infra テスト戦略

## 概要

mirakc-infraのテストは以下の3つのレベルで行います：

1. **単体テスト**: `wiremock`を使用したモックサーバーでのテスト
2. **統合テスト**: レコーディングテスト
3. **オプショナルな手動テスト**: 実際のmirakcサーバーを使用したテスト

## 1. 単体テスト

### 目的
- 各APIエンドポイントの基本的な動作を検証
- CI環境でも実行可能なテスト

### 実装方法
- `wiremock` クレートを使用してモックサーバーを実装
- 各APIエンドポイントのレスポンスを事前に定義
- テスト中にモックサーバーを起動し、MirakcClientの接続先として使用

### 例
```rust
#[tokio::test]
async fn test_get_version() {
    // モックサーバーを起動
    let mock_server = MockServer::start().await;
    
    // モックレスポンスを設定
    Mock::given(method("GET"))
        .and(path("/api/version"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "current": "1.0.0",
                "latest": "1.0.0"
            })))
        .mount(&mock_server)
        .await;
    
    // テスト対象のリポジトリを作成
    let repo = VersionRepositoryImpl::new(&mock_server.uri());
    
    // メソッドを呼び出し
    let version = repo.get_version().await.unwrap();
    
    // 結果を検証
    assert_eq!(version.current, "1.0.0");
    assert_eq!(version.latest, "1.0.0");
}
```

## 2. 統合テスト（レコーディングテスト）

### 目的
- 実際のAPIレスポンスを記録して再生
- CI環境でも実行可能な統合テスト

### 実装方法
- 初回実行時に実際のAPIレスポンスを記録
- 以降の実行では記録したレスポンスを再生
- 環境変数でレコーディングモードを制御

### 例
```rust
async fn record_or_replay(endpoint: &str, record: bool) -> Value {
    let path = format!("tests/fixtures/{}.json", endpoint.replace("/", "_"));
    
    if record || !Path::new(&path).exists() {
        // 実際のAPIを呼び出し
        let mirakc_url = "http://tuner:40772";
        let client = reqwest::Client::new();
        let response = client.get(format!("{}{}", mirakc_url, endpoint))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        
        // レスポンスを保存
        fs::write(&path, serde_json::to_string_pretty(&response).unwrap()).unwrap();
        response
    } else {
        // 記録したレスポンスを読み込み
        let data = fs::read_to_string(&path).unwrap();
        serde_json::from_str(&data).unwrap()
    }
}

#[tokio::test]
async fn test_get_version_recording() {
    // 環境変数でレコーディングモードを制御
    let record = std::env::var("RECORD").is_ok();
    
    // APIレスポンスを取得（または再生）
    let response = record_or_replay("/api/version", record).await;
    
    // レスポンスを検証
    assert!(response["current"].is_string());
    assert!(response["latest"].is_string());
}
```

## 3. オプショナルな手動テスト

### 目的
- 実際の環境での動作を確認
- 開発環境でのみ実行

### 実装方法
- テスト用の設定ファイルで環境変数からmirakcサーバーのURLを取得
- CIでは実行しないようにフラグを設定

### 例
```rust
#[tokio::test]
#[ignore] // CI環境では実行しない
async fn test_with_real_mirakc() {
    // 環境変数からmirakcサーバーのURLを取得
    let mirakc_url = std::env::var("MIRAKC_URL")
        .unwrap_or_else(|_| "http://tuner:40772".to_string());
    
    // テスト対象のリポジトリを作成
    let repo = VersionRepositoryImpl::new(&mirakc_url);
    
    // バージョン情報を取得
    let version = repo.get_version().await.unwrap();
    
    // 結果を検証
    assert!(!version.current.is_empty());
    assert!(!version.latest.is_empty());
}
```

## テスト実行方法

### 単体テスト
```bash
cargo test --package infra-mirakc
```

### レコーディングテスト（記録モード）
```bash
RECORD=1 cargo test --package infra-mirakc -- --test-threads=1
```

### レコーディングテスト（再生モード）
```bash
cargo test --package infra-mirakc -- --test-threads=1
```

### 実際のmirakcサーバーを使用したテスト
```bash
MIRAKC_URL=http://tuner:40772 cargo test --package infra-mirakc -- --ignored
