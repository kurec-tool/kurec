use crate::error::PublishError; // エラー型をインポート
use anyhow::{Context, Result};
use async_nats::jetstream;
use async_trait::async_trait;
use domain::event::Event; // 新しい Event トレイトをインポート
use domain::ports::event_sink::EventSink;
use heck::ToSnakeCase; // スネークケース変換用
use std::any::type_name; // 型名取得用
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn}; // info, warn を追加

// infra_nats クレートの NatsClient をインポート
use infra_nats::NatsClient;

/// JetStreamを使用したイベント発行者
pub struct JsPublisher<E: Event> {
    nats_client: Arc<NatsClient>,
    event_stream: crate::event_stream::EventStream<E>,
}

impl<E: Event> JsPublisher<E> {
    /// 新しいJsPublisherを作成
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

// ヘルパー関数: 型名からスネークケースのサブジェクト名を取得
fn type_name_to_snake_case<T: Event>() -> String {
    let type_name_full = type_name::<T>();
    // フルパスから型名部分を抽出 (例: my_crate::events::MyEvent -> MyEvent)
    let type_name_short = type_name_full.split("::").last().unwrap_or(type_name_full);
    type_name_short.to_snake_case()
}

#[async_trait]
impl<E> EventSink<E> for JsPublisher<E>
where
    // Event トレイトを実装し、関連定数を持つことを示す (マクロが保証)
    // Send + Sync + 'static は async_trait と Arc のために必要
    E: Event + Send + Sync + 'static,
{
    #[instrument(
        skip(self, event),
        fields(
            stream = %self.event_stream.stream_name(), // EventStream からストリーム名を取得
            subject = %type_name_to_snake_case::<E>() // 型名から導出
        )
    )]
    async fn publish(&self, event: E) -> Result<()> {
        let stream_name = self.event_stream.stream_name();
        let subject = type_name_to_snake_case::<E>(); // 型名からサブジェクト名を生成

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

        debug!(subject = %subject, "Serializing event for JetStream");
        let payload = serde_json::to_vec(&event)
            .with_context(|| format!("Failed to serialize event {} to JSON", subject))?;
        debug!(subject = %subject, size = payload.len(), "Event serialized successfully");

        let js_ctx = self.nats_client.jetstream_context();

        // --- ストリームの存在確認と作成 ---
        match js_ctx.get_stream(stream_name).await {
            Ok(_) => {
                debug!(stream = %stream_name, "Stream already exists");
                // TODO: ストリーム設定が STREAM_CONFIG と一致するか確認・更新する？
                //       現状は既存の設定をそのまま使う。
            }
            Err(err) => {
                // get_stream のエラーが StreamNotFound かどうかを判定
                // async_nats 0.40 時点ではエラーの種類を直接判定する良い方法がないため、
                // 文字列マッチングで判定する (将来的に改善される可能性あり)
                if err.to_string().contains("stream not found") {
                    warn!(stream = %stream_name, "Stream not found, attempting to create it");
                    // StreamConfig から async_nats::jetstream::stream::Config を作成
                    let mut nats_config: async_nats::jetstream::stream::Config =
                        (&stream_config).into();
                    nats_config.name = stream_name.to_string();
                    // サブジェクトを設定 (特定のサブジェクトのみ許可)
                    nats_config.subjects = vec![subject.clone()];

                    match js_ctx.create_stream(nats_config).await {
                        Ok(stream_info) => {
                            info!(stream = %stream_name, "Successfully created stream");
                        }
                        Err(create_err) => {
                            error!(stream = %stream_name, error = %create_err, "Failed to create stream");
                            return Err(anyhow::Error::new(create_err)
                                .context(format!("Failed to create stream {}", stream_name)));
                        }
                    }
                } else {
                    // StreamNotFound 以外のエラー
                    error!(stream = %stream_name, error = %err, "Failed to get stream info");
                    return Err(anyhow::Error::new(err)
                        .context(format!("Failed to get stream info for {}", stream_name)));
                }
            }
        }

        // --- イベントの発行 ---
        debug!(subject = %subject, "Publishing event to JetStream");
        match js_ctx.publish(subject.clone(), payload.into()).await {
            Ok(_) => {
                debug!(subject = %subject, "Successfully published event to JetStream");
                Ok(())
            }
            Err(e) => {
                error!(subject = %subject, error = %e, "Failed to publish event to JetStream");
                Err(anyhow::Error::new(e).context("Failed to publish event to JetStream"))
            }
        }
    }
}
