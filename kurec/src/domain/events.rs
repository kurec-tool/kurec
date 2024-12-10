use crate::adapter::{MirakcAdapter, NatsAdapter};

#[derive(Clone, Debug)]
pub struct EventsDomain {
    mirakc_adapter: MirakcAdapter,
    nats_adapter: NatsAdapter,
}

impl EventsDomain {
    pub fn new(mirakc_adapter: MirakcAdapter, nats_adapter: NatsAdapter) -> Self {
        Self {
            mirakc_adapter,
            nats_adapter,
        }
    }

    pub async fn copy_events_to_jetstream(&self) -> Result<(), anyhow::Error> {
        if let Ok(stream) = self.mirakc_adapter.get_events_stream().await {
            self.nats_adapter.copy_events_to_jetstream(stream).await?;
        }

        Ok(())
    }
}
