//! mirakcイベント処理コマンド
//!
//! このモジュールはmirakcイベントを処理するコマンドを提供します。

use async_trait::async_trait;
use domain::usecases::mirakc_event_usecase::MirakcEventUseCase;
use infra_mirakc::MirakcEventRepositoryImpl;
use shared_core::event_publisher::EventPublisher;
use std::sync::Arc;
use tracing::{error, info};

/// mirakcイベント処理コマンドを実行
pub async fn run_mirakc_events(
    config: &crate::AppConfig,
    mirakc_url: &str,
    shutdown: tokio_util::sync::CancellationToken,
) -> anyhow::Result<()> {
    info!("Starting mirakc events command with URL: {}", mirakc_url);

    // JetStreamの設定
    let js_ctx = infra_jetstream::connect(&config.nats_url).await?;
    infra_jetstream::setup_all_streams(&js_ctx.js).await?;

    // リポジトリの作成
    let repository = MirakcEventRepositoryImpl::new(mirakc_url.to_string());

    // 各イベント型に対応するパブリッシャーを作成
    let publisher = CombinedPublisher::new(Arc::new(js_ctx));

    // ユースケースの作成と実行
    let usecase = MirakcEventUseCase::new(repository, publisher).with_shutdown(shutdown);

    info!("Starting to process mirakc events...");
    match usecase.process_events().await {
        Ok(_) => {
            info!("Mirakc events processing completed");
            Ok(())
        }
        Err(e) => {
            error!("Error processing mirakc events: {:?}", e);
            Err(e)
        }
    }
}

/// 複数のイベント型に対応するパブリッシャー
pub struct CombinedPublisher {
    js_ctx: Arc<infra_jetstream::JetStreamCtx>,
}

impl CombinedPublisher {
    /// 新しいCombinedPublisherを作成
    pub fn new(js_ctx: Arc<infra_jetstream::JetStreamCtx>) -> Self {
        Self { js_ctx }
    }
}

// 各イベント型に対するEventPublisherの実装
macro_rules! impl_event_publisher {
    ($event_type:ty) => {
        #[async_trait]
        impl EventPublisher<$event_type> for CombinedPublisher {
            async fn publish(&self, event: $event_type) -> anyhow::Result<()> {
                // イベントをJSONにシリアライズ
                let data = serde_json::to_vec(&event)?;

                // イベントのストリーム名を取得
                let stream = <$event_type as shared_core::event_metadata::Event>::stream_subject();

                // JetStreamに公開
                self.js_ctx.js.publish(stream, data.into()).await?;

                Ok(())
            }
        }
    };
}

// 各イベント型に対するEventPublisherを実装
impl_event_publisher!(domain::events::mirakc_events::TunerStatusChangedEvent);
impl_event_publisher!(domain::events::mirakc_events::EpgProgramsUpdatedEvent);
impl_event_publisher!(domain::events::mirakc_events::RecordingStartedEvent);
impl_event_publisher!(domain::events::mirakc_events::RecordingStoppedEvent);
impl_event_publisher!(domain::events::mirakc_events::RecordingFailedEvent);
impl_event_publisher!(domain::events::mirakc_events::RecordingRescheduledEvent);
impl_event_publisher!(domain::events::mirakc_events::RecordingRecordSavedEvent);
impl_event_publisher!(domain::events::mirakc_events::RecordingRecordRemovedEvent);
impl_event_publisher!(domain::events::mirakc_events::RecordingContentRemovedEvent);
impl_event_publisher!(domain::events::mirakc_events::RecordingRecordBrokenEvent);
impl_event_publisher!(domain::events::mirakc_events::OnairProgramChangedEvent);
