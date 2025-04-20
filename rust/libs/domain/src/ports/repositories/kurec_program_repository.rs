use anyhow::Result;
use async_trait::async_trait;

use crate::models::epg::KurecProgram;

/// Kurecで扱う番組情報 (`KurecProgram`) を永続化するためのリポジトリトレイト。
/// 主にKVS (Key-Value Store) への保存・取得を想定。
#[async_trait]
pub trait KurecProgramRepository: Send + Sync {
    /// 指定されたmirakc URLとサービスIDに対応する番組情報リストを保存する。
    /// 既存のデータは上書きされることを想定。
    ///
    /// # Arguments
    ///
    /// * `mirakc_url` - 番組情報を取得したmirakcのベースURL
    /// * `service_id` - Mirakurun Service ID
    /// * `programs` - 保存する番組情報のリスト (`Vec<KurecProgram>`)
    ///
    /// # Returns
    ///
    /// 保存に成功した場合は `Ok(())`、失敗した場合は `Err`。
    async fn save_service_programs(
        &self,
        mirakc_url: &str,
        service_id: i32,
        programs: Vec<KurecProgram>,
    ) -> Result<()>;

    /// 指定されたmirakc URLとサービスIDに対応する番組情報リストを取得する。
    ///
    /// # Arguments
    ///
    /// * `mirakc_url` - 番組情報を取得したmirakcのベースURL
    /// * `service_id` - Mirakurun Service ID
    ///
    /// # Returns
    ///
    /// データが存在する場合は `Ok(Some(Vec<KurecProgram>))`、存在しない場合は `Ok(None)`、
    /// 取得に失敗した場合は `Err`。
    async fn get_service_programs(
        &self,
        mirakc_url: &str,
        service_id: i32,
    ) -> Result<Option<Vec<KurecProgram>>>;

    // 必要に応じて他のメソッドを追加 (例: delete_service_programs)
}
