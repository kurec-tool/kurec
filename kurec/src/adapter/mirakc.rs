use std::error::Error;

use crate::kurec_config::KurecConfig;
use bytes::Bytes;
use eventsource_stream::{EventStream, Eventsource};
use futures::{future, Stream, StreamExt};
use kurec_proto::MirakcEventMessage;
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct MirakcAdapter {
    config: KurecConfig,
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionResponse {
    current: String,
    latest: String,
}

impl MirakcAdapter {
    pub async fn try_new(config: KurecConfig, tuner_name: &str) -> Result<Self, anyhow::Error> {
        let url = config
            .tuners
            .get(tuner_name)
            .ok_or_else(|| anyhow::anyhow!("Tuner not found"))?
            .clone();
        let version_url = format!("{}/api/version", url);
        let resp = reqwest::Client::new()
            .get(&version_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("error: {:?}", e))?
            .json::<VersionResponse>()
            .await?;
        tracing::debug!(
            "mirakc {} current: {}, latest: {}",
            tuner_name,
            resp.current,
            resp.latest
        );
        Ok(Self { config, url })
    }

    pub async fn get_events_stream<'a>(
        &'a self,
    ) -> Result<impl Stream<Item = MirakcEventMessage> + 'a, anyhow::Error> {
        let events_url = format!("{}/events", self.url);
        let s = reqwest::Client::new()
            .get(&events_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
            .bytes_stream()
            .eventsource();
        Ok(s.filter(|e| future::ready(e.is_ok()))
            .map(|e| e.expect("already checked"))
            .map(|i| MirakcEventMessage {
                tuner_url: self.url.clone(),
                event: i.event,
                data: i.data,
            }))
    }
}
