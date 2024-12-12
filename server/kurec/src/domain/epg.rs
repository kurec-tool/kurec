use kurec_adapter::{MirakcAdapter, NatsAdapter, StreamType};
use kurec_interface::{
    EpgProgramsUpdatedMessage, EpgProgramsUpdatedMessageData, MirakcEventMessage,
};

pub struct EpgDomain {
    pub mirakc_adapter: MirakcAdapter,
    pub nats_adapter: NatsAdapter,
}

impl EpgDomain {
    pub fn new(mirakc_adapter: MirakcAdapter, nats_adapter: NatsAdapter) -> Self {
        Self {
            mirakc_adapter,
            nats_adapter,
        }
    }

    async fn collect_service_programs(
        mirakc_adapter: MirakcAdapter,
        ev: MirakcEventMessage,
    ) -> Result<Option<EpgProgramsUpdatedMessage>, anyhow::Error> {
        let event_data = serde_json::from_str::<EpgProgramsUpdatedMessageData>(&ev.data)?;
        let service_id = event_data.service_id;

        dbg!(&service_id);

        // service取得
        let service = mirakc_adapter
            .get_service(&ev.tuner_url, service_id)
            .await?;

        let programs = mirakc_adapter
            .get_programs(&ev.tuner_url, service_id)
            .await?;

        let message = EpgProgramsUpdatedMessage {
            tuner_url: ev.tuner_url,
            service,
            programs,
        };

        Ok(Some(message))
    }

    pub async fn collect_epg_stream(&self) -> Result<(), anyhow::Error> {
        let f = |ev| Self::collect_service_programs(self.mirakc_adapter.clone(), ev);
        self.nats_adapter
            .filter_map_stream(
                StreamType::SseEpgProgramsUpdated,
                StreamType::EpgUpdated,
                "collector",
                f,
            )
            .await?;
        Ok(())
    }
}
