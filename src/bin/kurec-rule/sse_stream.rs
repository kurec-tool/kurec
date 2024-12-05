use futures::{future, Stream, StreamExt, TryStreamExt};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EpgProgramUpdatedData {
    service_id: u64,
}
const EVENT_EPG_PROGRAM_UPDATED_HEADER: &[u8] = b"event: epg.programs-updated\ndata: ";

pub async fn convert_to_service_id_stream<T>(
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

pub async fn get_sse_stream(
    events_url: &str,
) -> Result<impl Stream<Item = Result<bytes::Bytes, anyhow::Error>>, anyhow::Error> {
    let resp = reqwest::get(events_url).await?;
    Ok(resp.bytes_stream().map_err(anyhow::Error::new))
}

pub async fn get_sse_service_id_stream(
    tuner_url: &str,
) -> Result<impl Stream<Item = u64>, anyhow::Error> {
    let events_url = format!("{}/events", tuner_url);
    let stream = get_sse_stream(&events_url).await?;
    convert_to_service_id_stream(stream).await
}
