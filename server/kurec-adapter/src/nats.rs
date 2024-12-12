use std::future::Future;

use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use bytes::Bytes;

use futures::StreamExt;
use kurec_interface::KurecConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum StreamType {
    SseEpgProgramsUpdated,
    EpgUpdated,
    EpgConverted,
}

impl StreamType {
    fn as_str(&self) -> &str {
        match self {
            StreamType::SseEpgProgramsUpdated => "sse-epg-programs-updated",
            StreamType::EpgUpdated => "epg-updated",
            StreamType::EpgConverted => "epg-converted",
        }
    }
}

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

    fn get_prefixed_subject_name(&self, stream_type: &StreamType) -> String {
        format!("{}-{}", self.config.prefix, stream_type.as_str())
    }

    fn get_prefixed_consumer_name(&self, stream_type: &StreamType, consumer_name: &str) -> String {
        format!(
            "{}-{}",
            self.get_prefixed_subject_name(stream_type),
            consumer_name
        )
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
        stream_type: &StreamType,
    ) -> Result<Stream, anyhow::Error> {
        // TODO: ConfigをKuRecConfigから取得する
        let stream = jc
            .get_or_create_stream(jetstream::stream::Config {
                name: self.get_prefixed_subject_name(stream_type),
                max_messages: 10_000_000,
                ..Default::default()
            })
            .await?;
        Ok(stream)
    }

    pub async fn publish_to_stream(
        &self,
        stream_type: StreamType,
        payload: Bytes,
    ) -> Result<(), anyhow::Error> {
        let subject_name = self.get_prefixed_subject_name(&stream_type);
        let jc = self.connect().await?;
        let _ = self.get_or_create_stream(&jc, &stream_type).await?;
        tracing::debug!(
            "publishing to NATS subject: {} payload_len: {}",
            subject_name,
            payload.len()
        );
        let resp = jc.publish(subject_name, payload).await?;
        tracing::debug!("published: {:?}", resp);
        let resp2 = resp.await?;
        tracing::debug!("sequence: {}", resp2.sequence);
        Ok(())
    }

    // fはErrを返せばエラーで終了する、Ok(None)を返せば次のメッセージを待つ、Ok(Some(v))を返せばvをpublishする
    pub async fn filter_map_stream<T, U, F, Fut>(
        &self,
        from: StreamType,
        to: StreamType,
        consumer_name: &str,
        f: F,
    ) -> Result<(), anyhow::Error>
    where
        T: for<'a> Deserialize<'a>,
        U: Serialize,
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<Option<U>, anyhow::Error>>,
    {
        let jc = self.connect().await?;
        let from_stream = self.get_or_create_stream(&jc, &from).await?;
        let consumer: PullConsumer = from_stream
            .get_or_create_consumer(
                "kurec-hogehoge-consumer", // この名前の意味は？
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.get_prefixed_consumer_name(&from, consumer_name)),
                    // TODO: Config調整
                    ..Default::default()
                },
            )
            .await?;
        // publishする方はstream使わなくて良いが、Config設定する必要があるのでget_or_create_streamを使う
        let _ = self.get_or_create_stream(&jc, &to).await?;

        let to_subject_name = self.get_prefixed_subject_name(&to);
        let mut messages = consumer.messages().await?;
        while let Some(Ok(msg)) = messages.next().await {
            let message: T = serde_json::from_slice(msg.payload.as_ref())?;
            match f(message).await {
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
