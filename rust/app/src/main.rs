use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use domain::events::MirakcEventInput; // MirakcEventInput をインポート
use domain::{
    events::{kurec_events::EpgStoredEvent, mirakc_events::EpgProgramsUpdatedEvent},
    handlers::mirakc_event_handler::MirakcEventSinks,
    ports::{event_sink::EventSink, event_source::EventSource},
};
use infra_jetstream::{self, JsPublisher, JsSubscriber}; // infra_jetstream とその要素をインポート
use infra_mirakc::MirakcSseSource; // MirakcSseSource をインポート
use std::{env, sync::Arc}; // Arc をインポート
use tokio::signal;
use tokio_util::sync::CancellationToken;

mod cmd;
mod streams_def;

/// アプリケーション設定
pub struct AppConfig {
    /// NATS接続URL
    pub nats_url: String,
}

/// KuRec アプリケーション CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 起動するワーカーの種類
    #[command(subcommand)]
    worker: WorkerType,
}

/// 起動可能なワーカーの種類
#[derive(Subcommand, Debug)]
enum WorkerType {
    /// mirakcのバージョンを確認
    CheckVersion {
        /// mirakcサーバーのURL
        #[arg(long, default_value = "http://localhost:40772")]
        mirakc_url: String,
    },
    /// mirakcのイベントを処理するワーカー
    MirakcEvents {
        /// mirakcサーバーのURL
        #[arg(long, default_value = "http://localhost:40772")]
        mirakc_url: String,
    },
    /// EPG更新イベントを処理するワーカー
    EpgUpdater, // mirakc_url 引数を削除
                // 将来的に他のワーカーを追加する場合はここに追加
}

