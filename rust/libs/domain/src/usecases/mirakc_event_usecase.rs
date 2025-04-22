//! mirakcイベントユースケース
//!
//! このモジュールはmirakcイベントを処理するユースケースを定義します。

use crate::events::mirakc_events::*;
use crate::ports::event_sink::EventSink; // 正しいパスからインポート
use crate::ports::repositories::mirakc_event_repository::MirakcEventRepository;
use futures::StreamExt;
// use shared_core::event_sink::EventSink; // 削除

/// mirakcイベントユースケース
pub struct MirakcEventUseCase<R, S>
// P -> S
where
    R: MirakcEventRepository,
    // EventPublisher -> EventSink
    S: EventSink<TunerStatusChangedEvent>
        + EventSink<EpgProgramsUpdatedEvent>
        + EventSink<RecordingStartedEvent>
        + EventSink<RecordingStoppedEvent>
        + EventSink<RecordingFailedEvent>
        + EventSink<RecordingRescheduledEvent>
        + EventSink<RecordingRecordSavedEvent>
        + EventSink<RecordingRecordRemovedEvent>
        + EventSink<RecordingContentRemovedEvent>
        + EventSink<RecordingRecordBrokenEvent>
        + EventSink<OnairProgramChangedEvent>,
{
    repository: R,
    sink: S, // publisher -> sink
    shutdown: Option<tokio_util::sync::CancellationToken>,
}

impl<R, S> MirakcEventUseCase<R, S>
// P -> S
where
    R: MirakcEventRepository,
    // EventPublisher -> EventSink
    S: EventSink<TunerStatusChangedEvent>
        + EventSink<EpgProgramsUpdatedEvent>
        + EventSink<RecordingStartedEvent>
        + EventSink<RecordingStoppedEvent>
        + EventSink<RecordingFailedEvent>
        + EventSink<RecordingRescheduledEvent>
        + EventSink<RecordingRecordSavedEvent>
        + EventSink<RecordingRecordRemovedEvent>
        + EventSink<RecordingContentRemovedEvent>
        + EventSink<RecordingRecordBrokenEvent>
        + EventSink<OnairProgramChangedEvent>,
{
    /// 新しいMirakcEventUseCaseを作成
    pub fn new(repository: R, sink: S) -> Self {
        // publisher -> sink
        Self {
            repository,
            sink, // publisher -> sink
            shutdown: None,
        }
    }

    /// シャットダウントークンを設定
    pub fn with_shutdown(mut self, token: tokio_util::sync::CancellationToken) -> Self {
        self.shutdown = Some(token);
        self
    }

    /// mirakcサーバーからイベントを取得し、適切な型に変換してパブリッシュする
    pub async fn process_events(&self) -> anyhow::Result<()> {
        let mut stream = self.repository.get_event_stream().await?;

        loop {
            // シャットダウントークンが設定されている場合、キャンセルされたかチェック
            if let Some(token) = &self.shutdown {
                if token.is_cancelled() {
                    tracing::info!("Shutdown signal received, stopping event processing");
                    break;
                }
            }

            // イベントを取得（タイムアウト付き）
            let event_dto = tokio::select! {
                event = stream.next() => {
                    match event {
                        Some(dto) => dto,
                        None => break, // ストリームが終了した場合
                    }
                },
                // シャットダウントークンが設定されている場合、キャンセルを待機
                _ = async {
                    if let Some(token) = &self.shutdown {
                        token.cancelled().await;
                    } else {
                        // シャットダウントークンがない場合は永久に待機
                        std::future::pending::<()>().await;
                    }
                } => {
                    tracing::info!("Shutdown signal received, stopping event processing");
                    break;
                }
            };
            match event_dto.event_type.as_str() {
                "tuner.status-changed" => {
                    let data: shared_core::dtos::mirakc_event::TunerStatusChangedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = TunerStatusChangedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "epg.programs-updated" => {
                    let data: shared_core::dtos::mirakc_event::EpgProgramsUpdatedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = EpgProgramsUpdatedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "recording.started" => {
                    let data: shared_core::dtos::mirakc_event::RecordingStartedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = RecordingStartedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "recording.stopped" => {
                    let data: shared_core::dtos::mirakc_event::RecordingStoppedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = RecordingStoppedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "recording.failed" => {
                    let data: shared_core::dtos::mirakc_event::RecordingFailedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = RecordingFailedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "recording.rescheduled" => {
                    let data: shared_core::dtos::mirakc_event::RecordingRescheduledDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = RecordingRescheduledEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "recording.record-saved" => {
                    let data: shared_core::dtos::mirakc_event::RecordingRecordSavedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = RecordingRecordSavedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "recording.record-removed" => {
                    let data: shared_core::dtos::mirakc_event::RecordingRecordRemovedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = RecordingRecordRemovedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "recording.content-removed" => {
                    let data: shared_core::dtos::mirakc_event::RecordingContentRemovedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = RecordingContentRemovedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "recording.record-broken" => {
                    let data: shared_core::dtos::mirakc_event::RecordingRecordBrokenDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = RecordingRecordBrokenEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                "onair.program-changed" => {
                    let data: shared_core::dtos::mirakc_event::OnairProgramChangedDto =
                        serde_json::from_str(&event_dto.data)?;
                    let event = OnairProgramChangedEvent {
                        mirakc_url: event_dto.mirakc_url,
                        data,
                        received_at: event_dto.received_at,
                    };
                    self.sink.publish(event).await?; // publisher -> sink
                }
                _ => {
                    // 未知のイベントタイプはログに記録するだけ
                    tracing::debug!("Unknown mirakc event type: {}", event_dto.event_type);
                }
            }
        }

        Ok(())
    }
}
