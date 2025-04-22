use crate::error::JetstreamError;
// use crate::{JsPublisher, JsSubscriber}; // 未使用のため削除
use async_nats::jetstream::{
    self,
    context::GetStreamError,
    stream::{Config as StreamConfigNats, Info as StreamInfo, InfoError}, // stream::InfoError をインポート
};
use async_trait::async_trait;
use domain::{event::Event, ports::event_sink::DomainEventSink};
use serde::Serialize;
use shared_types::stream::StreamConfig;
use std::marker::PhantomData;
// use std::sync::Arc; // 未使用のため削除
use tracing::instrument;

/// イベントストリームへのアクセスを提供するジェネリック型
///
/// E: イベントの型 (domain::event::Eventを実装)
#[derive(Clone)]
pub struct EventStream<E: Event> {
    jetstream: jetstream::Context,
    stream_name: String,
    config: StreamConfig, // KuRecの設定としてのStreamConfig
    _phantom: PhantomData<E>,
}

impl<E: Event> EventStream<E> {
    /// 新しいEventStreamインスタンスを作成します。
    ///
    /// この関数はストリームが存在しない場合に作成または更新を試みます。
    pub async fn new(
        jetstream: jetstream::Context,
        stream_name: String,
        config: StreamConfig, // KuRecの設定
    ) -> Result<Self, JetstreamError> {
        // StreamConfig から NATS の Config を構築
        let nats_config = StreamConfigNats {
            name: config.name.clone(),
            description: config.description.clone(),
            subjects: config.subjects.clone().unwrap_or_default(),
            retention: config
                .retention
                .map(|r| match r {
                    shared_types::stream::RetentionPolicy::Limits => {
                        async_nats::jetstream::stream::RetentionPolicy::Limits
                    }
                    shared_types::stream::RetentionPolicy::Interest => {
                        async_nats::jetstream::stream::RetentionPolicy::Interest
                    }
                    shared_types::stream::RetentionPolicy::WorkQueue => {
                        async_nats::jetstream::stream::RetentionPolicy::WorkQueue
                    }
                })
                .unwrap_or(async_nats::jetstream::stream::RetentionPolicy::Limits),
            max_consumers: config.max_consumers.map(|v| v as i32).unwrap_or(-1),
            max_messages: config.max_msgs.map(|v| v as i64).unwrap_or(-1),
            max_bytes: config.max_bytes.map(|v| v as i64).unwrap_or(-1),
            max_age: config.max_age.unwrap_or_default(),
            max_message_size: config.max_msg_size.map(|v| v as i32).unwrap_or(-1),
            storage: config
                .storage
                .map(|s| match s {
                    shared_types::stream::StorageType::File => {
                        async_nats::jetstream::stream::StorageType::File
                    }
                    shared_types::stream::StorageType::Memory => {
                        async_nats::jetstream::stream::StorageType::Memory
                    }
                })
                .unwrap_or(async_nats::jetstream::stream::StorageType::File),
            discard: config
                .discard
                .map(|d| match d {
                    shared_types::stream::DiscardPolicy::Old => {
                        async_nats::jetstream::stream::DiscardPolicy::Old
                    }
                    shared_types::stream::DiscardPolicy::New => {
                        async_nats::jetstream::stream::DiscardPolicy::New
                    }
                })
                .unwrap_or(async_nats::jetstream::stream::DiscardPolicy::Old),
            duplicate_window: config.duplicate_window.unwrap_or_default(),
            allow_rollup: config.allow_rollup.unwrap_or(false),
            deny_delete: config.deny_delete.unwrap_or(false),
            deny_purge: config.deny_purge.unwrap_or(false),
            num_replicas: 1,
            allow_direct: true,
            mirror_direct: false,
            compression: None,
            max_messages_per_subject: -1,
            no_ack: false,
            template_owner: Default::default(),
            republish: None,
            sealed: false,
            metadata: Default::default(),
            placement: None,
            mirror: None,
            sources: None,
            subject_transform: None,
            discard_new_per_subject: false,
            ..Default::default()
        };

        // ストリームが存在するか確認
        match jetstream.get_stream(&stream_name).await {
            Ok(mut stream) => {
                // `info` が `&mut self` を取るため `mut` に変更
                // get_stream は Stream を返す
                // ストリームが存在する場合、設定が異なれば更新
                let stream_info = stream.info().await?; // .await? を追加 (From<InfoError> があるので ? で OK)
                let existing_config = &stream_info.config;
                if *existing_config != nats_config {
                    tracing::info!(stream_name = %stream_name, "Updating existing stream config");
                    jetstream
                        .update_stream(&nats_config)
                        .await
                        .map_err(JetstreamError::UpdateStream)?;
                } else {
                    tracing::info!(stream_name = %stream_name, "Stream already exists with the same config");
                }
            }
            Err(e) => {
                // エラーメッセージで NotFound を判定
                if e.to_string().contains("stream not found") {
                    // ストリームが存在しない場合、新規作成
                    tracing::info!(stream_name = %stream_name, "Stream not found, creating new stream");
                    jetstream
                        .create_stream(nats_config)
                        .await
                        .map_err(JetstreamError::CreateStream)?;
                } else {
                    // その他のエラー
                    tracing::error!(stream_name = %stream_name, error = %e, "Error getting stream info");
                    return Err(JetstreamError::GetStream(e));
                }
            }
        }

        Ok(Self {
            jetstream: jetstream.clone(),
            stream_name,
            config,
            _phantom: PhantomData,
        })
    }

    // publish メソッドは DomainEventSink トレイト実装に移動するため削除

    // TODO: subscribeメソッドの実装 (JsSubscriberを利用)
    // pub async fn subscribe<F>(&self, handler: F) -> Result<(), JetstreamError>
    // where
    //     F: Fn(E) -> Fut + Send + Sync + 'static,
    //     Fut: Future<Output = ()> + Send + 'static,
    // {
    //     // ...
    // }
}

// DomainEventSink トレイトの実装
#[async_trait]
impl<E: Event> DomainEventSink<E> for EventStream<E> {
    /// イベントを発行します。
    #[instrument(skip(self, event), fields(stream = %self.stream_name, event_type = %E::event_name()))]
    async fn publish(&self, event: E) -> anyhow::Result<()>
    where
        E: Serialize,
    {
        let subject = format!("{}.{}", self.stream_name, E::event_name());
        let payload = serde_json::to_vec(&event)?;
        self.jetstream
            .publish(subject, payload.into())
            .await?
            .await?; // double await for publish ack
        Ok(())
    }
}

// TryFrom 実装は削除
