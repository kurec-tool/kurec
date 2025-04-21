use serde::{Deserialize, Serialize};
use shared_types::stream::{DiscardPolicy, RetentionPolicy, StorageType, Stream, StreamConfig};
use std::time::Duration;

// イベント型の定義
#[derive(Serialize, Deserialize, Debug)]
struct TestEvent;

impl Stream for TestEvent {
    const NAME: &'static str = "test-stream";

    fn config() -> StreamConfig {
        StreamConfig {
            name: Self::NAME.to_string(),
            subjects: None,
            retention: Some(RetentionPolicy::Limits),
            max_consumers: None,
            max_msgs: None,
            max_bytes: None,
            max_age: Some(Duration::from_secs(60 * 60)), // 1h
            max_msg_size: None,
            storage: Some(StorageType::File),
            discard: Some(DiscardPolicy::Old),
            duplicate_window: None,
            allow_rollup: None,
            deny_delete: None,
            deny_purge: None,
            description: Some("test stream".to_string()),
        }
    }
}

#[test]
fn stream_config_registered() {
    // イベントのストリーム名を確認
    assert_eq!(TestEvent::NAME, "test-stream");
    assert_eq!(TestEvent::config().name, "test-stream");
}
