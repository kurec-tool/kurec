//! ストリーム定義
//!
//! このモジュールはJetStreamストリームの定義を提供します。

use shared_types;
use shared_types::stream::Stream;
use shared_types::stream::{DiscardPolicy, RetentionPolicy, StorageType, StreamConfig};
use std::time::Duration;

/// mirakcイベントストリーム
pub struct MirakcEvents;

impl Stream for MirakcEvents {
    const NAME: &'static str = "mirakc-events";

    fn config() -> StreamConfig {
        StreamConfig {
            name: Self::NAME.to_string(),
            subjects: None,
            retention: Some(RetentionPolicy::Limits),
            max_consumers: None,
            max_msgs: None,
            max_bytes: None,
            max_age: Some(Duration::from_secs(60 * 60 * 24 * 7)), // 7d
            max_msg_size: None,
            storage: Some(StorageType::File),
            discard: Some(DiscardPolicy::Old),
            duplicate_window: None,
            allow_rollup: None,
            deny_delete: None,
            deny_purge: None,
            description: Some("mirakc events stream".to_string()),
        }
    }
}

/// kurecイベントストリーム
pub struct KurecEvents;

impl Stream for KurecEvents {
    const NAME: &'static str = "kurec-events";

    fn config() -> StreamConfig {
        StreamConfig {
            name: Self::NAME.to_string(),
            subjects: None,
            retention: Some(RetentionPolicy::Limits),
            max_consumers: None,
            max_msgs: None,
            max_bytes: None,
            max_age: Some(Duration::from_secs(60 * 60 * 24 * 7)), // 7d
            max_msg_size: None,
            storage: Some(StorageType::File),
            discard: Some(DiscardPolicy::Old),
            duplicate_window: None,
            allow_rollup: None,
            deny_delete: None,
            deny_purge: None,
            description: Some("kurec events stream".to_string()),
        }
    }
}
