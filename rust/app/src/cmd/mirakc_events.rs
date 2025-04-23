//! mirakcイベント処理コマンド (StreamWorker を使わない実装)
//!
//! このモジュールはmirakcイベントを処理するコマンドを提供します。

use anyhow::Result; // Context は未使用なので削除
use domain::{
    events::MirakcEventInput, // MirakcEventInput をインポート
    handlers::mirakc_event_handler::{MirakcEventHandler, MirakcEventSinks},
    ports::event_source::EventSource, // EventSource をインポート
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

/// mirakcイベント処理コマンドを実行
pub async fn run_mirakc_events(
    // _config: &crate::AppConfig // AppConfig型が見つからないため削除
    _mirakc_url: &str, // mirakc_url は source 作成にしか使わないので不要
    // 引数の型を MirakcEventInput に変更
    source: Arc<dyn EventSource<MirakcEventInput>>, // Source を引数で受け取る
    sinks: MirakcEventSinks,
    shutdown: CancellationToken,
) -> Result<()> {
    info!("Starting mirakc events command..."); // URL表示を削除

    // SSEイベントソースは引数で受け取るため削除
    // let source = MirakcSseSource::new(mirakc_url.to_string());

    // Sink 構築ロジックは削除済み

    // イベントハンドラの作成 (引数で受け取った sinks を使用)
    let handler = MirakcEventHandler::new(sinks);

    // イベント処理ループ
    info!("Starting to process mirakc events...");
    let mut event_stream = source.subscribe().await?; // event_stream() -> subscribe()
    let shutdown_token = shutdown.clone();

    loop {
        select! {
            // シャットダウントークンが発火したら終了
            _ = shutdown_token.cancelled() => {
                info!("Shutdown signal received, stopping mirakc event processing.");
                break;
            }
            // イベントを受信したら処理
            // maybe_event_dto -> maybe_event_input
            maybe_event_input = event_stream.next() => {
                match maybe_event_input {
                    // event_dto -> event_input
                    Some(Ok(event_input)) => {
                        // イベント受信のログを追加
                        info!(
                            event_type = %event_input.event_type,
                            received_at = %event_input.received_at,
                            "Received mirakc event"
                        );

                        // ハンドラでイベントを処理 (clone は不要になる場合があるが、念のため残す)
                        match handler.handle(event_input.clone()).await {
                            Ok(_) => {
                                // 処理成功のログを追加
                                debug!("Successfully handled mirakc event");
                            }
                            Err(e) => {
                                // エラー処理 (ClassifyError に基づく)
                                error!("Error handling mirakc event: {}", e);
                                // エラーログを出力するだけ
                                error!("Error handling mirakc event: {}. Continuing...", e);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        error!("Error receiving mirakc event: {}", e);
                        // エラーログを出力するだけ
                        error!("Error receiving mirakc event: {}. Continuing...", e);
                    }
                    None => {
                        error!("Mirakc event stream ended unexpectedly. Attempting to reconnect...");
                        // ストリームが終了したら再接続を試みる
                        match source.subscribe().await { // event_stream() -> subscribe()
                            Ok(new_stream) => {
                                info!("Successfully reconnected to mirakc event stream");
                                event_stream = new_stream;
                            }
                            Err(e) => {
                                error!("Failed to reconnect to mirakc event stream: {:?}. Exiting.", e);
                                break; // 再接続に失敗したらループを抜ける
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Mirakc events processing stopped.");
    Ok(())
}

// 古い CombinedPublisher と impl_event_sink マクロは削除
