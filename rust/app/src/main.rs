use anyhow::Result;
use shared_infra_jetstream as jetstream;
use tokio::signal;
use tokio_util::sync::CancellationToken;

mod workers;

#[tokio::main]
async fn main() -> Result<()> {
    // JetStreamに接続
    let js_ctx = jetstream::connect("nats://localhost:4222").await?;

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
    let epg_worker_handle = tokio::spawn(workers::run_process_epg_event(
        &js_ctx,
        shutdown.clone(),
    ));

    // シャットダウンを待機
    shutdown.cancelled().await;
    println!("Shutdown complete");

    Ok(())
}
