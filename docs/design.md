# システム設計概要

以下は 全体アーキテクチャと主要コンポーネントのまとめです。

---

## 🏛 アーキテクチャレイヤー

以下の表は、各レイヤーの依存関係を示しています。

| レイヤー         | 依存先                                         |
|-----------------|-----------------------------------------------|
| **shared-macros** | なし                                          |
| **shared-core**   | なし                                          |
| **domain**        | shared-core, shared-macros (マクロ生成のため)   |
| **infra**         | shared-core, shared-macros                    |
| **app (workers)** | domain, infra                                 |

---

## ⚙️ shared-core（コア機能）

```
shared/core/
├─ src/
│  ├── error_handling.rs       ← `pub trait ClassifyError`, `ErrorAction`
│  ├── event_metadata.rs       ← `pub trait Event`
│  ├── event_publisher.rs      ← `pub trait EventPublisher`
│  ├── event_subscriber.rs     ← `pub trait EventSubscriber`, `AckHandle`
│  ├── streams.rs              ← `StreamConfig`, ストリーム設定の登録・取得
│  ├── worker.rs               ← `WorkerBuilder`, `Middleware`, `Handler`
│  └── stream_worker.rs        ← `StreamWorker`, `StreamMiddleware`, `StreamHandler`
```

- **`Event`**: 全ての `#[event]` 型が実装するトレイト（`stream_name`と`event_name`を提供）
- **`streams`**: ストリーム設定の登録・取得機能
- **`EventPublisher`／`EventSubscriber`**: 入出力の抽象ポート
- **`ClassifyError`／`ErrorAction`**: エラー分類と処理方法の決定
- **`WorkerBuilder`／`StreamWorker`**: ワーカーの構築と実行

---

## 🛠 shared-macros（コード生成）

```
shared/macros/  (proc‑macro = true)
├─ src/lib.rs
│  ├─ #[event(stream=…)]      → Event トレイト実装
│  ├─ define_streams!{...}    → ストリーム設定の登録
│  └─ #[worker(...)]          → WorkerDef 登録＋属性パース
├─ src/define_streams.rs      ← ストリーム定義マクロの実装
└─ src/stream_worker.rs       ← ワーカー定義マクロの実装
```

- **`#[event]`**: イベント型に`Event`トレイトを実装
- **`define_streams!`**: ストリーム設定を一元管理し、自動登録
- **`#[worker]`**: ワーカー定義のメタ情報を登録

---

## 📦 infra（JetStream 実装例）

```
infra/jetstream/
├─ src/lib.rs
│    ├─ connect(nats_url) → `JetStreamCtx`
│    └─ setup_all_streams(js) → ストリーム設定を列挙 & idempotent 作成
├─ src/js_publisher.rs      ← `EventPublisher for JsPublisher`
└─ src/js_subscriber.rs     ← `EventSubscriber for JsSubscriber`
```

- **JetStream** に特化した Publisher/Subscriber
- `define_streams!`マクロで定義されたストリームを自動プロビジョニング
- イベント型から自動的にdurable nameを生成

---

## 📦 app (workers)

```
app/
└─ src/
   ├─ main.rs
   └─ workers/
       ├─ epg_worker.rs
       └─ discord_worker.rs
```

- 各ワーカーは `WorkerBuilder::new(sub, pub, handler)`
  - `.with_middleware(...)`
  - `.durable_auto()`
  - `.run(shutdown_token)`
- **ミドルウェア**
  - ロギング／メトリクス／エラー分類(retry/ack)をプラグイン式に挟める
- **CLI**
  - `clap` を使用したコマンドライン引数の解析
  - サブコマンドで起動するワーカーを選択可能
  - 例: `kurec-app epg` でEPGワーカーを起動

---

## 🔄 エラーハンドリング & ミドルウェア

1. **ClassifyError** トレイト → `error_action(): ErrorAction`
   - `ErrorAction::Retry` → 再試行（nack）
   - `ErrorAction::Ignore` → 無視（ack）
2. **ミドルウェア層** でError分類 → retry/ack 決定
3. **DLQ** は専用パイプライン or 管理UIで手動再投入

## 🔄 ストリームワーカー

ストリームワーカーは、入力イベントを処理して出力イベントを生成するコンポーネントです。詳細は `docs/stream_worker.md` を参照してください。

```
┌─────────────┐    ┌───────────────┐    ┌─────────────┐
│ EventSource │───>│ StreamHandler │───>│ EventSink   │
└─────────────┘    └───────────────┘    └─────────────┘
                          │
                          │
                   ┌──────┴──────┐
                   │ Middleware  │
                   └─────────────┘
```

1. **EventSource**: 入力イベントのソース（例: JetStream, SSE）
2. **EventSink**: 出力イベントの送信先（例: JetStream）
3. **StreamHandler**: イベント処理ロジック
4. **StreamMiddleware**: 処理の前後に挟む共通処理
5. **StreamWorker**: 上記コンポーネントを組み合わせて実行するワーカー

ストリームワーカーの実装方法は以下の通りです：

```rust
// ワーカーの構築
let worker = StreamWorker::new(source, sink, handler)
    .durable("my-worker")
    .with_middleware(LoggingMiddleware::new());

// ワーカーの実行
worker.run(shutdown_token).await?;
```

