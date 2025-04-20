//! EPG更新ワーカーコマンド
//!
//! このモジュールはEPG更新イベントを処理するコマンドを提供します。

use anyhow::{Context, Result};
use async_trait::async_trait; // EpgNotifier の実装に必要
use domain::{
    events::{kurec_events::EpgStoredEvent, mirakc_events::EpgProgramsUpdatedEvent},
    handlers::epg_update_handler::EpgUpdateHandler, // 新しいハンドラ
    ports::{
        notifiers::epg_notifier::EpgNotifier as EpgNotifierTrait, // EpgNotifier トレイト
        repositories::kurec_program_repository::KurecProgramRepository,
    },
};
use infra_jetstream::{JsPublisher, JsSubscriber}; // 新しい Source/Sink 実装
use infra_kvs::nats_kv::NatsKvProgramRepository;
use infra_mirakc::factory::MirakcClientFactoryImpl;
use shared_core::{
    event_sink::EventSink,       // EventSink トレイト
    event_source::EventSource,   // EventSource トレイト
    stream_worker::StreamWorker, // StreamWorker
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::info;

// EpgNotifier トレイトを実装するためのラッパー構造体
struct JsEpgNotifier {
    // EpgStoredEvent を発行できる EventSink (JsPublisher) を保持
    sink: Arc<dyn EventSink<EpgStoredEvent>>,
}

impl JsEpgNotifier {
    fn new(sink: Arc<dyn EventSink<EpgStoredEvent>>) -> Self {
        Self { sink }
    }
}

#[async_trait]
impl EpgNotifierTrait for JsEpgNotifier {
    async fn notify_epg_stored(&self, event: EpgStoredEvent) -> Result<()> {
        self.sink.publish(event).await
    }
}

/// EPG更新ワーカーを実行 (StreamWorker を使用)
pub async fn run_epg_updater(config: &crate::AppConfig, shutdown: CancellationToken) -> Result<()> {
    info!("Starting EPG updater worker...");

    // JetStreamの設定
    let js_ctx = Arc::new(infra_jetstream::connect(&config.nats_url).await?);
    infra_jetstream::setup_all_streams(&js_ctx.js).await?; // ストリーム設定は必要

    // KVSストアを取得または作成
    let kv_store = js_ctx
        .js
        .create_key_value(async_nats::jetstream::kv::Config {
            bucket: "kurec_epg".to_string(),
            ..Default::default()
        })
        .await
        .context("Failed to create/get KV store")?;

    // KVSリポジトリの作成
    let program_repository: Arc<dyn KurecProgramRepository> =
        Arc::new(NatsKvProgramRepository::new(kv_store));

    // MirakcClientFactory の作成
    let mirakc_api_factory = Arc::new(MirakcClientFactoryImpl::new());

    // EventSource (JsSubscriber) の作成
    let source: Arc<dyn EventSource<EpgProgramsUpdatedEvent>> =
        Arc::new(JsSubscriber::from_event_type(js_ctx.clone())); // js_ctx.clone() を渡す

    // EventSink (JsPublisher) の作成 (EpgStoredEvent 用)
    let sink: Arc<dyn EventSink<EpgStoredEvent>> =
        Arc::new(JsPublisher::from_event_type(js_ctx.clone())); // js_ctx.clone() を渡す

    // EpgNotifier (JsEpgNotifier) の作成
    let epg_notifier: Arc<dyn EpgNotifierTrait> = Arc::new(JsEpgNotifier::new(sink.clone()));

    // StreamHandler (EpgUpdateHandler) の作成
    // TODO: EpgUpdateHandler::new に mirakc_api_factory を渡すように修正が必要
    let handler = Arc::new(EpgUpdateHandler::new(
        program_repository,
        epg_notifier,
        // mirakc_api_factory, // TODO: ハンドラに追加
    ));

    // StreamWorker の構築
    // 入力: EpgProgramsUpdatedEvent, 出力: Option<EpgStoredEvent>, エラー: EpgUpdateError
    // sink は Option<EpgStoredEvent> を直接扱えないため、StreamWorker 内部で Some の場合のみ publish される
    let worker: StreamWorker<EpgProgramsUpdatedEvent, EpgStoredEvent, _> =
        StreamWorker::new(source, sink, handler).durable("epg-updater-worker"); // durable name を設定

    // ワーカーの実行
    info!("Starting StreamWorker for EPG updates...");
    worker.run(shutdown).await?;

    info!("EPG updater worker stopped gracefully.");
    Ok(())
}
