use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MirakcEventMessage {
    pub tuner_url: String,
    pub event: String,
    pub data: String,
}
