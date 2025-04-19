use anyhow::Result;
use async_nats::subject::ToSubject;
use async_trait::async_trait;
use shared_core::event_metadata::Event;
use shared_core::event_publisher::EventPublisher;
use std::marker::PhantomData;

use crate::JetStreamCtx;

/// JetStreamを使用したイベント発行者
pub struct JsPublisher<E: Event> {
    js_ctx: JetStreamCtx,
    _phantom: PhantomData<E>,
}

impl<E: Event> JsPublisher<E> {
    /// イベント型から新しいJsPublisherを作成
    pub fn from_event_type(js_ctx: JetStreamCtx) -> Self {
        Self {
            js_ctx,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<E: Event> EventPublisher<E> for JsPublisher<E> {
    async fn publish(&self, event: E) -> Result<()> {
        // イベントをJSONにシリアライズ
        let payload = serde_json::to_vec(&event)?;

        // JetStreamにパブリッシュ
        self.js_ctx
            .js
            .publish(E::stream_subject(), payload.into())
            .await?;

        Ok(())
    }
}
