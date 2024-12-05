use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnEpgProgramUpdated {
    pub tuner_url: String,
    pub service_id: u64,
    pub program_ids: Vec<u64>,
}
