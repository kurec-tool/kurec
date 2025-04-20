//! EPG更新ワーカーコマンド
//!
//! このモジュールはEPG更新イベントを処理するコマンドを提供します。

use anyhow::{Context, Result};
use async_nats;
use domain::events::kurec_events::EpgStoredEvent;
use domain::events::mirakc_events::EpgProgramsUpdatedEvent;
use domain::ports::notifiers::epg_notifier::EpgNotifier as EpgNotifierTrait;
use domain::ports::repositories::kurec_program_repository::KurecProgramRepository;
use futures::StreamExt;
use infra_jetstream::JetStreamCtx;
use infra_kvs::nats_kv::NatsKvProgramRepository;
use infra_mirakc::factory::MirakcClientFactoryImpl; // Factory 実装をインポート
                                                    // use infra_mirakc::mirakc_client::MirakcClient; // 直接使わない
use shared_core::error_handling::ClassifyError;
use shared_core::event_metadata::Event;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::workers::epg_updater_worker::EpgUpdaterWorker;

/// EPG更新ワーカーを実行
pub async fn run_epg_updater(
    config: &crate::AppConfig,
    // mirakc_url: &str, // 引数から削除
    shutdown: CancellationToken,
) -> Result<()> {
    info!("Starting EPG updater worker..."); // URL表示を削除

    // JetStreamの設定
    let js_ctx = Arc::new(infra_jetstream::connect(&config.nats_url).await?);
    infra_jetstream::setup_all_streams(&js_ctx.js).await?;

    // KVSストアを取得または作成
    let kv_store = js_ctx
        .js
        .create_key_value(async_nats::jetstream::kv::Config {
            bucket: "kurec_epg".to_string(), // バケット名を指定
            ..Default::default()
        })
        .await
        .context("Failed to create/get KV store")?;

    // KVSリポジトリの作成
    let program_repository: Arc<dyn KurecProgramRepository> =
        Arc::new(NatsKvProgramRepository::new(kv_store));

    // EPG通知機能の作成
    let epg_notifier = Arc::new(EpgNotifierImpl::new(js_ctx.clone()));

    // MirakcClientFactory の作成
    let mirakc_api_factory = Arc::new(MirakcClientFactoryImpl::new());

    // EPG更新ワーカーの作成 (Factory を渡す)
    let worker = EpgUpdaterWorker::new(program_repository, epg_notifier, mirakc_api_factory);

    // ワーカーの実行
    info!("Starting to process EPG update events...");

    // サブスクリプションの設定
    let subscription = subscribe_to_epg_programs_updated_events(&js_ctx, shutdown.clone()).await?;

    // イベント処理ループ
    let mut messages = subscription; // PullSubscription から Stream を取得
    while let Some(result) = messages.next().await {
        if shutdown.is_cancelled() {
            break;
        }

        match result {
            Ok(msg) => {
                debug!("Received message: {:?}", msg);
                // メッセージからイベントをデシリアライズ
                match serde_json::from_slice::<EpgProgramsUpdatedEvent>(&msg.message.payload) {
                    // msg.message.payload を使用
                    Ok(event) => {
                        info!(
                            "Received EPG programs updated event for service_id={}",
                            event.data.service_id
                        );

                        // イベントを処理
                        match worker.process_epg_programs_updated(event.clone()).await {
                            Ok(_) => {
                                // 処理成功
                                if let Err(e) = msg.ack().await {
                                    // msg.ack() を使用
                                    error!("Failed to ack message: {}", e);
                                }
                            }
                            Err(e) => {
                                // エラー処理
                                error!("Error processing EPG programs updated event: {}", e);

                                // エラーの種類に応じて再試行するかどうかを決定
                                match e.error_action() {
                                    shared_core::error_handling::ErrorAction::Retry => {
                                        // Pull Consumer では NAK はないので、ACK しないことで再配信を待つ
                                        info!("Will retry processing message later for event service_id={}", event.data.service_id);
                                    }
                                    shared_core::error_handling::ErrorAction::Ignore => {
                                        // 無視してACKする
                                        if let Err(e) = msg.ack().await {
                                            // msg.ack() を使用
                                            error!("Failed to ack message: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize EPG programs updated event: {}", e);
                        // デシリアライズエラーは再試行しても解決しないのでACKする
                        if let Err(e) = msg.ack().await {
                            // msg.ack() を使用
                            error!("Failed to ack message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error receiving message from JetStream: {}", e);
                // エラーが続く場合は少し待機
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }

    info!("EPG updater worker stopped");
    Ok(())
}

/// EPG更新イベントをサブスクライブ
async fn subscribe_to_epg_programs_updated_events(
    js_ctx: &Arc<JetStreamCtx>,
    _shutdown: CancellationToken, // shutdown は pull subscription では直接使わない
) -> Result<async_nats::jetstream::consumer::pull::Stream> {
    // ストリーム名とサブジェクト
    let stream_name = "mirakc-events";
    let subject = "mirakc-events.epg_programs_updated_event";

    // コンシューマー設定 (フィールドに直接設定)
    let consumer_config = async_nats::jetstream::consumer::pull::Config {
        durable_name: Some("epg-updater-worker".to_string()),
        filter_subject: subject.to_string(),
        max_deliver: 10,                              // Option 不要
        ack_wait: std::time::Duration::from_secs(30), // Option 不要
        ..Default::default()
    };

    // ストリームを取得
    let stream = js_ctx.js.get_stream(stream_name).await?;

    // コンシューマーの作成 (IntoConsumerConfig を使う)
    let consumer = stream.create_consumer(consumer_config).await?;

    // メッセージストリームを取得 (consumer.messages() を使用)
    let message_stream = consumer.messages().await?;

    Ok(message_stream)
}

// EpgNotifierTrait の実装
pub struct EpgNotifierImpl {
    js_ctx: Arc<JetStreamCtx>,
}

impl EpgNotifierImpl {
    /// 新しいEpgNotifierを作成
    pub fn new(js_ctx: Arc<JetStreamCtx>) -> Self {
        Self { js_ctx }
    }
}

#[async_trait::async_trait]
impl EpgNotifierTrait for EpgNotifierImpl {
    async fn notify_epg_stored(&self, event: EpgStoredEvent) -> Result<()> {
        // イベントをJSONにシリアライズ
        let data = serde_json::to_vec(&event)?;

        // イベントのストリーム名を取得
        let stream_subject = EpgStoredEvent::stream_subject();

        // JetStreamに公開
        self.js_ctx.js.publish(stream_subject, data.into()).await?;

        Ok(())
    }
}
