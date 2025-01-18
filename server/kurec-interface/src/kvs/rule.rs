use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeilisearchRule {
    pub id: String,
    pub query: String,
    pub filter: String,
    pub is_ignore: bool,
}

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeilisearchRules {
    pub rules: Vec<MeilisearchRule>,
}

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualRule {
    #[typeshare(serialized_as = "number")]
    pub program_id: i64,
    #[typeshare(serialized_as = "number")]
    pub service_id: i64,
    pub is_ignore: bool,
}

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualRules {
    pub rules: Vec<IndividualRule>,
}
