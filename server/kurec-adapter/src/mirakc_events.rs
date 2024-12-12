use eventsource_stream::Eventsource;
use futures::{future, Stream, StreamExt};
use kurec_interface::{KurecConfig, MirakcEventMessage};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct MirakcEventsAdapter {
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionResponse {
    current: String,
    latest: String,
}

impl MirakcEventsAdapter {
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
        Ok(Self { url })
    }

    // lifetimeのこと良く分からず警告潰しちゃってるのでちゃんと調べる
    #[allow(clippy::needless_lifetimes)]
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
        Ok(s.filter_map(|e| future::ready(e.ok()))
            .map(|i| MirakcEventMessage {
                tuner_url: self.url.clone(),
                event: i.event,
                data: i.data,
            }))
    }
}
