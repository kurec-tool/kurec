use anyhow::Result;
use async_nats::jetstream;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use shared_core::event_metadata::Event;
use shared_core::event_subscriber::{AckHandle, EventSubscriber};
use std::marker::PhantomData;

use crate::JetStreamCtx;

/// JetStreamを使用したイベント購読者
pub struct JsSubscriber<E: Event> {
    js_ctx: JetStreamCtx,
    _phantom: PhantomData<E>,
}

impl<E: Event> JsSubscriber<E> {
    /// 新しいJsSubscriberを作成
    pub fn new(js_ctx: JetStreamCtx) -> Self {
        Self {
            js_ctx,
            _phantom: PhantomData,
        }
    }

    /// HasStreamDefトレイトを使用して新しいJsSubscriberを作成
    pub fn from_event_type(js_ctx: JetStreamCtx) -> Self {
        Self {
            js_ctx,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<E: Event> EventSubscriber<E> for JsSubscriber<E> {
    async fn subscribe(&self) -> Result<BoxStream<'static, (E, AckHandle)>> {
        // JetStreamからコンシューマーを取得
        let stream = self.js_ctx.js.get_stream(E::stream_name()).await?;

        // プルコンシューマーを作成
        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                durable_name: Some(format!("consumer_{}", E::stream_subject())),
                filter_subject: E::stream_subject().to_string(),
                ..Default::default()
            })
            .await?;

        // メッセージをプル
        let messages = consumer.messages().await?;

        // メッセージをイベントとAckHandleに変換
        let events = messages.filter_map(move |msg_result| async move {
            match msg_result {
                Ok(msg) => {
                    // メッセージをデシリアライズ
                    let event: E = match serde_json::from_slice(&msg.payload) {
                        Ok(event) => event,
                        Err(e) => {
                            eprintln!("Failed to deserialize message: {}", e);
                            return None;
                        }
                    };

                    // AckHandleを作成
                    let ack_handle = AckHandle::new(Box::new(move || {
                        let msg = msg.clone();
                        Box::pin(async move {
                            msg.ack()
                                .await
                                .map_err(|e| anyhow::anyhow!("Failed to ack message: {}", e))
                        })
                    }));

                    Some((event, ack_handle))
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    None
                }
            }
        });

        Ok(Box::pin(events))
    }
}
