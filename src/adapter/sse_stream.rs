use futures::{future, Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EpgProgramUpdatedData {
    service_id: u64,
}
const EVENT_EPG_PROGRAM_UPDATED_HEADER: &[u8] = b"event: epg.programs-updated\ndata: ";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingRecordSavedData {
    pub record_id: String,
    pub recording_status: RecordingStatusUnion,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecordingStatusUnion {
    Enum(RecordingStatusEnum),
    RecordingStatusClass(RecordingStatusClass),
}

#[derive(Serialize, Deserialize)]
pub struct RecordingStatusClass {
    pub failed: Option<Failed>,
}

#[derive(Serialize, Deserialize)]
pub struct Failed {
    pub reason: Option<Reason>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reason {
    pub message: Option<String>,
    #[serde(rename = "type")]
    pub reason_type: Option<Type>,
    pub os_error: Option<f64>,
    pub exit_code: Option<f64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Type {
    #[serde(rename = "io-error")]
    IoError,
    #[serde(rename = "need-rescheduling")]
    NeedRescheduling,
    #[serde(rename = "pipeline-error")]
    PipelineError,
    #[serde(rename = "removed-from-epg")]
    RemovedFromEpg,
    #[serde(rename = "schedule-expired")]
    ScheduleExpired,
    #[serde(rename = "start-recording-failed")]
    StartRecordingFailed,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingStatusEnum {
    Canceled,
    Finished,
    Recording,
}

const EVENT_RECORDING_RECORD_SAVED_HEADER: &[u8] = b"event: recording.record-saved\ndata: ";

pub async fn get_sse_stream(
    tuner_url: &str,
) -> Result<impl Stream<Item = Result<bytes::Bytes, anyhow::Error>>, anyhow::Error> {
    let events_url = format!("{}/events", tuner_url);
    let resp = reqwest::get(events_url).await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to get events stream {}",
            resp.status().as_str()
        ));
    }
    Ok(resp.bytes_stream().map_err(anyhow::Error::new))
}

async fn convert_to_service_id_stream<T>(
    stream: T,
) -> Result<impl Stream<Item = u64>, anyhow::Error>
where
    T: Stream<Item = Result<bytes::Bytes, anyhow::Error>>,
{
    Ok(stream
        .filter(|r| future::ready(r.is_ok()))
        .map(|r| r.unwrap())
        .filter(|b| future::ready(b.starts_with(EVENT_EPG_PROGRAM_UPDATED_HEADER)))
        .map(|b| {
            // 後ろは改行文字なのでJSONパースに影響しないので放置
            serde_json::from_slice::<EpgProgramUpdatedData>(
                &b[EVENT_EPG_PROGRAM_UPDATED_HEADER.len()..],
            )
        })
        .filter(|r| future::ready(r.is_ok()))
        .map(|r| r.unwrap())
        .map(|d| d.service_id))
}

pub async fn get_sse_service_id_stream(
    tuner_url: &str,
) -> Result<impl Stream<Item = u64>, anyhow::Error> {
    let stream = get_sse_stream(tuner_url).await?;
    convert_to_service_id_stream(stream).await
}

async fn convert_to_record_id_stream<T>(
    stream: T,
) -> Result<impl Stream<Item = String>, anyhow::Error>
where
    T: Stream<Item = Result<bytes::Bytes, anyhow::Error>>,
{
    Ok(stream
        .filter(|r| future::ready(r.is_ok()))
        .map(|r| r.unwrap())
        .filter(|b| future::ready(b.starts_with(EVENT_RECORDING_RECORD_SAVED_HEADER)))
        .map(|b| {
            // 後ろは改行文字なのでJSONパースに影響しないので放置
            serde_json::from_slice::<RecordingRecordSavedData>(
                &b[EVENT_RECORDING_RECORD_SAVED_HEADER.len()..],
            )
        })
        .filter(|r| future::ready(r.is_ok()))
        .map(|r| r.unwrap())
        .filter(|d| {
            future::ready(matches!(
                d.recording_status,
                RecordingStatusUnion::Enum(RecordingStatusEnum::Finished)
            ))
        })
        .map(|d| d.record_id))
}

pub async fn get_sse_record_id_stream(
    tuner_url: &str,
) -> Result<impl Stream<Item = String>, anyhow::Error> {
    let stream = get_sse_stream(tuner_url).await?;
    convert_to_record_id_stream(stream).await
}
