use futures::StreamExt;

use kurec_adapter::{MirakcEventsAdapter, NatsAdapter, StreamType};

#[derive(Clone, Debug)]
pub struct EventsDomain {
    mirakc_adapter: MirakcEventsAdapter,
    nats_adapter: NatsAdapter,
}

impl EventsDomain {
    pub fn new(mirakc_adapter: MirakcEventsAdapter, nats_adapter: NatsAdapter) -> Self {
        Self {
            mirakc_adapter,
            nats_adapter,
        }
    }

    pub async fn copy_events_to_jetstream(&self) -> Result<(), anyhow::Error> {
        if let Ok(mut stream) = self.mirakc_adapter.get_events_stream().await {
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
}
