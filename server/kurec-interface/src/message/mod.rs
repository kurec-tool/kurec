use mirakc_client::models::{MirakurunProgram, MirakurunService};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EpgProgramsUpdatedMessage {
    pub tuner_url: String,
    pub service: MirakurunService,
    pub programs: Vec<MirakurunProgram>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IndexUpdatedMessage {
    pub tuner_url: String,
    pub service: MirakurunService,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OgpRequestMessage {
    pub url: String,
    pub hash: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum RuleUpdatedMessage {
    RuleUpdated,
    EpgUpdated { tuner_url: String, service_id: i64 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScheduleUpdatedMessage {}
