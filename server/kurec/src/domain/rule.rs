use std::sync::Arc;

use kurec_adapter::{MeilisearchAdapter, MirakcAdapter, NatsAdapter, StreamType};
use kurec_interface::{ProgramDocument, RuleUpdatedMessage, ScheduleUpdatedMessage};
use tracing::debug;

pub struct RuleDomain {
    pub nats_adapter: NatsAdapter,
    pub meilisearch_adapter: MeilisearchAdapter,
}

impl RuleDomain {
    pub fn new(nats_adapter: NatsAdapter, meilisearch_adapter: MeilisearchAdapter) -> Self {
        Self {
            nats_adapter,
            meilisearch_adapter,
        }
    }

    pub async fn execute_rule(&self) -> Result<(), anyhow::Error> {
        let f = |data: RuleUpdatedMessage| async move {
            debug!("RuleUpdatedMessage: {:?}", data);
            // 一旦RuleUpdatedもEpgUpdatedも同じ処理を行う
            let epgs: Vec<ProgramDocument> = self
                .nats_adapter
                .kv_get_all_decoded(&kurec_adapter::KvsType::EpgConverted)
                .await
                .unwrap();

            Ok(Some(ScheduleUpdatedMessage {}))
        };
        self.nats_adapter
            .filter_map_stream_async(
                StreamType::RuleUpdated,
                StreamType::ScheduleUpdated,
                "converter",
                f,
            )
            .await?;
        Ok(())
    }
}
