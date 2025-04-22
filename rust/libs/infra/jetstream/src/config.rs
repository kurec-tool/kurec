//! JetStream ストリーム設定
use std::time::Duration;

// --- Enums (async_nats::jetstream::stream から再定義 or エイリアス) ---
// async_nats の型を直接使うと、マクロから参照しにくい場合があるため、
// 必要に応じてここで再定義するか、エイリアスを作成する。
// 一旦、async_nats の型をそのまま使うことを試みる。
// マクロ側では `::async_nats::jetstream::stream::RetentionPolicy` のように参照する。
// もし問題があれば、ここで再定義する。

// pub use async_nats::jetstream::stream::{RetentionPolicy, StorageType, DiscardPolicy};
// ↑ use pub はクレートのルート (lib.rs) で行うのが一般的

// --- StreamConfig ---

/// JetStream ストリームの設定を表す構造体。
///
/// `#[define_event_stream]` マクロによってイベント型に関連定数として生成される。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamConfig {
    // 注意: ストリーム名 (name) はここには含めない。
    //       STREAM_NAME 定数として別途生成されるため。
    // 注意: subjects もここには含めない。
    //       サブジェクトはイベント名 (型名から導出) を使うため。
    // 注意: max_consumers もここには含めない。
    //       コンシューマ設定 (ConsumerConfig) で管理するため。
    pub retention: Option<async_nats::jetstream::stream::RetentionPolicy>,
    pub max_messages: Option<i64>, // async_nats は i64
    pub max_bytes: Option<i64>,    // async_nats は i64
    pub max_age: Option<Duration>,
    pub max_message_size: Option<i32>, // async_nats は i32
    pub storage: Option<async_nats::jetstream::stream::StorageType>,
    pub discard: Option<async_nats::jetstream::stream::DiscardPolicy>,
    pub duplicate_window: Option<Duration>,
    pub allow_rollup: Option<bool>,
    pub deny_delete: Option<bool>,
    pub deny_purge: Option<bool>,
    pub description: Option<&'static str>,
    // 必要に応じて他の async_nats::jetstream::stream::Config フィールドを追加
}

// StreamConfig を async_nats::jetstream::stream::Config に変換する From 実装
// (Publisher/Subscriber でストリーム作成/更新時に使用)
impl From<&StreamConfig> for async_nats::jetstream::stream::Config {
    fn from(config: &StreamConfig) -> Self {
        async_nats::jetstream::stream::Config {
            // name と subjects は呼び出し側で設定する
            name: String::new(),  // ダミー、呼び出し側で上書き
            subjects: Vec::new(), // ダミー、呼び出し側で上書き
            retention: config.retention.unwrap_or_default(), // Default は Limits
            max_consumers: -1,    // デフォルト (-1 は無制限)
            max_messages: config.max_messages.unwrap_or(-1),
            max_bytes: config.max_bytes.unwrap_or(-1),
            max_age: config.max_age.unwrap_or_default(), // Default は 0 (無制限)
            max_message_size: config.max_message_size.unwrap_or(-1),
            storage: config.storage.unwrap_or_default(), // Default は File
            discard: config.discard.unwrap_or_default(), // Default は Old
            num_replicas: 1, // デフォルト (変更が必要なら StreamConfig に追加)
            duplicate_window: config.duplicate_window.unwrap_or_default(), // Default は 0
            allow_rollup: config.allow_rollup.unwrap_or(false),
            deny_delete: config.deny_delete.unwrap_or(false),
            deny_purge: config.deny_purge.unwrap_or(false),
            allow_direct: true, // デフォルト (変更が必要なら StreamConfig に追加)
            mirror_direct: false, // デフォルト (変更が必要なら StreamConfig に追加)
            description: config.description.map(|s| s.to_string()),
            // 以下、必要に応じて StreamConfig から設定するフィールドを追加
            ..Default::default() // 他のフィールドは async_nats のデフォルト値を使用
        }
    }
}
