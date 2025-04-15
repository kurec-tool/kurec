use mirakc_client::models::{MirakurunProgram, MirakurunService};
use serde::{Deserialize, Serialize};

use crate::StorageType;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatus {
    Recording,
    Finished,
    Canceled,
    Failed,
}

// mirakcのメッセージがそのまま流れてくるのでcamelCase
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingRecordSaved {
    pub record_id: String,
    pub recording_status: RecordingStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingStatusMessage {
    pub tuner_url: String,
    pub record_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncodeResultFile {
    pub name: String,
    pub description: String,
    pub file_size: u64,
    pub storage: StorageType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncodeResultMessage {
    pub tuner_url: String,
    pub encode_results: Vec<EncodeResultFile>,
}
