use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeilisearchRule {
    pub query: String,
    pub filter: String,
}

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MeilisearchRules {
    pub rules: Vec<MeilisearchRule>,
}
