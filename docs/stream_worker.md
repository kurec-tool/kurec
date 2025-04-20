# ストリームワーカーの設計と実装ガイド

このドキュメントでは、イベント駆動型アーキテクチャにおけるストリームワーカーの設計と実装について説明します。

## 概要

ストリームワーカーは、入力イベントを処理して出力イベントを生成するコンポーネントです。このシステムでは、以下のような特徴を持つストリームワーカーを実装しています：

- **型安全性**: コンパイル時に入力イベント型と出力イベント型の整合性を確認
- **エラーハンドリング**: エラーの分類と適切な処理（リトライ/無視）
- **ミドルウェア**: 処理の前後に共通処理を挟むことができる
- **シャットダウン処理**: グレースフルシャットダウンをサポート
- **柔軟性**: 様々なイベントソースとイベントシンクに対応

## アーキテクチャ

ストリームワーカーは以下のコンポーネントで構成されています：

1. **EventSource**: 入力イベントのソース（例: JetStream, SSE）
2. **EventSink**: 出力イベントの送信先（例: JetStream）
3. **StreamHandler**: イベント処理ロジック
4. **StreamMiddleware**: 処理の前後に挟む共通処理
5. **StreamWorker**: 上記コンポーネントを組み合わせて実行するワーカー

### コンポーネント間の関係

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

## 実装方法

### 1. ハンドラの実装

ハンドラは `StreamHandler` トレイトを実装します。このトレイトは入力イベントを処理して、出力イベントを生成するメソッドを持ちます。

```rust
#[async_trait]
impl StreamHandler<InputEvent, OutputEvent, MyError> for MyHandler {
    async fn handle(&self, event: InputEvent) -> Result<Option<OutputEvent>, MyError> {
        // イベント処理ロジック
        // 成功時は Ok(Some(output_event)) または Ok(None)
        // 失敗時は Err(error)
    }
}
```

ハンドラの戻り値は `Result<Option<OutputEvent>, MyError>` です：

- `Ok(Some(output_event))`: 処理成功、出力イベントあり
- `Ok(None)`: 処理成功、出力イベントなし
- `Err(error)`: 処理失敗

### 2. エラー型の実装

エラー型は `ClassifyError` トレイトを実装します。このトレイトはエラーの分類と処理方法を決定するメソッドを持ちます。

```rust
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Repository error: {0}")]
    Repository(#[from] anyhow::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl ClassifyError for MyError {
    fn error_action(&self) -> ErrorAction {
        match self {
            MyError::Repository(_) => ErrorAction::Retry, // リトライ可能なエラー
            MyError::Serialization(_) => ErrorAction::Ignore, // リトライ不可能なエラー
        }
    }
}
```

### 3. ワーカーの構築と実行

ワーカーは `StreamWorker` を使って構築します。

```rust
// EventSource の作成
let source: Arc<dyn EventSource<InputEvent>> = 
    Arc::new(JsSubscriber::from_event_type(js_ctx.clone()));

// EventSink の作成
let sink: Arc<dyn EventSink<OutputEvent>> = 
    Arc::new(JsPublisher::from_event_type(js_ctx.clone()));

// StreamHandler の作成
let handler = Arc::new(MyHandler::new(...));

// StreamWorker の構築
let worker = StreamWorker::new(source, sink, handler)
    .durable("my-worker") // durable名を設定
    .with_middleware(LoggingMiddleware::new()); // ミドルウェアを追加（オプション）

// ワーカーの実行
worker.run(shutdown_token).await?;
```

### 4. マクロを使った簡易実装

単純なケースでは、`#[stream_worker]` マクロを使って簡単にワーカーを実装できます。

```rust
#[stream_worker]
async fn process_event(event: InputEvent) -> Result<OutputEvent, MyError> {
    // イベント処理ロジック
    // 成功時は Ok(output_event)
    // 失敗時は Err(error)
}
```

このマクロは以下の関数を生成します：

```rust
async fn process_event_worker(
    js_ctx: &infra_jetstream::JetStreamCtx,
    shutdown: tokio_util::sync::CancellationToken
) -> anyhow::Result<()> {
    // StreamWorker の構築と実行
}
```

