use anyhow::Result;
use clap::{Parser, Subcommand};
use infra_jetstream as jetstream;
use std::env;
use tokio::signal;
use tokio_util::sync::CancellationToken;

mod cmd;
mod workers;

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
    worker: Option<WorkerType>,
}

/// 起動可能なワーカーの種類
#[derive(Subcommand, Debug)]
enum WorkerType {
    /// EPG情報を処理するワーカー
    Epg,
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
    // コマンドライン引数を解析
    let cli = Cli::parse();

    // JetStreamに接続
    let nats_url = get_nats_url();
    let js_ctx = std::sync::Arc::new(jetstream::connect(&nats_url).await?);

    // 共通ストリームを設定
    jetstream::setup_all_streams(&js_ctx.js).await?;
    // KuRec 固有リソースを設定
    jetstream::setup_kurec_resources(&js_ctx.js).await?;

    // シャットダウントークンを作成
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // Ctrl+Cでシャットダウン
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("Shutting down...");
        shutdown_clone.cancel();
    });

    // アプリケーション設定を作成
    let app_config = AppConfig {
        nats_url: nats_url.clone(),
    };

    // ワーカーを起動
    match cli.worker.unwrap_or(WorkerType::Epg) {
        WorkerType::Epg => {
            println!("Starting EPG worker...");
            let js_ctx_clone = js_ctx.clone();
            let shutdown_worker = shutdown.clone();
            let _epg_worker_handle = tokio::spawn(async move {
                if let Err(e) =
                    workers::process_epg_event_worker(&js_ctx_clone, shutdown_worker).await
                {
                    eprintln!("EPG worker error: {}", e);
                }
            });
        }
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

            // シャットダウントークンのクローンを作成
            let worker_shutdown = shutdown.clone();

            // mirakcイベント処理コマンドを実行
            if let Err(e) =
                cmd::mirakc_events::run_mirakc_events(&app_config, &mirakc_url, worker_shutdown)
                    .await
            {
                eprintln!("mirakc events worker error: {}", e);
                std::process::exit(1);
            }

            // 正常終了
            shutdown.cancel();
        }
        WorkerType::EpgUpdater => {
            // mirakc_url を削除
            println!("Starting EPG updater worker..."); // URL表示を削除

            // シャットダウントークンのクローンを作成
            let worker_shutdown = shutdown.clone();

            // EPG更新ワーカーを実行 (mirakc_url 引数を削除)
            let _epg_updater_handle = tokio::spawn(async move {
                if let Err(e) = cmd::epg_updater::run_epg_updater(&app_config, worker_shutdown) // mirakc_url を削除
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
    fn test_cli_default_worker() {
        // 引数なしの場合
        let cli = Cli::parse_from(["app"]);

        // デフォルトでは None が返される
        assert!(cli.worker.is_none());
    }

    #[test]
    fn test_cli_epg_worker() {
        // EPGワーカーを指定
        let cli = Cli::parse_from(["app", "epg"]);

        // WorkerType::Epg が返される
        match cli.worker {
            Some(WorkerType::Epg) => (),
            _ => panic!("Expected WorkerType::Epg"),
        }
    }
}
