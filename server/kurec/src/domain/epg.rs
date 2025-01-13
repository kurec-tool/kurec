use kurec_adapter::{MeilisearchAdapter, MeilisearchIndex, MirakcAdapter, NatsAdapter, StreamType};
use kurec_interface::{
    EpgProgramsUpdatedMessage, EpgProgramsUpdatedMessageData, MirakcEventMessage, ProgramDocument,
};
use tracing::debug;

pub struct EpgDomain {
    pub mirakc_adapter: MirakcAdapter,
    pub nats_adapter: NatsAdapter,
    pub meilisearch_adapter: MeilisearchAdapter,
}

impl EpgDomain {
    pub fn new(
        mirakc_adapter: MirakcAdapter,
        nats_adapter: NatsAdapter,
        meilisearch_adapter: MeilisearchAdapter,
    ) -> Self {
        Self {
            mirakc_adapter,
            nats_adapter,
            meilisearch_adapter,
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
            .filter_map_stream_async(
                StreamType::SseEpgProgramsUpdated,
                StreamType::EpgUpdated,
                "collector",
                f,
            )
            .await?;
        Ok(())
    }

    pub async fn convert_epg_stream(&self) -> Result<(), anyhow::Error> {
        let f = |data: EpgProgramsUpdatedMessage| async move {
            let documents = data
                .programs
                .iter()
                .map(|p| {
                    kurec_interface::ProgramDocument::from_program_with_service(
                        p.clone(),
                        data.service.clone(),
                    )
                })
                .collect::<Vec<_>>();

            for doc in &documents {
                if let Some(ogp_url) = doc.ogp_url.clone() {
                    let request = kurec_interface::OgpRequestMessage {
                        url: ogp_url.clone(),
                        hash: doc.ogp_url_hash.clone().unwrap(),
                    };
                    let payload = serde_json::to_vec(&request)?;
                    self.nats_adapter
                        .publish_to_stream(StreamType::OgpRequest, payload.into())
                        .await?;
                }
            }

            let program_document_update = kurec_interface::ProgramDocumentsUpdatedMessage {
                tuner_url: data.tuner_url,
                service: data.service,
                programs: documents,
            };

            Ok(Some(program_document_update))
        };
        self.nats_adapter
            .filter_map_stream_async(
                StreamType::EpgUpdated,
                StreamType::EpgConverted,
                "converter",
                f,
            )
            .await?;
        Ok(())
    }

    pub async fn index_epg_stream(&self) -> Result<(), anyhow::Error> {
        let f = |doc: kurec_interface::ProgramDocumentsUpdatedMessage| async move {
            let service_id = doc.service.id;
            let docs = doc.programs.clone();
            debug!(
                "indexing programs for service_id: {} num of documents: {}",
                service_id,
                docs.len()
            );
            self.meilisearch_adapter
                .update_documents(
                    &MeilisearchIndex::Epg,
                    |d: &ProgramDocument| d.service_id == service_id,
                    &docs,
                    Some("program_id"),
                )
                .await?;
            Ok(())
        };
        self.nats_adapter
            .stream_sink_async(StreamType::EpgConverted, "indexer", f)
            .await?;
        Err(anyhow::Error::msg("unreachable?"))
    }
}