/// 環境変数NATS_URLからNATS接続URLを取得する
/// 環境変数が設定されていない場合はデフォルト値を返す
fn get_nats_url() -> String {
    env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt::init();

    // コマンドライン引数を解析
    let cli = Cli::parse();

    // NATS に接続
    let nats_url = get_nats_url();
    let nats_client = infra_nats::connect(&nats_url)
        .await
        .context("NATS への接続に失敗しました")?;

    // 共通ストリームを設定
    infra_jetstream::setup_all_streams(nats_client.jetstream_context()) // jetstream:: -> infra_jetstream::
        .await
        .context("JetStream ストリームのセットアップに失敗しました")?;
    // KuRec 固有リソースの設定は infra_nats または infra_kvs で行うため削除
    // jetstream::setup_kurec_resources(&js_ctx.js).await?;

    // シャットダウントークンを作成
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // Ctrl+Cでシャットダウン
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("Shutting down...");
        shutdown_clone.cancel();
    });

    // ワーカーを起動
    match cli.worker {
        WorkerType::CheckVersion { mirakc_url } => {
            println!("mirakcバージョンを確認中: {}...", mirakc_url);

            // バージョンリポジトリとユースケースを作成
            let version_repo = infra_mirakc::repositories::domain_version_repository::DomainVersionRepositoryImpl::new(&mirakc_url);
            let version_usecase =
                domain::usecases::version_usecase::VersionUseCase::new(version_repo);

            // バージョン状態を取得
            match version_usecase.get_version_status().await {
                Ok((version, status)) => {
                    println!("現在のバージョン: {}", version.current);
                    println!("最新バージョン: {}", version.latest);

                    match status {
                        domain::models::version::VersionStatus::UpToDate => {
                            println!("✅ mirakcは最新バージョンです");
                        }
                        domain::models::version::VersionStatus::PatchUpdate => {
                            println!("⚠️ パッチアップデートが利用可能です");
                        }
                        domain::models::version::VersionStatus::MinorUpdate => {
                            println!("⚠️ マイナーアップデートが利用可能です");
                        }
                        domain::models::version::VersionStatus::MajorUpdate => {
                            println!("⚠️ メジャーアップデートが利用可能です");
                        }
                        domain::models::version::VersionStatus::Development => {
                            println!("🔧 開発版を使用中です");

                            // 開発版と最新版の比較情報も表示
                            if let Ok((current, latest)) = version.parse_versions() {
                                if current.major != latest.major
                                    || current.minor != latest.minor
                                    || current.patch != latest.patch
                                {
                                    println!(
                                        "  開発版のベースバージョン: {}.{}.{}",
                                        current.major, current.minor, current.patch
                                    );
                                    println!(
                                        "  最新安定版: {}.{}.{}",
                                        latest.major, latest.minor, latest.patch
                                    );
                                }
                            }
                        }
                    }

                    // 正常終了
                    shutdown.cancel();
                }
                Err(e) => {
                    eprintln!("mirakcバージョン確認エラー: {}", e);
                    std::process::exit(1);
                }
            }
        }
        WorkerType::MirakcEvents { mirakc_url } => {
            println!("Starting mirakc events worker with URL: {}...", mirakc_url);

            // 依存関係の初期化
            // EventSource の型パラメータを MirakcEventInput に変更
            let mirakc_source: Arc<dyn EventSource<MirakcEventInput>> =
                Arc::new(MirakcSseSource::new(mirakc_url.clone()));

            // TODO: MirakcEventSinks の初期化 (必要な Sink を作成して渡す)
            // 例: let epg_updated_sink = Arc::new(JsPublisher::new(nats_client.clone(), "mirakc-events"));
            //     let sinks = MirakcEventSinks { epg_programs_updated: Some(epg_updated_sink), ... };
            let sinks = MirakcEventSinks::default(); // 仮実装

            // シャットダウントークンのクローンを作成
            let worker_shutdown = shutdown.clone();

            // mirakcイベント処理コマンドを実行
            if let Err(e) =
                cmd::mirakc_events::run_mirakc_events(mirakc_source, sinks, worker_shutdown).await
            {
                eprintln!("mirakc events worker error: {}", e);
                std::process::exit(1);
            }

            // 正常終了
            shutdown.cancel();
        }
        WorkerType::EpgUpdater => {
            println!("Starting EPG updater worker...");

            // 依存関係の初期化
            // ストリーム定義を取得
            let mirakc_stream = streams_def::mirakc_event_stream();
            let kurec_stream = streams_def::kurec_event_stream();

            // JsSubscriber と JsPublisher を作成
            let epg_updated_source: Arc<dyn EventSource<EpgProgramsUpdatedEvent>> = Arc::new(
                JsSubscriber::<EpgProgramsUpdatedEvent>::new(nats_client.clone(), mirakc_stream),
            );
            let epg_stored_sink: Arc<dyn EventSink<EpgStoredEvent>> =
                Arc::new(JsPublisher::<EpgStoredEvent>::new(
                    nats_client.clone(),
                    kurec_stream,
                ));

            // シャットダウントークンのクローンを作成
            let worker_shutdown = shutdown.clone();

            // EPG更新ワーカーを実行
            let _epg_updater_handle = tokio::spawn(async move {
                if let Err(e) = cmd::epg_updater::run_epg_updater(
                    epg_updated_source,
                    epg_stored_sink,
                    worker_shutdown,
                )
                .await
                {
                    eprintln!("EPG updater worker error: {}", e);
                }
            });
        }
    }

    // シャットダウンを待機
    shutdown.cancelled().await;
    println!("Shutdown complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory as _;

    use super::*;

    #[test]
    fn test_get_nats_url_default() {
        // 環境変数をクリア
        env::remove_var("NATS_URL");

        // デフォルト値が返されることを確認
        assert_eq!(get_nats_url(), "nats://localhost:4222");
    }

    #[test]
    fn test_get_nats_url_custom() {
        // 環境変数を設定
        env::set_var("NATS_URL", "nats://example.com:4222");

        // 設定した値が返されることを確認
        assert_eq!(get_nats_url(), "nats://example.com:4222");

        // テスト後に環境変数をクリア
        env::remove_var("NATS_URL");
    }
    #[test]
    fn test_cli_structure() {
        // CLI構造が正しく生成されることを確認
        Cli::command().debug_assert();
    }

    #[test]
    fn test_cli_check_version() {
        // CheckVersionサブコマンドの引数を解析
        let args = vec!["app", "check-version", "--mirakc-url", "http://example.com"];
        let cli = Cli::parse_from(args);

        if let WorkerType::CheckVersion { mirakc_url } = cli.worker {
            assert_eq!(mirakc_url, "http://example.com");
        } else {
            panic!("Expected WorkerType::CheckVersion");
        }
    }

    #[test]
    fn test_cli_mirakc_events() {
        // MirakcEventsサブコマンドの引数を解析
        let args = vec!["app", "mirakc-events", "--mirakc-url", "http://example.com"];
        let cli = Cli::parse_from(args);

        if let WorkerType::MirakcEvents { mirakc_url } = cli.worker {
            assert_eq!(mirakc_url, "http://example.com");
        } else {
            panic!("Expected WorkerType::MirakcEvents");
        }
    }

    #[test]
    fn test_cli_epg_updater() {
        // EpgUpdaterサブコマンドの引数を解析
        let args = vec!["app", "epg-updater"];
        let cli = Cli::parse_from(args);

        if let WorkerType::EpgUpdater = cli.worker {
            // 正常にEpgUpdaterが選択されることを確認
        } else {
            panic!("Expected WorkerType::EpgUpdater");
        }
    }
}
