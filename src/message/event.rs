use serde::{Deserialize, Serialize};
use tracing::error;

const EVENT_TUNER_STATUS_CHANGED: &str = "tuner.status-changed";
const EVENT_EPG_PROGRAM_UPDATED: &str = "epg.programs-updated";
const EVENT_RECORDING_STARTED: &str = "recording.started";
const EVENT_RECORDING_STOPPED: &str = "recording.stopped";
const EVENT_RECORDING_FAILED: &str = "recording.failed";
const EVENT_RECORDING_RESCHEDULED: &str = "recording.rescheduled";
const EVENT_RECORDING_RECORD_SAVED: &str = "recording.record-saved";
const EVENT_RECORDING_RECORD_REMOVED: &str = "recording.record-removed";
const EVENT_RECORDING_CONTENT_REMOVED: &str = "recording.content-removed";
const EVENT_RECORDING_RECORD_BROKEN: &str = "recording.record-broken";
// const EVENT_TIMESHIFT_TIMELINE: &str = "timeshift.timeline";
// const EVENT_TIMESHIFT_STARTED: &str = "timeshift.started";
// const EVENT_TIMESHIFT_STOPPED: &str = "timeshift.stopped";
// const EVENT_TIMESHIFT_RECORD_STARTED: &str = "timeshift.record.started";
// const EVENT_TIMESHIFT_RECORD_UPDATED: &str = "timeshift.record.updated";
// const EVENT_TIMESHIFT_RECORD_ENDED: &str = "timeshift.record.ended";
const EVENT_ONAIR_PROGRAM_CHANGED: &str = "onair.program-changed";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TunerStatusChangedData {
    pub tuner_index: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EpgProgramsUpdatedData {
    pub service_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStartedData {
    pub program_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStoppedData {
    pub program_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RecordingFailedReason {
    #[serde(rename_all = "camelCase")]
    StartRecordingFailed {
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    IoError {
        message: String,
        os_error: Option<i32>,
    },
    #[serde(rename_all = "camelCase")]
    PipelineError {
        exit_code: i32,
    },
    NeedRescheduling,
    ScheduleExpired,
    RemovedFromEpg,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordingFailedData {
    pub program_id: u64,
    pub reason: RecordingFailedReason,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordingRescheduledData {
    pub program_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordingRecordSavedData {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordingRecordRemovedData {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordingContentRemovedData {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordingRecordBrokenData {
    pub id: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OnAirProgramChanged {
    pub service_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MirakcEventData {
    EpgProgramsUpdated {
        service_id: u64,
    },
    TunerStatusChanged {
        tuner_index: usize,
    },
    RecordingStarted {
        program_id: u64,
    },
    RecordingStopped {
        program_id: u64,
    },
    RecordingFailed {
        program_id: u64,
        reason: RecordingFailedReason,
    },
    RecordingRescheduled {
        program_id: u64,
    },
    RecordingRecordSaved {
        id: String,
    },
    RecordingRecordRemoved {
        id: String,
    },
    RecordingContentRemoved {
        id: String,
    },
    RecordingRecordBroken {
        id: String,
        reason: String,
    },
    OnAirProgramChanged {
        service_id: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MirakcEvent {
    pub event: String,
    pub data: MirakcEventData,
}

pub fn parse_event(s: &[u8]) -> Option<MirakcEvent> {
    let s = String::from_utf8_lossy(s);
    let (event, data) = s.split_once('\n')?;

    if event.len() < 7 || data.len() < 6 {
        // "\n\n\n": no-event
        return None;
    }
    let event = event[7..].trim();
    let data = data[6..].trim();

    if event.starts_with("timeshift") {
        // TODO: timeshift対応
        return None;
    }

    match event {
        EVENT_TUNER_STATUS_CHANGED => {
            let parsed = serde_json::from_str::<TunerStatusChangedData>(data);
            let tuner_index = parsed.ok()?.tuner_index;
            Some(MirakcEvent {
                event: EVENT_TUNER_STATUS_CHANGED.to_string(),
                data: MirakcEventData::TunerStatusChanged { tuner_index },
            })
        }
        EVENT_EPG_PROGRAM_UPDATED => {
            let parsed = serde_json::from_str::<EpgProgramsUpdatedData>(data);
            let service_id = parsed.ok()?.service_id;
            Some(MirakcEvent {
                event: EVENT_EPG_PROGRAM_UPDATED.to_string(),
                data: MirakcEventData::EpgProgramsUpdated { service_id },
            })
        }
        EVENT_RECORDING_STARTED => {
            let parsed = serde_json::from_str::<RecordingStartedData>(data);
            let program_id = parsed.ok()?.program_id;
            Some(MirakcEvent {
                event: EVENT_RECORDING_STARTED.to_string(),
                data: MirakcEventData::RecordingStarted { program_id },
            })
        }
        EVENT_RECORDING_STOPPED => {
            let parsed = serde_json::from_str::<RecordingStoppedData>(data);
            let program_id = parsed.ok()?.program_id;
            Some(MirakcEvent {
                event: EVENT_RECORDING_STOPPED.to_string(),
                data: MirakcEventData::RecordingStopped { program_id },
            })
        }
        EVENT_RECORDING_FAILED => {
            let parsed = serde_json::from_str::<RecordingFailedData>(data).ok()?;
            let program_id = parsed.program_id;
            let reason = parsed.reason;
            Some(MirakcEvent {
                event: EVENT_RECORDING_FAILED.to_string(),
                data: MirakcEventData::RecordingFailed { program_id, reason },
            })
        }
        EVENT_RECORDING_RESCHEDULED => {
            let parsed = serde_json::from_str::<RecordingRescheduledData>(data);
            let program_id = parsed.ok()?.program_id;
            Some(MirakcEvent {
                event: EVENT_RECORDING_RESCHEDULED.to_string(),
                data: MirakcEventData::RecordingRescheduled { program_id },
            })
        }
        EVENT_RECORDING_RECORD_SAVED => {
            let parsed = serde_json::from_str::<RecordingRecordSavedData>(data);
            let id = parsed.ok()?.id;
            Some(MirakcEvent {
                event: EVENT_RECORDING_RECORD_SAVED.to_string(),
                data: MirakcEventData::RecordingRecordSaved { id },
            })
        }
        EVENT_RECORDING_RECORD_REMOVED => {
            let parsed = serde_json::from_str::<RecordingRecordRemovedData>(data);
            let id = parsed.ok()?.id;
            Some(MirakcEvent {
                event: EVENT_RECORDING_RECORD_REMOVED.to_string(),
                data: MirakcEventData::RecordingRecordRemoved { id },
            })
        }
        EVENT_RECORDING_CONTENT_REMOVED => {
            let parsed = serde_json::from_str::<RecordingContentRemovedData>(data);
            let id = parsed.ok()?.id;
            Some(MirakcEvent {
                event: EVENT_RECORDING_CONTENT_REMOVED.to_string(),
                data: MirakcEventData::RecordingContentRemoved { id },
            })
        }
        EVENT_RECORDING_RECORD_BROKEN => {
            let parsed = serde_json::from_str::<RecordingRecordBrokenData>(data).ok()?;
            let id = parsed.id;
            let reason = parsed.reason;
            Some(MirakcEvent {
                event: EVENT_RECORDING_RECORD_BROKEN.to_string(),
                data: MirakcEventData::RecordingRecordBroken { id, reason },
            })
        }
        EVENT_ONAIR_PROGRAM_CHANGED => {
            let parsed = serde_json::from_str::<OnAirProgramChanged>(data);
            let service_id = parsed.ok()?.service_id;
            Some(MirakcEvent {
                event: EVENT_ONAIR_PROGRAM_CHANGED.to_string(),
                data: MirakcEventData::OnAirProgramChanged { service_id },
            })
        }
        _ => {
            error!("unknown event: {}", event);
            None
        }
    }
}
