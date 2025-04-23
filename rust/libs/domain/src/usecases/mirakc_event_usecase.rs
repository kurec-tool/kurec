//! mirakcイベントユースケース
//!
//! このモジュールはmirakcイベントを処理するユースケースを定義します。

// MirakcEventInput をインポート
use crate::events::mirakc_events::*;
use crate::ports::event_sink::EventSink; // 正しいパスからインポート
use crate::ports::repositories::mirakc_event_repository::MirakcEventRepository;
use futures::StreamExt;
use serde::Deserialize; // Deserialize をインポート
use tracing::{debug, info, warn}; // ログレベルを追加

// --- ローカル定義: 各イベントのデータ部分をパースするための一時構造体 ---
// mirakc_event_handler.rs と同じ定義
#[derive(Deserialize)]
struct TunerStatusChangedData {
    #[serde(rename = "tunerIndex")]
    tuner_index: usize,
}
#[derive(Deserialize)]
struct EpgProgramsUpdatedData {
    #[serde(rename = "serviceId")]
    service_id: i64,
}
#[derive(Deserialize)]
struct RecordingStartedData {
    #[serde(rename = "programId")]
    program_id: u64,
}
#[derive(Deserialize)]
struct RecordingStoppedData {
    #[serde(rename = "programId")]
    program_id: u64,
}
#[derive(Deserialize)]
struct RecordingFailedData {
    #[serde(rename = "programId")]
    program_id: u64,
    reason: RecordingFailedReason,
}
#[derive(Deserialize)]
struct RecordingRescheduledData {
    #[serde(rename = "programId")]
    program_id: u64,
}
#[derive(Deserialize)]
struct RecordingRecordSavedData {
    #[serde(rename = "recordId")]
    record_id: String,
    #[serde(rename = "recordingStatus")]
    recording_status: RecordingStatus,
}
#[derive(Deserialize)]
struct RecordingRecordRemovedData {
    #[serde(rename = "recordId")]
    record_id: String,
}
#[derive(Deserialize)]
struct RecordingContentRemovedData {
    #[serde(rename = "recordId")]
    record_id: String,
}
#[derive(Deserialize)]
struct RecordingRecordBrokenData {
    #[serde(rename = "recordId")]
    record_id: String,
    reason: String,
}
#[derive(Deserialize)]
struct OnairProgramChangedData {
    #[serde(rename = "serviceId")]
    service_id: i64,
}

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
            // event_dto -> event_input
            let event_input = tokio::select! {
                event = stream.next() => {
                    match event {
                        Some(input) => input, // dto -> input
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
                    info!("Shutdown signal received, stopping event processing");
                    break;
                }
            };

            // mirakc_url と received_at を取得
            let mirakc_url = event_input.mirakc_url.clone();
            let received_at = event_input.received_at;

            // イベントタイプに応じてパースし、ドメインイベントを組み立てて発行
            // パースエラー時はログに記録して処理を続行
            let publish_result = match event_input.event_type.as_str() {
                "tuner.status-changed" => {
                    match serde_json::from_str::<TunerStatusChangedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = TunerStatusChangedEvent {
                                mirakc_url,
                                tuner_index: parsed_data.tuner_index,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse TunerStatusChangedData");
                            continue;
                        }
                    }
                }
                "epg.programs-updated" => {
                    match serde_json::from_str::<EpgProgramsUpdatedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = EpgProgramsUpdatedEvent {
                                mirakc_url,
                                service_id: parsed_data.service_id,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse EpgProgramsUpdatedData");
                            continue;
                        }
                    }
                }
                "recording.started" => {
                    match serde_json::from_str::<RecordingStartedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = RecordingStartedEvent {
                                mirakc_url,
                                program_id: parsed_data.program_id,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse RecordingStartedData");
                            continue;
                        }
                    }
                }
                "recording.stopped" => {
                    match serde_json::from_str::<RecordingStoppedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = RecordingStoppedEvent {
                                mirakc_url,
                                program_id: parsed_data.program_id,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse RecordingStoppedData");
                            continue;
                        }
                    }
                }
                "recording.failed" => {
                    match serde_json::from_str::<RecordingFailedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = RecordingFailedEvent {
                                mirakc_url,
                                program_id: parsed_data.program_id,
                                reason: parsed_data.reason,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse RecordingFailedData");
                            continue;
                        }
                    }
                }
                "recording.rescheduled" => {
                    match serde_json::from_str::<RecordingRescheduledData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = RecordingRescheduledEvent {
                                mirakc_url,
                                program_id: parsed_data.program_id,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse RecordingRescheduledData");
                            continue;
                        }
                    }
                }
                "recording.record-saved" => {
                    match serde_json::from_str::<RecordingRecordSavedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = RecordingRecordSavedEvent {
                                mirakc_url,
                                record_id: parsed_data.record_id,
                                recording_status: parsed_data.recording_status,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse RecordingRecordSavedData");
                            continue;
                        }
                    }
                }
                "recording.record-removed" => {
                    match serde_json::from_str::<RecordingRecordRemovedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = RecordingRecordRemovedEvent {
                                mirakc_url,
                                record_id: parsed_data.record_id,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse RecordingRecordRemovedData");
                            continue;
                        }
                    }
                }
                "recording.content-removed" => {
                    match serde_json::from_str::<RecordingContentRemovedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = RecordingContentRemovedEvent {
                                mirakc_url,
                                record_id: parsed_data.record_id,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse RecordingContentRemovedData");
                            continue;
                        }
                    }
                }
                "recording.record-broken" => {
                    match serde_json::from_str::<RecordingRecordBrokenData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = RecordingRecordBrokenEvent {
                                mirakc_url,
                                record_id: parsed_data.record_id,
                                reason: parsed_data.reason,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse RecordingRecordBrokenData");
                            continue;
                        }
                    }
                }
                "onair.program-changed" => {
                    match serde_json::from_str::<OnairProgramChangedData>(&event_input.data) {
                        Ok(parsed_data) => {
                            let event = OnairProgramChangedEvent {
                                mirakc_url,
                                service_id: parsed_data.service_id,
                                received_at,
                            };
                            self.sink.publish(event).await
                        }
                        Err(e) => {
                            warn!(error=%e, data=%event_input.data, "Failed to parse OnairProgramChangedData");
                            continue;
                        }
                    }
                }
                _ => {
                    // 未知のイベントタイプはログに記録するだけ
                    debug!("Unknown mirakc event type: {}", event_input.event_type);
                    // 何も発行しないので Ok(()) を返す
                    Ok(())
                }
            };

            // パブリッシュエラーがあればログに記録して処理を続行
            if let Err(e) = publish_result {
                warn!(error = %e, "Failed to publish event");
            }
        }

        Ok(())
    }
}
