use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc; // Arc をインポート
                    // use domain::events::kurec_events::EpgStoredEvent; // EpgNotifier 関連削除
                    // use domain::ports::notifiers::EpgNotifier; // EpgNotifier 関連削除
use shared_core::event_metadata::Event;
use shared_core::event_sink::EventSink; // event_publisher -> event_sink
use std::marker::PhantomData;
use tracing::{debug, error, instrument}; // debug, error マクロを追加

use crate::JetStreamCtx;

/// JetStreamを使用したイベント発行者
pub struct JsPublisher<E: Event> {
    js_ctx: Arc<JetStreamCtx>, // JetStreamCtx -> Arc<JetStreamCtx>
    _phantom: PhantomData<E>,
}

impl<E: Event> JsPublisher<E> {
    /// イベント型から新しいJsPublisherを作成
    pub fn from_event_type(js_ctx: Arc<JetStreamCtx>) -> Self {
        // JetStreamCtx -> Arc<JetStreamCtx>
        Self {
            js_ctx,
            _phantom: PhantomData,
        }
    }

    // EpgNotifier 用のコンストラクタ (必要であれば)
    // ジェネリック版と共存させるか、別の構造体にするか検討
    // 今回は既存の JsPublisher に実装を追加する
    // pub fn new_epg_notifier(js_ctx: JetStreamCtx) -> Self {
    //     Self {
    //         js_ctx,
    //         _phantom: PhantomData, // PhantomData の扱いを再考する必要あり
    //     }
    // }
}

#[async_trait]
// EventPublisher -> EventSink
impl<E: Event> EventSink<E> for JsPublisher<E> {
    #[instrument(skip(self, event), fields(subject = %E::stream_subject()))]
    async fn publish(&self, event: E) -> Result<()> {
        let subject = E::stream_subject();
        debug!(subject = %subject, "Serializing event for JetStream");

        // イベントをJSONにシリアライズ
        let payload = match serde_json::to_vec(&event) {
            Ok(p) => {
                debug!(subject = %subject, size = p.len(), "Event serialized successfully");
                p
            }
            Err(e) => {
                error!(subject = %subject, error = %e, "Failed to serialize event to JSON");
                return Err(anyhow::anyhow!(e).context("Failed to serialize event to JSON"));
            }
        };

        // JetStreamにパブリッシュ
        debug!(subject = %subject, "Publishing event to JetStream");
        match self
            .js_ctx
            .js
            .publish(subject.clone(), payload.into())
            .await
        {
            Ok(_) => {
                debug!(subject = %subject, "Successfully published event to JetStream");
                Ok(())
            }
            Err(e) => {
                error!(subject = %subject, error = %e, "Failed to publish event to JetStream");
                Err(anyhow::anyhow!(e).context("Failed to publish event to JetStream"))
            }
        }
    }
}

// EpgNotifier 関連の実装を削除
