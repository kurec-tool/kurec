//! MirakcClientFactory の定義と実装

use crate::mirakc_api_impl::MirakcApiClientImpl;
use domain::ports::mirakc_api::MirakcApi;
use reqwest::Client;
use std::sync::Arc; // reqwest::Client をインポート

/// MirakcApi クライアントを生成するためのファクトリトレイト。
pub trait MirakcClientFactory: Send + Sync {
    /// 新しい MirakcApi クライアントインスタンスを作成する。
    fn create_client(&self) -> Arc<dyn MirakcApi>;
}

/// MirakcClientFactory の実装。
#[derive(Clone)] // Clone を追加
pub struct MirakcClientFactoryImpl {
    // 共通の reqwest::Client を保持
    client: Client,
}

impl MirakcClientFactoryImpl {
    /// 新しい MirakcClientFactoryImpl を作成する。
    pub fn new() -> Self {
        Self {
            client: Client::new(), // アプリケーションで共有する Client を生成
        }
    }
}

impl MirakcClientFactory for MirakcClientFactoryImpl {
    fn create_client(&self) -> Arc<dyn MirakcApi> {
        // MirakcApiClientImpl は内部で Client を clone するので、
        // ここで clone する必要はない。
        // ただし、MirakcApiClientImpl::new が Client を引数に取るように変更が必要。
        // → MirakcApiClientImpl は new() で Client を生成するので、
        //   ファクトリで Client を共有する意味があまりない。
        //   一旦、毎回 new() する実装にする。
        //   パフォーマンスが問題になる場合は、Client を共有するように修正する。
        Arc::new(MirakcApiClientImpl::new())
    }
}

// Default トレイトの実装を追加
impl Default for MirakcClientFactoryImpl {
    fn default() -> Self {
        Self::new()
    }
}