## 実装例

### 例1: JetStream → JetStream ワーカー

```rust
// EPG更新ワーカー
#[stream_worker]
async fn process_epg_update(event: EpgUpdateEvent) -> Result<RecordingScheduleEvent, EpgProcessError> {
    // 番組情報のバリデーション
    if event.start_time >= event.end_time {
        return Err(EpgProcessError::InvalidProgram("Invalid time range".to_string()));
    }

    // 録画予約イベントを生成
    let recording_event = RecordingScheduleEvent {
        program_id: event.program_id,
        title: event.title,
        start_time: event.start_time,
        end_time: event.end_time,
        channel: event.channel,
    };

    Ok(recording_event)
}
```

### 例2: 複雑なハンドラの実装

```rust
pub struct EpgUpdateHandler {
    program_repository: Arc<dyn KurecProgramRepository>,
    epg_notifier: Arc<dyn EpgNotifier>,
    mirakc_client_factory: Arc<dyn MirakcClientFactory>,
}

#[async_trait]
impl StreamHandler<EpgProgramsUpdatedEvent, EpgStoredEvent, EpgUpdateError> for EpgUpdateHandler {
    async fn handle(
        &self,
        event: EpgProgramsUpdatedEvent,
    ) -> Result<Option<EpgStoredEvent>, EpgUpdateError> {
        // MirakcClientを作成
        let client = self.mirakc_client_factory.create_client();
        
        // サービス情報を取得
        let service = client.get_service(&event.mirakc_url, event.data.service_id).await?;
        
        // プログラム一覧を取得
        let programs = client.get_programs_of_service(&event.mirakc_url, event.data.service_id).await?;
        
        // プログラムをKurecProgramに変換
        let kurec_programs = programs.into_iter()
            .map(|p| convert_to_kurec_program(p, &service))
            .collect::<Vec<_>>();
        
        // KVSに保存
        self.program_repository.save_service_programs(
            &event.mirakc_url,
            event.data.service_id,
            kurec_programs,
        ).await?;
        
        // 通知イベントの作成
        let stored_event = EpgStoredEvent {
            mirakc_url: event.mirakc_url.clone(),
            service_id: event.data.service_id,
        };
        
        // 通知
        self.epg_notifier.notify_epg_stored(stored_event.clone()).await?;
        
        Ok(Some(stored_event))
    }
}
```

## ベストプラクティス

1. **責務の分離**:
   - イベントメッセージの読み取り・書き込み処理はインフラ層の責務
   - イベントに対する処理の主なものはドメイン層の責務

2. **エラーハンドリング**:
   - ドメイン層でのエラーは通常リトライ不可能
   - インフラ層のエラーは通常リトライ可能
   - エラーの種類に応じて適切な `ErrorAction` を返す

3. **テスト容易性**:
   - ハンドラは純粋な関数として実装し、依存をインジェクションする
   - モックを使ってハンドラをテストする

4. **シャットダウン処理**:
   - `CancellationToken` を使ってグレースフルシャットダウンを実装する
   - 処理中のイベントは完了させてから終了する

5. **ミドルウェア**:
   - ロギング、メトリクス、エラーハンドリングなどの共通処理はミドルウェアとして実装する
   - ミドルウェアは再利用可能なコンポーネントとして設計する

## 注意点

1. **メモリリーク**:
   - `Arc` を使ったサイクリック参照に注意する
   - 長時間実行されるワーカーではメモリ使用量をモニタリングする

2. **パフォーマンス**:
   - 重い処理はブロッキングしないようにする
   - 必要に応じてバッチ処理を検討する

3. **エラー伝播**:
   - エラーは適切に伝播させ、ログに記録する
   - エラーの原因を特定しやすくするために、コンテキスト情報を付加する

4. **冪等性**:
   - イベント処理は冪等であるべき（同じイベントを複数回処理しても結果が変わらない）
   - 特にリトライ可能なエラーの場合、同じイベントが複数回処理される可能性がある
