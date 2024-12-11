use kurec_adapter::{MirakcAdapter, NatsAdapter};
use kurec_proto::MirakcEventMessage;

const SSE_EPG_STREAM: &str = "sse-epg-programs-updated";
const CONVERTED_STREAM: &str = "epg-converted";

pub struct EpgCollector {
    pub mirakc_adapter: MirakcAdapter,
    pub nats_adapter: NatsAdapter,
}

impl EpgCollector {
    pub fn new(mirakc_adapter: MirakcAdapter, nats_adapter: NatsAdapter) -> Self {
        Self {
            mirakc_adapter,
            nats_adapter,
        }
    }

    pub async fn collect_epg_stream(&self) -> Result<(), anyhow::Error> {
        self.nats_adapter
            .filter_map_stream(
                SSE_EPG_STREAM,
                CONVERTED_STREAM,
                |epg: MirakcEventMessage| -> Result<Option<MirakcEventMessage>, anyhow::Error> {
                    dbg!(&epg);
                    Ok(Some(epg))
                },
            )
            .await?;
        Ok(())
    }
}
