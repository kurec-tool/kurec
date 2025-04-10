use futures::StreamExt;

use kurec_adapter::{MirakcEventsAdapter, NatsAdapter, StreamType};
use kurec_interface::MirakcEventMessage;
use tracing::{debug, info};

#[derive(Clone, Debug)]
pub struct EventsDomain {
    mirakc_adapter: Option<MirakcEventsAdapter>,
    nats_adapter: NatsAdapter,
}

impl EventsDomain {
    pub fn new(mirakc_adapter: Option<MirakcEventsAdapter>, nats_adapter: NatsAdapter) -> Self {
        Self {
            mirakc_adapter,
            nats_adapter,
        }
    }

    pub async fn copy_events_to_jetstream(&self) -> Result<(), anyhow::Error> {
        if let Ok(mut stream) = self
            .mirakc_adapter
            .clone()
            .unwrap()
            .get_events_stream()
            .await
        {
            while let Some(ev) = stream.next().await {
                tracing::debug!("event: {:?}", ev);
                let v = serde_json::to_vec(&ev)?;
                self.nats_adapter
                    .publish_to_stream_by_event_name(&ev.event, v.into())
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn split_records_saved(&self) -> Result<(), anyhow::Error> {
        let f = |msg: MirakcEventMessage| async move {
            debug!("received msg data:{}", msg.data);
            let data: kurec_interface::RecordingRecordSaved = serde_json::from_str(&msg.data)?;
            info!(
                "splitting records_saved for record_id: {} status: {:?}",
                data.record_id, data.recording_status
            );
            let stream = match data.recording_status {
                kurec_interface::RecordingStatus::Recording => StreamType::RecordRecording,
                kurec_interface::RecordingStatus::Finished => StreamType::RecordFinishied,
                kurec_interface::RecordingStatus::Canceled => StreamType::RecordCanceled,
                kurec_interface::RecordingStatus::Failed => StreamType::RecordFailed,
            };
            self.nats_adapter
                .publish_to_stream(stream, data.record_id.clone().into())
                .await?;
            Ok(())
        };
        self.nats_adapter
            .stream_sink_async(StreamType::SseRecordingRecordSaved, "splitter", f)
            .await?;
        Ok(())
    }
}
