//! EPG更新ワーカーコマンド
//!
//! このモジュールはEPG更新イベントを処理するコマンドを提供します。

use anyhow::Result;
// use async_trait::async_trait; // 不要になった
use domain::ports::event_source::EventSource; // domain::ports::event_source をインポート
use domain::{
    events::{kurec_events::EpgStoredEvent, mirakc_events::EpgProgramsUpdatedEvent},
    handlers::epg_update_handler::{EpgUpdateHandler, StreamHandler},
    ports::event_sink::EventSink,
};
use futures::StreamExt; // StreamExt をインポート
use std::sync::Arc;
use tokio::select; // select! マクロをインポート
use tokio_util::sync::CancellationToken;
use tracing::{error, info}; // error, info をインポート

// JsEpgNotifier 構造体と実装を削除

/// EPG更新ワーカーを実行 (手動ループ)
pub async fn run_epg_updater(
    source: Arc<dyn EventSource<EpgProgramsUpdatedEvent>>, // EventSource を引数で受け取る
    sink: Arc<dyn EventSink<EpgStoredEvent>>,              // EventSink を引数で受け取る
    shutdown: CancellationToken,
) -> Result<()> {
    info!("Starting EPG updater worker...");

    // EpgUpdateHandler の作成
    let handler = Arc::new(EpgUpdateHandler::new());

    // StreamWorker 関連のコードは完全に削除されていることを確認

    // イベント処理ループ
    let mut event_stream = source.subscribe().await?; // event_stream() -> subscribe() に変更
    let shutdown_token = shutdown.clone();

    loop {
        select! {
            // シャットダウントークンが発火したら終了
            _ = shutdown_token.cancelled() => {
                info!("Shutdown signal received, stopping EPG updater worker.");
                break;
            }
            // イベントを受信したら処理
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        info!(
                            service_id = event.service_id, // data フィールドを削除
                            "Received EpgProgramsUpdatedEvent"
                        );
                        // ハンドラでイベントを処理
                        match handler.handle(event.clone()).await {
                            Ok(Some(stored_event)) => {
                                // StreamWorker がないので、ここで明示的に Sink に発行
                                if let Err(e) = sink.publish(stored_event).await {
                                     error!("Failed to publish EpgStoredEvent: {}", e);
                                     // エラー処理: リトライ or 無視など検討
                                } else {
                                     info!("Successfully processed EPG update and published EpgStoredEvent");
                                }
                            }
                            Ok(None) => {
                                info!("EPG update handled, no event to publish.");
                            }
                            Err(e) => {
                                // エラー処理 (ClassifyError に基づく)
                                error!("Error handling EPG update: {}", e);
                        // エラーログを出力するだけ
                        error!("EPG update error: {}. Continuing...", e);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        error!("Error receiving EPG update event: {}", e);
                        // エラー処理 (ClassifyError に基づく)
                        // エラーログを出力するだけ
                        error!("EPG update event error: {}. Continuing...", e);
                    }
                    None => {
                        error!("EPG update event stream ended unexpectedly. Attempting to reconnect...");
                        // ストリームが終了したら再接続を試みる (EventSource が対応していれば)
                        match source.subscribe().await { // event_stream() -> subscribe() に変更
                            Ok(new_stream) => {
                                info!("Successfully reconnected to EPG update event stream");
                                event_stream = new_stream;
                            }
                            Err(e) => {
                                error!("Failed to reconnect to EPG update event stream: {:?}. Exiting.", e);
                                break; // 再接続に失敗したらループを抜ける
                            }
                        }
                    }
                }
            }
        }
    }

    info!("EPG updater worker stopped gracefully.");
    Ok(())
}
