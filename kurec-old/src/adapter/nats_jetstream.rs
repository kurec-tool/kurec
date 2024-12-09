use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KvEpgUpdated {
    pub program_id: u64,
    pub service_id: u64,
    pub tuner_url: String,
}
