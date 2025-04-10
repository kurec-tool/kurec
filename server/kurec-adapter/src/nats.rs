use std::future::Future;

use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use bytes::Bytes;

use futures::StreamExt;
use kurec_interface::KurecConfig;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};
use tracing::debug;

// TODO: mirakcのイベント種別全部列挙しなきゃ・・・
#[derive(Clone, Debug, EnumIter, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum StreamType {
    SseEpgProgramsUpdated,
    SseTunerStatusChanged,
    SseRecordingStarted,
    SseRecordingStopped,
    SseRecordingFailed,
    SseRecordingRescheduled,
    SseRecordingRecordSaved,
    SseRecordingRecordRemoved,
    SseRecordingRecordBroken,
    SseOnairProramChanged,
    EpgUpdated,
    EpgConverted,
    OgpRequest,
    RuleUpdated,
    ScheduleUpdated,
    RecordRecording,
    RecordFinishied,
    RecordCanceled,
    RecordFailed,
}

#[derive(Clone, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum KvsType {
    Ogp,
    UrlHash,
    EpgConverted,
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

    fn get_prefixed_subject_name_by_event_name(&self, event_name: &str) -> String {
        format!(
            "{}-sse-{}",
            self.config.prefix,
            event_name.replace(".", "-").replace("_", "-")
        )
    }

    fn get_prefixed_subject_name_by_stream_type(&self, stream_type: &StreamType) -> String {
        format!("{}-{}", self.config.prefix, stream_type.as_ref())
    }

    fn get_prefixed_consumer_name(&self, stream_type: &StreamType, consumer_name: &str) -> String {
        format!(
            "{}-{}",
            self.get_prefixed_subject_name_by_stream_type(stream_type),
            consumer_name
        )
    }

    fn get_prefixed_kvs_name(&self, kvs_type: &KvsType) -> String {
        format!("{}-{}", self.config.prefix, kvs_type.as_ref())
    }

    async fn connect(&self) -> Result<jetstream::Context, anyhow::Error> {
        let nats_url = &self.nats_url;
        tracing::debug!("connecting to NATS: {}", nats_url);
        let nc = async_nats::connect(nats_url).await.unwrap();
        let jc = async_nats::jetstream::new(nc);
        Ok(jc)
    }

    async fn get_or_create_stream(
        &self,
        jc: &jetstream::Context,
        stream_type: &StreamType,
    ) -> Result<Stream, anyhow::Error> {
        // TODO: ConfigをKuRecConfigから取得する・・・いらないかな？
        let stream = jc
            .get_or_create_stream(jetstream::stream::Config {
                name: self.get_prefixed_subject_name_by_stream_type(stream_type),
                max_messages: 10_000_000,
                ..Default::default()
            })
            .await
            .unwrap();
        Ok(stream)
    }

    async fn get_or_create_stream_by_event_name(
        &self,
        jc: &jetstream::Context,
        event_name: &str,
    ) -> Result<Stream, anyhow::Error> {
        // TODO: ConfigをKuRecConfigから取得する
        let stream = jc
            .get_or_create_stream(jetstream::stream::Config {
                name: self.get_prefixed_subject_name_by_event_name(event_name),
                max_messages: 10_000_000,
                ..Default::default()
            })
            .await
            .unwrap();
        Ok(stream)
    }

    pub async fn publish_to_stream_by_event_name(
        &self,
        event_name: &str,
        payload: Bytes,
    ) -> Result<(), anyhow::Error> {
        let subject_name = self.get_prefixed_subject_name_by_event_name(event_name);
        let jc = self.connect().await.unwrap();
        let _ = self
            .get_or_create_stream_by_event_name(&jc, event_name)
            .await
            .unwrap();
        tracing::debug!(
            "publishing to NATS subject: {} payload_len: {}",
            subject_name,
            payload.len()
        );
        let resp = jc.publish(subject_name, payload).await.unwrap();
        tracing::debug!("published: {:?}", resp);
        let resp2 = resp.await.unwrap();
        tracing::debug!("sequence: {}", resp2.sequence);
        Ok(())
    }

    pub async fn publish_to_stream(
        &self,
        to: StreamType,
        payload: Bytes,
    ) -> Result<(), anyhow::Error> {
        let subject_name = self.get_prefixed_subject_name_by_stream_type(&to);
        let jc = self.connect().await.unwrap();
        let _ = self.get_or_create_stream(&jc, &to).await.unwrap();
        tracing::debug!(
            "publishing to NATS subject: {} payload_len: {}",
            subject_name,
            payload.len()
        );
        // 2段階に分けてるのはデバッグ用。まとめてawait?.await?でも良さそう
        let resp = jc.publish(subject_name, payload).await.unwrap();
        tracing::debug!("published: {:?}", resp);
        let resp2 = resp.await.unwrap();
        tracing::debug!("ack awaited: {:?}", resp2);
        Ok(())
    }

    // fはErrを返せばエラーで終了する、Ok(None)を返せば次のメッセージを待つ、Ok(Some(v))を返せばvをpublishする
    pub async fn filter_map_stream<T, U, F>(
        &self,
        from: StreamType,
        to: StreamType,
        consumer_name: &str,
        f: F,
    ) -> Result<(), anyhow::Error>
    where
        T: for<'a> Deserialize<'a>,
        U: Serialize,
        F: Fn(T) -> Result<Option<U>, anyhow::Error>,
    {
        let jc = self.connect().await.unwrap();
        let from_stream = self.get_or_create_stream(&jc, &from).await.unwrap();
        let consumer: PullConsumer = from_stream
            .get_or_create_consumer(
                "kurec-hogehoge-consumer", // この名前の意味は？
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.get_prefixed_consumer_name(&from, consumer_name)),
                    // TODO: Config調整
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        // publishする方はstream使わなくて良いが、Config設定する必要があるのでget_or_create_streamを使う
        let stream = self.get_or_create_stream(&jc, &to).await.unwrap();

        let to_subject_name = self.get_prefixed_subject_name_by_stream_type(&to);
        let mut messages = consumer.messages().await.unwrap();
        while let Some(Ok(msg)) = messages.next().await {
            let message: T = serde_json::from_slice(msg.payload.as_ref()).unwrap();
            match f(message) {
                Ok(None) => continue,
                Ok(Some(mapped)) => {
                    jc.publish(
                        to_subject_name.clone(),
                        serde_json::to_vec(&mapped).unwrap().into(),
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("publish error: {:?}", e))
                    .unwrap();
                }
                Err(e) => return Err(e),
            }
            msg.ack()
                .await
                .map_err(|e| anyhow::anyhow!("ack error: {:?}", e))
                .unwrap();
        }

        todo!()
    }

    // fはErrを返せばエラーで終了する、Ok(None)を返せば次のメッセージを待つ、Ok(Some(v))を返せばvをpublishする
    pub async fn filter_map_stream_async<T, U, F, Fut>(
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
        let jc = self.connect().await.unwrap();
        let from_stream = self.get_or_create_stream(&jc, &from).await.unwrap();
        let consumer: PullConsumer = from_stream
            .get_or_create_consumer(
                "kurec-hogehoge-consumer", // この名前の意味は？
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.get_prefixed_consumer_name(&from, consumer_name)),
                    // TODO: Config調整
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        // publishする方はstream使わなくて良いが、Config設定する必要があるのでget_or_create_streamを使う
        let _ = self.get_or_create_stream(&jc, &to).await.unwrap();

        let to_subject_name = self.get_prefixed_subject_name_by_stream_type(&to);
        let mut messages = consumer.messages().await.unwrap();
        while let Some(Ok(msg)) = messages.next().await {
            let message: T = serde_json::from_slice(msg.payload.as_ref()).unwrap();
            match f(message).await {
                Ok(None) => continue,
                Ok(Some(mapped)) => {
                    jc.publish(
                        to_subject_name.clone(),
                        serde_json::to_vec(&mapped).unwrap().into(),
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("publish error: {:?}", e))
                    .unwrap();
                }
                Err(e) => return Err(e),
            }
            msg.ack()
                .await
                .map_err(|e| anyhow::anyhow!("ack error: {:?}", e))
                .unwrap();
        }

        todo!()
    }

    // fはErrを返せばエラーで終了する、Ok(None)を返せば次のメッセージを待つ、Ok(Some(v))を返せばvをpublishする
    pub async fn stream_sink_async<T, F, Fut>(
        &self,
        from: StreamType,
        consumer_name: &str,
        f: F,
    ) -> Result<(), anyhow::Error>
    where
        T: for<'a> Deserialize<'a>,
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<(), anyhow::Error>>,
    {
        let jc = self.connect().await.unwrap();
        let from_stream = self.get_or_create_stream(&jc, &from).await.unwrap();
        let consumer: PullConsumer = from_stream
            .get_or_create_consumer(
                "kurec-hogehoge-consumer", // この名前の意味は？
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.get_prefixed_consumer_name(&from, consumer_name)),
                    // TODO: Config調整
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let mut messages = consumer.messages().await.unwrap();
        while let Some(Ok(msg)) = messages.next().await {
            let message: T = serde_json::from_slice(msg.payload.as_ref()).unwrap();
            match f(message).await {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
            msg.ack()
                .await
                .map_err(|e| anyhow::anyhow!("ack error: {:?}", e))
                .unwrap();
        }

        todo!()
    }

    pub async fn kv_exists_key(
        &self,
        kvs_type: &KvsType,
        key: &str,
    ) -> Result<bool, anyhow::Error> {
        let jc = self.connect().await.unwrap();
        let bucket = self.get_prefixed_kvs_name(kvs_type);
        let kv = match jc.get_key_value(&bucket).await {
            Ok(kv) => kv,
            Err(_) => jc
                .create_key_value(jetstream::kv::Config {
                    bucket,
                    ..Default::default()
                })
                .await
                .unwrap(),
        };
        debug!("key: {}", key);
        let entry = kv.entry(key).await.unwrap();
        Ok(entry.is_some())
    }

    pub async fn kv_put_str(
        &self,
        kvs_type: &KvsType,
        key: &str,
        value: &str,
    ) -> Result<(), anyhow::Error> {
        let jc = self.connect().await.unwrap();
        let bucket = self.get_prefixed_kvs_name(&kvs_type);
        let kv = match jc.get_key_value(&bucket).await {
            Ok(kv) => kv,
            Err(_) => jc
                .create_key_value(jetstream::kv::Config {
                    bucket,
                    ..Default::default()
                })
                .await
                .unwrap(),
        };
        let bytes = Bytes::copy_from_slice(value.as_bytes());
        kv.put(key, bytes).await.unwrap();
        Ok(())
    }

    pub async fn kv_put_bytes<T: AsRef<[u8]>>(
        &self,
        kvs_type: &KvsType,
        key: &str,
        value: T,
    ) -> Result<(), anyhow::Error> {
        let jc = self.connect().await.unwrap();
        let bucket = self.get_prefixed_kvs_name(kvs_type);
        let kv = match jc.get_key_value(&bucket).await {
            Ok(kv) => kv,
            Err(_) => jc
                .create_key_value(jetstream::kv::Config {
                    bucket,
                    ..Default::default()
                })
                .await
                .unwrap(),
        };
        let bytes = Bytes::copy_from_slice(value.as_ref());

        kv.put(key, bytes).await.unwrap();
        Ok(())
    }

    pub async fn kv_get_bytes(
        &self,
        kvs_type: &KvsType,
        key: &str,
    ) -> Result<Bytes, anyhow::Error> {
        let jc = self.connect().await.unwrap();
        let bucket = self.get_prefixed_kvs_name(kvs_type);
        let kv = match jc.get_key_value(&bucket).await {
            Ok(kv) => kv,
            Err(_) => jc
                .create_key_value(jetstream::kv::Config {
                    bucket,
                    ..Default::default()
                })
                .await
                .unwrap(),
        };
        match kv.get(key).await.unwrap() {
            Some(v) => Ok(v),
            None => Err(anyhow::anyhow!("key not found")),
        }
    }

    pub async fn kv_get_decoded<T: for<'a> Deserialize<'a>>(
        &self,
        kvs_type: &KvsType,
        key: &str,
    ) -> Result<T, anyhow::Error> {
        let bytes = self.kv_get_bytes(kvs_type, key).await?;
        let v: T = serde_json::from_slice(bytes.as_ref()).unwrap();
        Ok(v)
    }

    pub async fn kv_get_keys(&self, kvs_type: &KvsType) -> Result<Vec<String>, anyhow::Error> {
        let jc = self.connect().await.unwrap();
        let bucket = self.get_prefixed_kvs_name(kvs_type);
        let kv = match jc.get_key_value(&bucket).await {
            Ok(kv) => kv,
            Err(_) => jc
                .create_key_value(jetstream::kv::Config {
                    bucket,
                    ..Default::default()
                })
                .await
                .unwrap(),
        };
        let mut keys = kv.keys().await.unwrap();
        let mut key_list: Vec<String> = Vec::new();
        while let Some(key) = keys.next().await {
            key_list.push(key.unwrap());
        }
        Ok(key_list)
    }

    pub async fn kv_get_all_bytes(&self, kvs_type: &KvsType) -> Result<Vec<Bytes>, anyhow::Error> {
        let jc = self.connect().await.unwrap();
        let bucket = self.get_prefixed_kvs_name(kvs_type);
        let kv = match jc.get_key_value(&bucket).await {
            Ok(kv) => kv,
            // TODO: 作成しないようにしても良いかも
            Err(_) => jc
                .create_key_value(jetstream::kv::Config {
                    bucket,
                    ..Default::default()
                })
                .await
                .unwrap(),
        };
        let mut keys = kv.keys().await.unwrap();
        let mut values_list: Vec<Bytes> = Vec::new();
        while let Some(key) = keys.next().await {
            let key = key.unwrap();
            let entry = kv.entry(&key).await.unwrap();
            values_list.push(entry.unwrap().value);
        }
        Ok(values_list)
    }

    pub async fn kv_get_all_decoded<T: for<'a> Deserialize<'a>>(
        &self,
        kvs_type: &KvsType,
    ) -> Result<Vec<T>, anyhow::Error> {
        let bytes_list = self.kv_get_all_bytes(kvs_type).await?;
        let mut values_list: Vec<T> = Vec::new();
        for bytes in bytes_list {
            let v: T = serde_json::from_slice(bytes.as_ref()).unwrap();
            values_list.push(v);
        }
        Ok(values_list)
    }

    pub async fn initialize(&self) -> Result<(), anyhow::Error> {
        let jc = self.connect().await.unwrap();
        for stream_type in StreamType::iter() {
            let _ = self.get_or_create_stream(&jc, &stream_type).await?;
        }
        for kvs_type in KvsType::iter() {
            let _ = match jc
                .get_key_value(self.get_prefixed_kvs_name(&kvs_type))
                .await
            {
                Ok(kv) => kv,
                Err(_) => {
                    jc.create_key_value(jetstream::kv::Config {
                        bucket: self.get_prefixed_kvs_name(&kvs_type),
                        ..Default::default()
                    })
                    .await?
                }
            };
        }
        Ok(())
    }
}
