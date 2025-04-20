# Rust ストリームワーカー実装ルール

## 基本方針

ストリームワーカーは、イベント駆動型アーキテクチャにおいて「何らかの入力（SSEまたはJetStreamストリーム）を読んでイベントメッセージを処理し、次のストリームにイベントメッセージを送る」というパターンを実装するためのコンポーネントです。以下のルールに従って実装してください。

## 実装ルール

### 1. 責務の分離

- **イベントメッセージの読み取り・書き込み処理**はインフラ層の責務
  - `EventSource` と `EventSink` インターフェースを使用
  - 具体的な実装は `JsSubscriber`、`JsPublisher`、`MirakcSseSource` など
- **イベントに対する処理**はドメイン層の責務
  - `StreamHandler` インターフェースを実装
  - ドメインロジックを含むハンドラクラスを作成

### 2. ハンドラの実装

- ハンドラは `StreamHandler` トレイトを実装する
- 戻り値は `Result<Option<OutputEvent>, Error>` の形式
  - `Ok(Some(output_event))`: 処理成功、出力イベントあり
  - `Ok(None)`: 処理成功、出力イベントなし
  - `Err(error)`: 処理失敗
- 依存関係は `Arc<dyn Interface>` の形でコンストラクタに渡す

```rust
#[async_trait]
impl StreamHandler<InputEvent, OutputEvent, MyError> for MyHandler {
    async fn handle(&self, event: InputEvent) -> Result<Option<OutputEvent>, MyError> {
        // イベント処理ロジック
    }
}
```

### 3. エラーハンドリング

- エラー型は `ClassifyError` トレイトを実装する
- `error_action()` メソッドで `ErrorAction::Retry` または `ErrorAction::Ignore` を返す
- リトライ可能なエラー（インフラ層のエラーなど）は `ErrorAction::Retry` を返す
- リトライ不可能なエラー（バリデーションエラーなど）は `ErrorAction::Ignore` を返す
- JetStreamはAckを返さなければ自動的にリトライするため、この機構を活用する

```rust
impl ClassifyError for MyError {
    fn error_action(&self) -> ErrorAction {
        match self {
            MyError::Repository(_) => ErrorAction::Retry,
            MyError::Validation(_) => ErrorAction::Ignore,
        }
    }
}
```

### 4. ワーカーの構築

- `StreamWorker::new()` を使用してワーカーを構築する
- `source`、`sink`、`handler` を引数に渡す
- 必要に応じて `durable()` または `durable_auto()` でdurable名を設定する
- 必要に応じて `with_middleware()` でミドルウェアを追加する

```rust
let worker = StreamWorker::new(source, sink, handler)
    .durable("my-worker")
    .with_middleware(LoggingMiddleware::new());
```

### 5. シャットダウン処理

- `CancellationToken` を使用してグレースフルシャットダウンを実装する
- `run()` メソッドに `shutdown_token` を渡す
- シャットダウン時は処理中のイベントを完了させてから終了する

```rust
worker.run(shutdown_token).await?;
```

### 6. 簡易実装（マクロ使用）

- 単純なケースでは `#[stream_worker]` マクロを使用できる
- マクロは関数に適用し、`process_event_worker` という名前の関数を生成する

```rust
#[stream_worker]
async fn process_event(event: InputEvent) -> Result<OutputEvent, MyError> {
    // イベント処理ロジック
}
```

## テスト戦略

### 1. ハンドラのテスト

- ハンドラの依存関係をモックして単体テストを実施
- `StreamHandler::handle()` メソッドの戻り値を検証

```rust
#[tokio::test]
async fn test_handler() {
    // モックの作成
    let mock_repository = MockRepository::new();
    let mock_notifier = MockNotifier::new();
    
    // ハンドラの作成
    let handler = MyHandler::new(
        Arc::new(mock_repository),
        Arc::new(mock_notifier),
    );
    
    // テスト対象のイベント
    let event = InputEvent { /* ... */ };
    
    // ハンドラの実行
    let result = handler.handle(event).await;
    
    // 結果の検証
    assert!(result.is_ok());
    if let Ok(Some(output)) = result {
        assert_eq!(output.field, expected_value);
    }
}
```

### 2. ワーカーのテスト

- `JsSubscriber` と `JsPublisher` のモックを作成
- `StreamWorker` を構築して実行
- 出力イベントを検証

```rust
#[tokio::test]
async fn test_worker() {
    // モックの作成
    let mock_source = MockEventSource::new();
    let mock_sink = MockEventSink::new();
    let handler = Arc::new(MyHandler::new(/* ... */));
    
    // ワーカーの構築
    let worker = StreamWorker::new(
        Arc::new(mock_source),
        Arc::new(mock_sink),
        handler,
    );
    
    // シャットダウントークンの作成
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();
    
    // ワーカーを別スレッドで実行
    let handle = tokio::spawn(async move {
        worker.run(shutdown_clone).await.unwrap();
    });
    
    // テスト対象のイベントを送信
    mock_source.send_event(InputEvent { /* ... */ }).await;
    
    // 少し待機
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // シャットダウン
    shutdown.cancel();
    handle.await.unwrap();
    
    // 結果の検証
    let events = mock_sink.get_published_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].field, expected_value);
}
```

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
