use crate::ack::JsAck;
use crate::error::JsEventError;
use anyhow::Result;
use async_nats::jetstream::{self, consumer::pull::MessagesErrorKind};
use async_trait::async_trait;
use domain::event::Event;
use futures::stream::{BoxStream, StreamExt, TryStreamExt};
use heck::ToSnakeCase;
use infra_common::ackable_event::AckableEvent;
use infra_common::event_source::EventSource;
use std::any::type_name;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use infra_nats::NatsClient;

/// 型情報を使用してdurable nameを生成する関数 (stream_name を削除)
fn generate_durable_name<E: Event>() -> String {
    // イベント型の完全修飾名を取得
    let event_type = type_name::<E>();
    // モジュールパスを含む型名をスネークケースに変換
    let type_name_snake = event_type.replace("::", "_").to_snake_case();
    // コンシューマ名に型情報を含める (ストリーム名は含めない)
    format!("consumer_{}", type_name_snake)
}

// ヘルパー関数: 型名からスネークケースのサブジェクト名を取得 (js_publisher.rs と同じもの)
// TODO: クレート内の共通 util モジュールに移動する方が良い
fn type_name_to_snake_case<T: ?Sized + Event>() -> String {
    let type_name_full = type_name::<T>();
    let type_name_short = type_name_full.split("::").last().unwrap_or(type_name_full);
    type_name_short.to_snake_case()
}

/// JetStreamを使用したイベント購読者
pub struct JsSubscriber<E: Event> {
    nats_client: Arc<NatsClient>,
    event_stream: crate::event_stream::EventStream<E>,
}

impl<E: Event> JsSubscriber<E> {
    /// 新しいJsSubscriberを作成
    pub fn new(
        nats_client: Arc<NatsClient>,
        event_stream: crate::event_stream::EventStream<E>,
    ) -> Self {
        Self {
            nats_client,
            event_stream,
        }
    }

    /// イベントストリームを取得
    pub fn event_stream(&self) -> &crate::event_stream::EventStream<E> {
        &self.event_stream
    }
}

