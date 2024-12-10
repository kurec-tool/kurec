use async_nats::jetstream;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::pin;

use kurec_interface::KurecConfig;
use kurec_proto::*;

#[derive(Clone, Debug)]
pub struct NatsAdapter {
    config: KurecConfig,
    nats_url: String,
}

impl NatsAdapter {
    pub fn new(config: KurecConfig) -> Self {
        let nats_url = config.nats.url.clone();
        Self { config, nats_url }
    }

    pub async fn copy_events_to_jetstream(
        &self,
        stream: impl Stream<Item = MirakcEventMessage>,
    ) -> Result<(), anyhow::Error> {
        pin!(stream);
        while let Some(ev) = stream.next().await {
            tracing::debug!("event: {:?}", ev);
            let base_name = ev.event.replace(".", "-").replace("_", "-");
            let v = ev.encode_to_vec();
            self.publish_to_stream(&base_name, v.into()).await?;
        }
        Err(anyhow::anyhow!("stream ended"))
    }

    fn get_subject_name(&self, base_name: &str) -> String {
        format!("{}-{}", self.config.prefix, base_name)
    }

    async fn create_stream(
        &self,
        stream_name: String,
    ) -> Result<jetstream::Context, anyhow::Error> {
        let nats_url = &self.nats_url;
        tracing::debug!("connecting to NATS: {}", nats_url);
        let nc = async_nats::connect(nats_url).await?;
        let jetstream = async_nats::jetstream::new(nc);
        // TODO: ConfigをKuRecConfigから取得する
        let _ = jetstream
            .get_or_create_stream(async_nats::jetstream::stream::Config {
                name: stream_name,
                max_messages: 10_000_000,
                ..Default::default()
            })
            .await?;
        Ok(jetstream)
    }

    async fn publish_to_stream(
        &self,
        subject_base_name: &str,
        payload: Bytes,
    ) -> Result<(), anyhow::Error> {
        let subject_name = self.get_subject_name(subject_base_name);
        let jetstream = self.create_stream(subject_name.clone()).await?;
        tracing::debug!(
            "publishing to NATS subject: {} payload_len: {}",
            subject_name,
            payload.len()
        );
        let resp = jetstream.publish(subject_name, payload).await?;
        tracing::debug!("published: {:?}", resp);
        let resp2 = resp.await?;
        tracing::debug!("sequence: {}", resp2.sequence);
        Ok(())
    }
}
