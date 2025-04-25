//! mirakc SSEイベントソースの実装

use anyhow::Result;
use async_trait::async_trait;
use backoff::{backoff::Backoff, ExponentialBackoff};
use bytes::Bytes;
use chrono::Utc;
use domain::events::MirakcEventInput; // MirakcEventInput をインポート
use domain::ports::event_source::EventSource; // domain::ports::event_source からインポート
use eventsource_stream::Eventsource;
use futures::{future, stream::BoxStream, Stream, StreamExt};
// 不要な DTO インポートを削除: use shared_core::dtos::mirakc_event::MirakcEventDto;
use std::time::Duration;

/// mirakc SSEイベントソース
pub struct MirakcSseSource {
    mirakc_url: String,
}

impl MirakcSseSource {
    /// 新しいMirakcSseSourceを作成
    pub fn new(mirakc_url: String) -> Self {
        Self { mirakc_url }
    }

    /// バックオフを使用してmirakcサーバーに接続し、SSEストリームを取得
    async fn get_sse_stream(
        &self,
    ) -> anyhow::Result<impl Stream<Item = Result<Bytes, anyhow::Error>>> {
        let events_url = format!("{}/events", self.mirakc_url);

        let mut backoff = ExponentialBackoff {
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            multiplier: 2.0,
            max_elapsed_time: None, // 無限に再試行
            ..ExponentialBackoff::default()
        };

        loop {
            match reqwest::get(&events_url).await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!("Connected to mirakc events endpoint: {}", events_url);
                    tracing::debug!("Starting to receive SSE events");

                    // StreamExt::map_errを使わずに手動で変換
                    let stream = futures::stream::unfold(resp, |mut resp| async move {
                        match resp.chunk().await {
                            Ok(Some(chunk)) => {
                                tracing::debug!("Received chunk of size: {} bytes", chunk.len());
                                Some((Ok(chunk), resp))
                            }
                            Ok(None) => {
                                tracing::warn!("SSE stream ended unexpectedly. This may cause events to be processed only once. Check if mirakc server is still running.");
                                None
                            }
                            Err(e) => {
                                tracing::error!("Error receiving chunk: {:?}. This may cause events to be processed only once.", e);
                                Some((Err(anyhow::Error::new(e)), resp))
                            }
                        }
                    });

                    return Ok(stream);
                }
                Ok(resp) => {
                    let status = resp.status();
                    tracing::warn!(
                        "Failed to connect to mirakc events endpoint: {}, status: {}",
                        events_url,
                        status
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Error connecting to mirakc events endpoint: {}, error: {:?}",
                        events_url,
                        e
                    );
                }
            }

            // バックオフして再試行
            if let Some(duration) = backoff.next_backoff() {
                tracing::info!("Retrying connection in {:?}...", duration);
                tokio::time::sleep(duration).await;
            } else {
                return Err(anyhow::anyhow!("Max retries exceeded"));
            } // loop の閉じ括弧を追加
        } // loop の閉じ括弧を追加
    }

    // #[async_trait] を削除
    // EventSource<MirakcEventDto> の実装を削除
    // 代わりに、MirakcEventInput を返すストリームを提供するメソッドを追加する

    /// MirakcEventInput のストリームを取得する
    /// EventSource トレイトの実装で使用するためのヘルパーメソッド
    // 返り値の型を MirakcEventInput に変更
    async fn event_stream(&self) -> Result<BoxStream<'static, MirakcEventInput>> {
        let mirakc_url = self.mirakc_url.clone();
        tracing::info!("Starting event stream from mirakc URL: {}", mirakc_url);

        // SSEストリームを取得
        tracing::debug!("Attempting to get SSE stream...");
        let stream = match self.get_sse_stream().await {
            Ok(s) => {
                tracing::info!("Successfully obtained SSE stream");
                s
            }
            Err(e) => {
                tracing::error!("Failed to get SSE stream: {:?}. This will prevent events from being processed.", e);
                return Err(e);
            }
        };

        // SSEストリームを MirakcEventInput ストリームに変換
        tracing::debug!("Converting SSE stream to MirakcEventInput stream"); // Dto -> Input
        let event_stream = stream
            .eventsource()
            .filter_map(move |event_result| {
                let mirakc_url = mirakc_url.clone(); // 各イベント用にURLをクローン
                future::ready(match event_result {
                    Ok(event) => {
                        // MirakcEventInput を作成
                        tracing::info!(
                            event_type = %event.event,
                            "Received SSE event - will be processed and published to JetStream"
                        );
                        // MirakcEventInput を作成
                        Some(MirakcEventInput {
                            mirakc_url,
                            event_type: event.event,
                            data: event.data,
                            received_at: Utc::now(),
                        })
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error receiving SSE event: {:?}. Event will be skipped.",
                            e
                        );
                        None // エラーが発生したイベントはスキップ
                    }
                })
            })
            .boxed();

        tracing::info!("Event stream setup complete. Events will be processed as they arrive.");
        Ok(event_stream)
    }
} // impl MirakcSseSource の閉じ括弧を追加

/// MirakcSseSource に EventSource<MirakcEventInput> トレイトを実装
#[async_trait]
// ジェネリック型を MirakcEventInput に変更
impl EventSource<MirakcEventInput> for MirakcSseSource {
    /// MirakcEventInput のストリームを購読する
    // 返り値の型を MirakcEventInput に変更
    async fn subscribe(
        &self,
    ) -> Result<BoxStream<'static, Result<MirakcEventInput, anyhow::Error>>> {
        // イベントストリームを取得
        let event_stream = self.event_stream().await?;

        // Result<MirakcEventInput, anyhow::Error> を返すストリームに変換
        let result_stream = event_stream.map(|event| Ok(event)).boxed();

        Ok(result_stream)
    }
}
