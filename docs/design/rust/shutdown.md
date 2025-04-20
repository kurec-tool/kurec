# Rust シャットダウン処理のベストプラクティス

## 基本方針

Rustアプリケーション、特に長時間実行されるサービスやワーカーでは、適切なシャットダウン処理が重要です。以下のベストプラクティスに従ってください。

## シャットダウン処理の実装

### 1. シグナルハンドリング

- **CTRL+C (SIGINT)** と **SIGTERM** の両方を捕捉する
- tokioを使用する場合は `tokio::signal::ctrl_c()` を使用
- 複数のシグナルを捕捉する場合は、適切に組み合わせる

```rust
// 基本的なCTRL+C捕捉
tokio::spawn(async move {
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
    println!("Shutting down...");
    shutdown_token.cancel();
});
```

### 2. CancellationTokenの使用

- `tokio_util::sync::CancellationToken` を使用してシャットダウンシグナルを伝播
- トークンは複数の場所で共有可能（`clone()`で複製）
- シャットダウン時に `cancel()` を呼び出す

```rust
// CancellationTokenの作成と共有
let shutdown = CancellationToken::new();
let worker_shutdown = shutdown.clone();

// ワーカーにシャットダウントークンを渡す
let worker = Worker::new().with_shutdown(worker_shutdown);
```

### 3. 非同期ループでのシャットダウン処理

- `tokio::select!` を使用して、タスク処理とシャットダウンシグナルを同時に待機
- シャットダウンシグナル受信時にループを終了

```rust
// 非同期ループでのシャットダウン処理
loop {
    tokio::select! {
        some_event = event_stream.next() => {
            // イベント処理
        },
        _ = shutdown_token.cancelled() => {
            println!("Shutdown signal received");
            break;
        }
    }
}
```

### 4. グレースフルシャットダウン

- 処理中のタスクを適切に完了させる
- リソースを適切に解放する
- 必要に応じてタイムアウトを設定

```rust
// グレースフルシャットダウンの例
async fn shutdown(tasks: Vec<JoinHandle<()>>) {
    // 実行中のタスクを待機（タイムアウト付き）
    let timeout = tokio::time::sleep(Duration::from_secs(10));
    
    tokio::select! {
        _ = futures::future::join_all(tasks) => {
            println!("All tasks completed");
        }
        _ = timeout => {
            println!("Shutdown timed out, forcing exit");
        }
    }
    
    // リソースのクリーンアップ
    cleanup_resources().await;
}
```

### 5. ワーカーパターン

- ワーカーには `with_shutdown()` メソッドを実装
- シャットダウントークンをフィールドとして保持
- `run()` メソッド内でシャットダウンを確認

```rust
pub struct Worker {
    // ...
    shutdown: Option<CancellationToken>,
}

impl Worker {
    pub fn with_shutdown(mut self, token: CancellationToken) -> Self {
        self.shutdown = Some(token);
        self
    }
    
    pub async fn run(&self) -> Result<()> {
        loop {
            // シャットダウンチェック
            if let Some(token) = &self.shutdown {
                if token.is_cancelled() {
                    break;
                }
            }
            
            // 通常の処理
        }
        
        Ok(())
    }
}
```

## テスト

- シャットダウン処理のテストも実装する
- `CancellationToken` を使用してシャットダウンをシミュレート
- タイムアウトを設定して、無限ループを防止

```rust
#[tokio::test]
async fn test_worker_shutdown() {
    let shutdown = CancellationToken::new();
    let worker = Worker::new().with_shutdown(shutdown.clone());
    
    // ワーカーを別スレッドで起動
    let handle = tokio::spawn(async move {
        worker.run().await.unwrap();
    });
    
    // シャットダウンシグナルを送信
    shutdown.cancel();
    
    // ワーカーが終了するのを待機（タイムアウト付き）
    tokio::select! {
        _ = handle => {
            // 正常に終了
        }
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            panic!("Worker did not shut down within timeout");
        }
    }
}