#[async_trait]
impl<E> EventSource<E, JsEventError> for JsSubscriber<E>
where
    E: Event + Debug + Send + Sync + 'static,
{
    async fn subscribe(&self) -> Result<BoxStream<'static, Result<AckableEvent<E>, JsEventError>>> {
        let stream_name = self.event_stream.stream_name();
        let subject_filter = type_name_to_snake_case::<E>(); // 型名からサブジェクト名を生成
        let durable_name = generate_durable_name::<E>(); // コンシューマ名

        // 個別の定数から StreamConfig を構築
        // 注: 現在の実装では、これらの定数は実際には使用されていません
        // 将来的には、E::STREAM_MAX_AGE などの定数を使用するように修正する必要があります
        let stream_config = crate::config::StreamConfig {
            max_age: None,
            max_messages: None,
            max_bytes: None,
            max_message_size: None,
            storage: None,
            retention: None,
            discard: None,
            duplicate_window: None,
            allow_rollup: None,
            deny_delete: None,
            deny_purge: None,
            description: None,
        };

        let js_ctx = self.nats_client.jetstream_context();

        // --- ストリームの存在確認と作成 ---
        let stream = match js_ctx.get_stream(stream_name).await {
            Ok(stream_info) => {
                debug!(stream = %stream_name, "Stream already exists");
                // TODO: ストリーム設定が STREAM_CONFIG と一致するか確認・更新？
                stream_info
            }
            Err(err) => {
                if err.to_string().contains("stream not found") {
                    warn!(stream = %stream_name, "Stream not found, attempting to create it");
                    let mut nats_config: async_nats::jetstream::stream::Config =
                        (&stream_config).into();
                    nats_config.name = stream_name.to_string();
                    // サブジェクトフィルターを設定 (特定のサブジェクトのみ購読)
                    nats_config.subjects = vec![subject_filter.clone()];

                    match js_ctx.create_stream(nats_config).await {
                        Ok(stream_info) => {
                            info!(stream = %stream_name, "Successfully created stream");
                            stream_info
                        }
                        Err(create_err) => {
                            error!(stream = %stream_name, error = %create_err, "Failed to create stream");
                            return Err(anyhow::anyhow!("Failed to create stream: {}", create_err));
                        }
                    }
                } else {
                    error!(stream = %stream_name, error = %err, "Failed to get stream info");
                    return Err(anyhow::anyhow!("Failed to get stream info: {}", err));
                }
            }
        };

        // --- コンシューマの作成 ---
        // TODO: コンシューマ設定 (ack_wait など) をどう扱うか？
        //       一旦デフォルト設定で作成する。必要なら subscribe の引数で受け取るなど変更。
        let consumer_config = jetstream::consumer::pull::Config {
            durable_name: Some(durable_name.clone()),
            filter_subject: subject_filter.clone(),
            // ack_policy: Default::default(), // Explicit (Default)
            // ack_wait: Duration::from_secs(30), // Default
            // max_deliver: -1, // Default (unlimited)
            ..Default::default()
        };

        // コンシューマを取得または作成
        // TODO: create_consumer は冪等ではないため、get -> create の方が安全か？
        let consumer = match stream.get_consumer(&durable_name).await {
            Ok(consumer) => {
                debug!(consumer = %durable_name, "Consumer already exists");
                // TODO: コンシューマ設定が config と一致するか確認・更新？
                consumer
            }
            Err(err) => {
                if err.to_string().contains("consumer not found") {
                    warn!(consumer = %durable_name, "Consumer not found, attempting to create it");
                    match stream.create_consumer(consumer_config).await {
                        Ok(consumer) => {
                            info!(consumer = %durable_name, "Successfully created consumer");
                            consumer
                        }
                        Err(create_err) => {
                            error!(consumer = %durable_name, error = %create_err, "Failed to create consumer");
                            return Err(anyhow::anyhow!(
                                "Failed to create consumer: {}",
                                create_err
                            ));
                        }
                    }
                } else {
                    error!(consumer = %durable_name, error = %err, "Failed to get consumer info");
                    return Err(anyhow::anyhow!("Failed to get consumer info: {}", err));
                }
            }
        };

        // メッセージストリームを取得
        let message_stream = match consumer.messages().await {
            Ok(stream) => stream,
            Err(e) => {
                error!(consumer = %durable_name, error = %e, "Failed to get messages stream");
                return Err(anyhow::anyhow!("Failed to get messages stream: {}", e));
            }
        };

        // durable_nameをクローンして'staticライフタイムを持つようにする
        let durable_name_clone = durable_name.clone();

        // メッセージをイベントに変換するストリームを作成
        let event_stream = message_stream
            .map_err(move |e| {
                // エラーの種類に応じてログレベルを変更
                match e.kind() {
                    MessagesErrorKind::MissingHeartbeat => {
                        debug!(consumer = %durable_name_clone, error = %e, "Messages stream heartbeat missed (expected)");
                    },
                    MessagesErrorKind::Pull => {
                        warn!(consumer = %durable_name_clone, error = %e, "Messages stream pull error");
                    },
                    MessagesErrorKind::PushBasedConsumer => {
                        warn!(consumer = %durable_name_clone, error = %e, "Messages stream push-based consumer error");
                    },
                    MessagesErrorKind::ConsumerDeleted => {
                        warn!(consumer = %durable_name_clone, error = %e, "Messages consumer deleted");
                    },
                    MessagesErrorKind::Other => {
                        error!(consumer = %durable_name_clone, error = %e, "Unknown error occurred");
                    },
                }
                
                let error = JsEventError::Stream { source: e };
                error.log();
                
                error
            })
            .and_then(|msg| async move {
                // メッセージをデシリアライズ
                match serde_json::from_slice::<E>(&msg.payload) {
                    Ok(event) => {
                        // AckableEventを作成して返す
                        let ack_fn = Box::new(JsAck::new(msg));
                        Ok(AckableEvent::new(event, ack_fn))
                    }
                    Err(e) => {
                        // デシリアライズエラー結果を返す
                        let error = JsEventError::Deserialize {
                            source: e,
                            payload: msg.payload.to_vec(),
                        };
                        
                        error.log();
                        
                        // デシリアライズエラーの場合はAckする
                        // (再試行しても同じエラーになるため)
                        if let Err(ack_err) = msg.ack().await {
                            error!(error = %ack_err, "Failed to ack message after deserialization error");
                        }
                        
                        Err(error)
                    }
                }
            });

        Ok(Box::pin(event_stream))
    }
}
