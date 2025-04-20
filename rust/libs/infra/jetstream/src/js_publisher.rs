use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc; // Arc をインポート
                    // use domain::events::kurec_events::EpgStoredEvent; // EpgNotifier 関連削除
                    // use domain::ports::notifiers::EpgNotifier; // EpgNotifier 関連削除
use shared_core::event_metadata::Event;
use shared_core::event_sink::EventSink; // event_publisher -> event_sink
use std::marker::PhantomData;
use tracing::instrument; // error マクロは不要になったので削除

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
        // イベントをJSONにシリアライズ
        let payload = serde_json::to_vec(&event).context("Failed to serialize event to JSON")?;

        // JetStreamにパブリッシュ
        self.js_ctx
            .js
            .publish(E::stream_subject(), payload.into())
            .await
            .context("Failed to publish event to JetStream")?;

        Ok(())
    }
}

// EpgNotifier 関連の実装を削除
