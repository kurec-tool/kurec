use anyhow::Result;
use infra_jetstream as jetstream;
use std::env;
use tokio::signal;
use tokio_util::sync::CancellationToken;

mod workers;

/// 環境変数NATS_URLからNATS接続URLを取得する
/// 環境変数が設定されていない場合はデフォルト値を返す
fn get_nats_url() -> String {
    env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    // JetStreamに接続
    let nats_url = get_nats_url();
    let js_ctx = std::sync::Arc::new(jetstream::connect(&nats_url).await?);

    // ストリームを設定
    jetstream::setup_all_streams(&js_ctx.js).await?;

    // シャットダウントークンを作成
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // Ctrl+Cでシャットダウン
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("Shutting down...");
        shutdown_clone.cancel();
    });

    // ワーカーを起動
    println!("Starting EPG worker...");
    let js_ctx_clone = js_ctx.clone();
    let shutdown_worker = shutdown.clone();
    let _epg_worker_handle = tokio::spawn(async move {
        if let Err(e) = workers::process_epg_event_worker(&js_ctx_clone, shutdown_worker).await {
            eprintln!("EPG worker error: {}", e);
        }
    });

    // シャットダウンを待機
    shutdown.cancelled().await;
    println!("Shutdown complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_nats_url_default() {
        // 環境変数をクリア
        env::remove_var("NATS_URL");

        // デフォルト値が返されることを確認
        assert_eq!(get_nats_url(), "nats://localhost:4222");
    }

    #[test]
    fn test_get_nats_url_custom() {
        // 環境変数を設定
        env::set_var("NATS_URL", "nats://example.com:4222");

        // 設定した値が返されることを確認
        assert_eq!(get_nats_url(), "nats://example.com:4222");

        // テスト後に環境変数をクリア
        env::remove_var("NATS_URL");
    }
}
