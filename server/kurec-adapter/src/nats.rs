use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use bytes::Bytes;

use futures::{future, StreamExt, TryStreamExt};
use kurec_interface::KurecConfig;
use serde::{Deserialize, Serialize};

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

    fn get_prefixed_name(&self, base_name: &str) -> String {
        format!("{}-{}", self.config.prefix, base_name)
    }

    async fn connect(&self) -> Result<jetstream::Context, anyhow::Error> {
        let nats_url = &self.nats_url;
        tracing::debug!("connecting to NATS: {}", nats_url);
        let nc = async_nats::connect(nats_url).await?;
        let jc = async_nats::jetstream::new(nc);
        Ok(jc)
    }

    async fn get_or_create_stream(
        &self,
        jc: &jetstream::Context,
        subject_name: String,
    ) -> Result<Stream, anyhow::Error> {
        // TODO: ConfigをKuRecConfigから取得する
        let stream = jc
            .get_or_create_stream(jetstream::stream::Config {
                name: subject_name,
                max_messages: 10_000_000,
                ..Default::default()
            })
            .await?;
        Ok(stream)
    }

    pub async fn publish_to_stream(
        &self,
        subject_base_name: &str,
        payload: Bytes,
    ) -> Result<(), anyhow::Error> {
        let subject_name = self.get_prefixed_name(subject_base_name);
        let jc = self.connect().await?;
        let _ = self.get_or_create_stream(&jc, subject_name.clone()).await?;
        tracing::debug!(
            "publishing to NATS subject: {} payload_len: {}",
            subject_base_name,
            payload.len()
        );
        let resp = jc.publish(subject_name, payload).await?;
        tracing::debug!("published: {:?}", resp);
        let resp2 = resp.await?;
        tracing::debug!("sequence: {}", resp2.sequence);
        Ok(())
    }

    // fはErrを返せばエラーで終了する、Ok(None)を返せば次のメッセージを待つ、Ok(Some(v))を返せばvをpublishする
    pub async fn filter_map_stream<T, U, F>(
        &self,
        from_subject_base_name: &str,
        to_subject_base_name: &str,
        f: F,
    ) -> Result<(), anyhow::Error>
    where
        T: for<'a> Deserialize<'a>,
        U: Serialize,
        F: Fn(T) -> Result<Option<U>, anyhow::Error>,
    {
        let from_subject_name = self.get_prefixed_name(from_subject_base_name);
        let to_subject_name = self.get_prefixed_name(to_subject_base_name);
        let jc = self.connect().await?;
        let from_stream = self
            .get_or_create_stream(&jc, from_subject_name.clone())
            .await?;
        let consumer: PullConsumer = from_stream
            .get_or_create_consumer(
                "kurec-hogehoge-consumer", // この名前の意味は？
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.get_prefixed_name("epg-updated")),
                    // TODO: Config調整
                    ..Default::default()
                },
            )
            .await?;
        // publishする方はstream使わなくて良いが、Config設定する必要があるのでget_or_create_streamを使う
        let _ = self
            .get_or_create_stream(&jc, to_subject_name.clone())
            .await?;

        let mut messages = consumer.messages().await?;
        while let Some(Ok(msg)) = messages.next().await {
            let message: T = serde_json::from_slice(msg.payload.as_ref())?;
            match f(message) {
                Ok(None) => continue,
                Ok(Some(mapped)) => {
                    jc.publish(to_subject_name.clone(), serde_json::to_vec(&mapped)?.into())
                        .await
                        .map_err(|e| anyhow::anyhow!("publish error: {:?}", e))?;
                }
                Err(e) => return Err(e),
            }
            msg.ack()
                .await
                .map_err(|e| anyhow::anyhow!("ack error: {:?}", e))?;
        }

        todo!()
    }
}
