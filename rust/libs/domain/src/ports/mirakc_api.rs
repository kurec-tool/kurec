//! mirakc API クライアントのインターフェース定義

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value; // 具体的な型ではなく汎用的な Value を使う

/// mirakc API との通信を行うためのトレイト。
#[async_trait]
pub trait MirakcApi: Send + Sync {
    /// 指定されたサービスIDのサービス情報を取得する。
    ///
    /// # Arguments
    ///
    /// * `mirakc_url` - 接続先の mirakc のベースURL
    /// * `service_id` - Mirakurun Service ID
    ///
    /// # Returns
    ///
    /// サービス情報 (JSON Value)。見つからない場合やエラー時は `Err`。
    async fn get_service(&self, mirakc_url: &str, service_id: i64) -> Result<Value>; // u64 -> i64 に戻す

    /// 指定されたサービスIDの番組情報リストを取得する。
    ///
    /// # Arguments
    ///
    /// * `mirakc_url` - 接続先の mirakc のベースURL
    /// * `service_id` - Mirakurun Service ID
    ///
    /// # Returns
    ///
    /// 番組情報リスト (JSON Value の Vec)。エラー時は `Err`。
    async fn get_programs_of_service(
        &self,
        mirakc_url: &str,
        service_id: i64, // u64 -> i64 に戻す
    ) -> Result<Vec<Value>>;

    // 必要に応じて他のAPIメソッドを追加 (例: get_version)
}
