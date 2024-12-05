use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnEpgProgramUpdated {
    pub program_id: u64,
    pub tuner_url: String,
}
