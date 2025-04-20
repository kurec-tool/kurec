//! mirakcイベントリポジトリの実装
//!
//! このモジュールはmirakcイベントリポジトリの実装を提供します。

use async_trait::async_trait;
use backoff::{backoff::Backoff, ExponentialBackoff};
use bytes::Bytes;
use chrono::Utc;
use domain::ports::repositories::mirakc_event_repository::MirakcEventRepository;
use eventsource_stream::Eventsource;
use futures::{future, Stream, StreamExt};
use shared_core::dtos::mirakc_event::MirakcEventDto;
use std::time::Duration;

/// mirakcイベントリポジトリの実装
pub struct MirakcEventRepositoryImpl {
    mirakc_url: String,
}

impl MirakcEventRepositoryImpl {
    /// 新しいMirakcEventRepositoryImplを作成
    pub fn new(mirakc_url: String) -> Self {
        Self { mirakc_url }
    }

    /// バックオフを使用してmirakcサーバーに接続
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

                    // StreamExt::map_errを使わずに手動で変換
                    let stream = futures::stream::unfold(resp, |mut resp| async move {
                        match resp.chunk().await {
                            Ok(Some(chunk)) => Some((Ok(chunk), resp)),
                            Ok(None) => None,
                            Err(e) => Some((Err(anyhow::Error::new(e)), resp)),
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
            }
        }
    }
}

#[async_trait]
impl MirakcEventRepository for MirakcEventRepositoryImpl {
    async fn get_event_stream(
        &self,
    ) -> anyhow::Result<futures::stream::BoxStream<'static, MirakcEventDto>> {
        let mirakc_url = self.mirakc_url.clone();

        // 初回接続
        let stream = self.get_sse_stream().await?;

        // SSEストリームをMirakcEventDtoストリームに変換
        let event_stream = stream
            .eventsource()
            .filter_map(move |event| {
                future::ready(event.ok().map(|e| MirakcEventDto {
                    mirakc_url: mirakc_url.clone(),
                    event_type: e.event,
                    data: e.data,
                    received_at: Utc::now(),
                }))
            })
            .boxed();

        Ok(event_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_event_stream() {
        // モックサーバーをセットアップ
        let mock_server = MockServer::start().await;

        // SSEエンドポイントをモック
        Mock::given(method("GET"))
            .and(path("/events"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("event: tuner.status-changed\ndata: {\"tunerIndex\":0}\n\n"),
            )
            .mount(&mock_server)
            .await;

        let repository = MirakcEventRepositoryImpl::new(mock_server.uri());
        let mut stream = repository.get_event_stream().await.unwrap();

        // イベントを受信できることを確認
        let event = stream.next().await.unwrap();
        assert_eq!(event.event_type, "tuner.status-changed");
        assert_eq!(event.mirakc_url, mock_server.uri());

        // データが正しいことを確認
        let data: shared_core::dtos::mirakc_event::TunerStatusChangedDto =
            serde_json::from_str(&event.data).unwrap();
        assert_eq!(data.tuner_index, 0);
    }
}
