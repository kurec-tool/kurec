use serde::{Deserialize, Serialize};
use shared_core::event_metadata::Event;
use shared_core::streams::{get_all_stream_configs, get_stream_config};
use shared_macros::{define_streams, event};

// ストリーム定義
define_streams! {
    stream test_stream {
        max_age: "1h",
        max_deliver: 5,
        ack_wait: "1m",
    }
}

// イベント型の定義
#[event(stream = "test_stream")]
#[derive(Serialize, Deserialize, Debug)]
struct TestEvent;

#[test]
fn stream_config_registered() {
    // ストリーム定義を登録（define_streamsマクロが#[ctor::ctor]を使用しているため、
    // テスト実行前に自動的に実行されるはずだが、明示的に登録する）
    shared_core::streams::register_stream(
        "test_stream",
        shared_core::streams::StreamConfig {
            name: "test_stream".to_string(),
            max_age: Some(std::time::Duration::from_secs(3600)), // 1h
            max_deliver: Some(5),
            ack_wait: Some(std::time::Duration::from_secs(60)), // 1m
        },
    );

    // 登録されたストリーム設定を取得
    let configs = get_all_stream_configs();
    assert!(!configs.is_empty(), "ストリーム設定が登録されていません");

    // test_streamの設定を取得
    let config = get_stream_config("test_stream");
    assert!(config.is_some(), "test_streamの設定が見つかりません");

    let config = config.unwrap();
    assert_eq!(config.name, "test_stream");
    assert_eq!(config.max_deliver, Some(5));

    // イベントのストリーム名を確認
    assert_eq!(TestEvent::stream_name(), "test_stream");
    assert_eq!(TestEvent::event_name(), "test_event");
    assert_eq!(TestEvent::stream_subject(), "test_stream.test_event");
}
