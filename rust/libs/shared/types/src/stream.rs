use serde::{Deserialize, Serialize};
use std::time::Duration;

/// ストリーム設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// ストリーム名
    pub name: String,
    /// サブジェクト名のリスト（指定しない場合は name.* が使用される）
    pub subjects: Option<Vec<String>>,
    /// 保持ポリシー
    pub retention: Option<RetentionPolicy>,
    /// 最大コンシューマー数
    pub max_consumers: Option<u32>,
    /// 最大メッセージ数
    pub max_msgs: Option<u64>,
    /// 最大バイト数
    pub max_bytes: Option<u64>,
    /// 最大保持期間
    pub max_age: Option<Duration>,
    /// 最大メッセージサイズ
    pub max_msg_size: Option<u32>,
    /// ストレージタイプ
    pub storage: Option<StorageType>,
    /// 破棄ポリシー
    pub discard: Option<DiscardPolicy>,
    /// 重複検出ウィンドウ
    pub duplicate_window: Option<Duration>,
    /// ロールアップを許可するかどうか
    pub allow_rollup: Option<bool>,
    /// 削除を拒否するかどうか
    pub deny_delete: Option<bool>,
    /// パージを拒否するかどうか
    pub deny_purge: Option<bool>,
    /// 説明
    pub description: Option<String>,
}

/// 保持ポリシー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionPolicy {
    /// メッセージ数・サイズ・期間の制限に基づく
    Limits,
    /// コンシューマーの関心に基づく
    Interest,
    /// ワークキュー（処理されたメッセージは削除）
    WorkQueue,
}

/// ストレージタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    /// ファイルベースのストレージ
    File,
    /// メモリベースのストレージ
    Memory,
}

/// 破棄ポリシー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscardPolicy {
    /// 古いメッセージを破棄
    Old,
    /// 新しいメッセージを拒否
    New,
}

/// ストリーム定義を表すトレイト
pub trait Stream: 'static {
    /// ストリーム名
    const NAME: &'static str;

    /// ストリーム設定を取得
    fn config() -> StreamConfig;
}