---

## 🚀 開発フロー

開発フローの各段階でテストコードを作成し、テストを実行することで手戻りを防ぎます

1. **マクロ定義** (`shared-macros`) を確立
2. **ポート定義** (`shared-core/ports`) を整理
3. **ドメインユースケース** を書く
4. **infra 実装** を差し込む
5. **app ワーカー** を `WorkerBuilder` ベースで組み立て


### JetStream のテスト

  - trybuild (マクロ)
  - in‑process / testcontainers (JetStream E2E)
  - in‑memory 実装 (単体)

---

## 📊 DTOとドメインモデル

### DTOの配置と役割

```
shared/core/src/dtos/         ← 共通DTOの定義
├─ version_dto.rs             ← 例: VersionDto

infra/{実装名}/src/dtos/      ← 実装固有DTOの定義
├─ version_dto.rs             ← 例: 外部APIのレスポンス型

domain/src/dtos/              ← ドメイン層で使用するDTOの定義
├─ version_dto.rs             ← 例: ドメイン層で使用するDTO
```

- **共通DTO (`shared/core/src/dtos/`)**: 
  - 複数のレイヤーで共有されるDTO
  - リポジトリインターフェースの入出力として使用
  - 例: `VersionDto`

- **実装固有DTO (`infra/{実装名}/src/dtos/`)**: 
  - 特定のインフラ実装に固有のDTO
  - 外部APIのレスポンス型など
  - 例: `MirakcVersionResponse`

- **ドメイン層DTO (`domain/src/dtos/`)**: 
  - ドメイン層で使用するDTO
  - ユースケースの入出力として使用
  - 例: `VersionDto`

### ドメインモデルとDTOの変換

- **リポジトリ実装内での変換**:
  - 外部APIのレスポンス → 共通DTO
  - 例: `MirakcVersionResponse` → `VersionDto`

- **ユースケース内での変換**:
  - 共通DTO → ドメインモデル
  - 例: `VersionDto` → `Version`

## 🏗 ユースケースの作成方法

### ユースケースの構造

```
domain/src/usecases/
├─ mod.rs                     ← ユースケースモジュールの公開
├─ version_usecase.rs         ← 例: VersionUseCase
```

### ユースケースの実装パターン

```rust
// ユースケースの基本構造
pub struct VersionUseCase<R: VersionRepository> {
    repository: R,
}

impl<R: VersionRepository> VersionUseCase<R> {
    // コンストラクタ
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    // ユースケースメソッド
    pub async fn get_version_status(&self) -> Result<(Version, VersionStatus)> {
        // 1. リポジトリからデータを取得
        let version = self.repository.get_version().await?;
        
        // 2. ドメインロジックを適用
        let status = version.version_status()?;
        
        // 3. 結果を返却
        Ok((version, status))
    }
}
```

### ユースケースのテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // モックリポジトリ
    struct MockVersionRepository {
        version: Arc<Mutex<Version>>,
    }
    
    #[async_trait]
    impl VersionRepository for MockVersionRepository {
        async fn get_version(&self) -> Result<Version> {
            let version = self.version.lock().unwrap().clone();
            Ok(version)
        }
    }
    
    #[tokio::test]
    async fn test_get_version_status_up_to_date() {
        // 1. モックリポジトリを準備
        let repo = MockVersionRepository::new("1.0.0", "1.0.0");
        
        // 2. ユースケースを作成
        let usecase = VersionUseCase::new(repo);
        
        // 3. ユースケースを実行
        let (version, status) = usecase.get_version_status().await.unwrap();
        
        // 4. 結果を検証
        assert_eq!(status, VersionStatus::UpToDate);
    }
}
```

## 🔌 リポジトリの実装方法

### リポジトリインターフェース

```rust
// domain/src/ports/repositories/version_repository.rs
#[async_trait]
pub trait VersionRepository: Send + Sync + 'static {
    async fn get_version(&self) -> Result<Version>;
}
```

### リポジトリ実装

```rust
// infra/mirakc/src/repositories/domain_version_repository.rs
pub struct DomainVersionRepositoryImpl {
    client: MirakcClient,
}

#[async_trait]
impl VersionRepository for DomainVersionRepositoryImpl {
    async fn get_version(&self) -> Result<Version> {
        // 1. 外部APIからデータを取得
        let mirakc_version = self.client.get_version().await?;
        
        // 2. ドメインモデルに変換
        Ok(Version {
            current: mirakc_version.current,
            latest: mirakc_version.latest,
        })
    }
}
```

### リポジトリ実装のテスト

```rust
#[tokio::test]
async fn test_get_version() {
    // 1. モックサーバーを準備
    let mock_server = MockServer::start().await;
    
    // 2. モックレスポンスを設定
    Mock::given(method("GET"))
        .and(path("/api/version"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "current": "1.0.0",
                "latest": "1.0.0"
            })))
        .mount(&mock_server)
        .await;
    
    // 3. リポジトリを作成
    let repo = DomainVersionRepositoryImpl::new(&mock_server.uri());
    
    // 4. リポジトリを実行
    let version = repo.get_version().await.unwrap();
    
    // 5. 結果を検証
    assert_eq!(version.current, "1.0.0");
    assert_eq!(version.latest, "1.0.0");
}
```

---
