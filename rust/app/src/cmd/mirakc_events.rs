//! mirakcイベント処理コマンド (StreamWorker を使わない実装)
//!
//! このモジュールはmirakcイベントを処理するコマンドを提供します。

use anyhow::Result;
use domain::{
    events::mirakc_events::*, // イベント型をインポート
    handlers::mirakc_event_handler::{MirakcEventHandler, MirakcEventSinks}, // 新しいハンドラ
};
use futures::StreamExt;
use infra_jetstream::JsPublisher; // Sink 実装
use infra_mirakc::MirakcSseSource; // Source 実装
use shared_core::{
    error_handling::ClassifyError, // エラー分類用
    event_sink::EventSink,         // EventSink トレイト
    stream_worker::StreamHandler,  // StreamHandler トレイト (ハンドラが実装)
};
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

/// mirakcイベント処理コマンドを実行
pub async fn run_mirakc_events(
    config: &crate::AppConfig,
    mirakc_url: &str,
    shutdown: CancellationToken,
) -> Result<()> {
    info!("Starting mirakc events command with URL: {}", mirakc_url);

    // JetStreamの設定
    let js_ctx = Arc::new(infra_jetstream::connect(&config.nats_url).await?);
    infra_jetstream::setup_all_streams(&js_ctx.js).await?;

    // SSEイベントソースの作成
    let source = MirakcSseSource::new(mirakc_url.to_string());

    // 各イベント型に対応する EventSink (JsPublisher) を作成
    let sinks = MirakcEventSinks {
        tuner_status_changed: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        epg_programs_updated: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        recording_started: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        recording_stopped: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        recording_failed: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        recording_rescheduled: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        recording_record_saved: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        recording_record_removed: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        recording_content_removed: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        recording_record_broken: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
        onair_program_changed: Arc::new(JsPublisher::from_event_type(js_ctx.clone())), // js_ctx.clone() を渡す
    };

    // イベントハンドラの作成
    let handler = MirakcEventHandler::new(sinks);

    // イベント処理ループ
    info!("Starting to process mirakc events...");
    let mut event_stream = source.event_stream().await?;
    let shutdown_token = shutdown.clone();

    loop {
        select! {
            // シャットダウントークンが発火したら終了
            _ = shutdown_token.cancelled() => {
                info!("Shutdown signal received, stopping mirakc event processing.");
                break;
            }
            // イベントを受信したら処理
            maybe_event_dto = event_stream.next() => {
                match maybe_event_dto {
                    Some(event_dto) => {
                        // ハンドラでイベントを処理
                        match handler.handle(event_dto).await {
                            Ok(_) => {
                                // SSEにはAckがないので何もしない
                            }
                            Err(e) => {
                                // エラー処理 (ClassifyError に基づく)
                                error!("Error handling mirakc event: {}", e);
                                match e.error_action() {
                                    shared_core::error_handling::ErrorAction::Retry => {
                                        // SSE ではリトライできないため、エラーログのみ
                                        error!("Retry action requested, but SSE source cannot retry. Ignoring error.");
                                    }
                                    shared_core::error_handling::ErrorAction::Ignore => {
                                        // 無視 (エラーログは既に出力済み)
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        info!("Mirakc event stream ended.");
                        break; // ストリームが終了したらループを抜ける
                    }
                }
            }
        }
    }

    info!("Mirakc events processing stopped.");
    Ok(())
}

// 古い CombinedPublisher と impl_event_sink マクロは削除
