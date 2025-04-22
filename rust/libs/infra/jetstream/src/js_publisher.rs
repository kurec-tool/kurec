use anyhow::{Context, Result};
use async_trait::async_trait;
use domain::ports::event_sink::DomainEventSink; // ドメイン層の DomainEventSink をインポート
use shared_core::event::Event; // shared_core の Event をインポート
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::{debug, error, instrument}; // debug, error マクロを追加

// infra_nats クレートの NatsClient をインポート
use infra_nats::NatsClient;

/// JetStreamを使用したイベント発行者
pub struct JsPublisher<E: Event> {
    nats_client: Arc<NatsClient>, // NatsClient を保持
    _phantom: PhantomData<E>,
}

impl<E: Event> JsPublisher<E> {
    /// イベント型から新しいJsPublisherを作成
    pub fn from_event_type(nats_client: Arc<NatsClient>) -> Self {
        // NatsClient を受け取るように変更
        Self {
            nats_client,
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
// ドメイン層の DomainEventSink トレイトを実装
impl<E: Event + 'static> DomainEventSink<E> for JsPublisher<E> {
    #[instrument(skip(self, event), fields(subject = %E::event_name()))] // stream_subject -> event_name
    async fn publish(&self, event: E) -> Result<()> {
        let subject = E::event_name(); // stream_subject -> event_name
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

        // JetStream にパブリッシュ (NatsClient 経由で JetStream コンテキストを取得)
        debug!(subject = %subject, "Publishing event to JetStream");
        match self
            .nats_client // js_ctx -> nats_client
            .jetstream_context() // jetstream_context() メソッドを呼び出す
            .publish(subject.clone(), payload.into())
            .await
        {
            Ok(_) => {
                debug!(subject = %subject, "Successfully published event to JetStream");
                Ok(())
            }
            Err(e) => {
                error!(subject = %subject, error = %e, "Failed to publish event to JetStream");
                // anyhow!(e) -> anyhow::Error::new(e) に変更
                Err(anyhow::Error::new(e).context("Failed to publish event to JetStream"))
            }
        }
    }
}

// EpgNotifier 関連の実装を削除
