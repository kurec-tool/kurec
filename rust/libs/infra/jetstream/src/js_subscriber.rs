use anyhow::Result;
use async_nats::jetstream;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use heck::ToSnakeCase;
use shared_core::event_metadata::Event;
use shared_core::event_subscriber::{AckHandle, EventSubscriber};
use std::marker::PhantomData;

use crate::JetStreamCtx;

/// 型情報を使用してdurable nameを生成する関数
fn generate_durable_name<E: Event>() -> String {
    use std::any::type_name;

    // イベント型の完全修飾名を取得
    let event_type = type_name::<E>();

    // モジュールパスを含む型名をスネークケースに変換
    let type_name_snake = event_type.replace("::", "_").to_snake_case();

    // ストリーム名と型情報を組み合わせる
    format!("consumer_{}_{}", E::stream_name(), type_name_snake)
}

/// JetStreamを使用したイベント購読者
pub struct JsSubscriber<E: Event> {
    js_ctx: JetStreamCtx,
    _phantom: PhantomData<E>,
}

impl<E: Event> JsSubscriber<E> {
    /// イベント型から新しいJsSubscriberを作成
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

        // イベント型から一意なdurable nameを生成
        let durable_name = generate_durable_name::<E>();

        // ストリーム設定を取得（存在すれば）
        let config = shared_core::streams::get_stream_config(E::stream_name());

        // プルコンシューマーを作成
        let mut consumer_config = jetstream::consumer::pull::Config {
            durable_name: Some(durable_name),
            filter_subject: E::stream_subject().to_string(),
            ..Default::default()
        };

        // ストリーム設定が存在する場合、コンシューマー設定に適用
        if let Some(stream_config) = config {
            if let Some(max_deliver) = stream_config.max_deliver {
                // max_deliverの型を確認（async_natsのバージョンによって異なる可能性がある）
                // 現在のバージョンではOption<usize>ではなくi64のようです
                consumer_config.max_deliver = max_deliver as i64;
            }
            if let Some(ack_wait) = stream_config.ack_wait {
                consumer_config.ack_wait = ack_wait;
            }
        }

        let consumer = stream.create_consumer(consumer_config).await?;

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
