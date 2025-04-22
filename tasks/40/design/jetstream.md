# Issue #40: Ack/Nack機能の修正設計 - JetStream実装

JetStream実装では、Ackトレイトの実装とJsSubscriberの更新を行います。

## JsEventError

```rust
// infra/jetstream/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsEventError {
    #[error("Failed to deserialize message: {source}")]
    Deserialize {
        #[source]
        source: serde_json::Error,
        payload: Vec<u8>,
    },
    
    #[error("Connection error: {source}")]
    Connection {
        #[source]
        source: async_nats::Error,
        endpoint: String,
    },
    
    #[error("Stream error: {source}")]
    Stream {
        #[source]
        source: async_nats::jetstream::consumer::pull::MessagesError,
    },
    
    #[error("Consumer error: {source}")]
    Consumer {
        #[source]
        source: async_nats::jetstream::consumer::ConsumerError,
    },
    
    #[error("Ack error: {source}")]
    Ack {
        #[source]
        source: async_nats::Error,
    },
}

impl JsEventError {
    pub fn should_retry(&self) -> bool {
        match self {
            Self::Deserialize { .. } => false,
            Self::Connection { .. } => true,
            Self::Stream { .. } => true,
            Self::Consumer { .. } => true,
            Self::Ack { .. } => true,
        }
    }
    
    pub fn log(&self) {
        use tracing::{error, warn};
        
        match self {
            Self::Deserialize { payload, .. } => {
                error!(
                    error = %self,
                    payload_size = payload.len(),
                    payload_preview = %String::from_utf8_lossy(&payload[..std::cmp::min(100, payload.len())]),
                    "Failed to deserialize message"
                );
            },
            Self::Connection { endpoint, .. } => {
                warn!(
                    error = %self,
                    endpoint = %endpoint,
                    "Connection error"
                );
            },
            Self::Stream { .. } => {
                warn!(
                    error = %self,
                    "Stream error"
                );
            },
            Self::Consumer { .. } => {
                warn!(
                    error = %self,
                    "Consumer error"
                );
            },
            Self::Ack { .. } => {
                warn!(
                    error = %self,
                    "Ack error"
                );
            },
        }
    }
}
```

## JsAck

```rust
// infra/jetstream/src/ack.rs
use async_nats::jetstream;
use async_trait::async_trait;
use anyhow::Result;
use infra_common::ack::Ack;
use tracing::error;

use crate::error::JsEventError;

/// JetStream用のAck実装
pub struct JsAck {
    message: jetstream::Message,
}

impl JsAck {
    pub fn new(message: jetstream::Message) -> Self {
        Self { message }
    }
}

#[async_trait]
impl Ack for JsAck {
    async fn ack(&self) -> Result<()> {
        self.message.ack().await.map_err(|e| {
            error!(error = %e, "Failed to ack message");
            JsEventError::Ack { source: e }.into()
        })
    }
}
```

## JsSubscriber

```rust
// infra/jetstream/src/js_subscriber.rs
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
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use infra_nats::NatsClient;

// 型情報を使用してdurable nameを生成する関数
fn generate_durable_name<E: Event>() -> String {
    let event_type = type_name::<E>();
    let type_name_snake = event_type.replace("::", "_").to_snake_case();
    format!("consumer_{}", type_name_snake)
}

// 型名からスネークケースのサブジェクト名を取得
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
        let subject_filter = type_name_to_snake_case::<E>();
        let durable_name = generate_durable_name::<E>();

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

        // ストリームの存在確認と作成
        let stream = match js_ctx.get_stream(stream_name).await {
            Ok(stream_info) => {
                debug!(stream = %stream_name, "Stream already exists");
                stream_info
            }
            Err(err) => {
                if err.to_string().contains("stream not found") {
                    warn!(stream = %stream_name, "Stream not found, attempting to create it");
                    let mut nats_config: async_nats::jetstream::stream::Config =
                        (&stream_config).into();
                    nats_config.name = stream_name.to_string();
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

        // コンシューマの作成
        let consumer_config = jetstream::consumer::pull::Config {
            durable_name: Some(durable_name.clone()),
            filter_subject: subject_filter.clone(),
            ..Default::default()
        };

        // コンシューマを取得または作成
        let consumer = match stream.get_consumer(&durable_name).await {
            Ok(consumer) => {
                debug!(consumer = %durable_name, "Consumer already exists");
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
        let endpoint = format!("nats://{}", self.nats_client.client().connection_info().server());

        // メッセージをイベントに変換するストリームを作成
        let event_stream = message_stream
            .map_err(move |e| {
                // メッセージストリームのエラー
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
                        warn!(consumer = %durable_name_clone, error = %e, "Messages stream consumer deleted");
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
                        // 成功結果を返す
                        let ack_fn = Box::new(JsAck::new(msg));
                        Ok(AckableEvent::new(event, ack_fn))
                    }
                    Err(e) => {
                        // デシリアライズエラー結果を返す
                        let error = JsEventError::Deserialize {
                            source: e,
                            payload: msg.payload.clone(),
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
```

## 設計のポイント

1. **JsAck**:
   - JetStreamのメッセージに対するAck実装
   - `ack`メソッドを呼び出すと、JetStreamのメッセージに対してAckを送信

2. **JsEventError**:
   - JetStream関連のエラーを表現する列挙型
   - `should_retry`メソッドで、エラーが再試行可能かどうかを判断
   - `log`メソッドで、エラーの詳細をログに記録

3. **JsSubscriber**:
   - JetStreamからのメッセージを購読するためのクラス
   - `subscribe`メソッドで、AckableEventを返すストリームを提供
   - デシリアライズエラーの場合は自動的にAckを送信（再試行しても同じエラーになるため）
