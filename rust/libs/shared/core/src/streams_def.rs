//! ストリーム定義
//!
//! このモジュールはJetStreamストリームの定義を提供します。

use crate::streams::{register_stream, StreamConfig};
use std::time::Duration;

/// ストリーム定義を初期化
pub fn init_streams() {
    // mirakcイベントストリーム
    register_stream(
        "mirakc-events",
        StreamConfig {
            name: "mirakc-events".to_string(),
            // 最大保持期間: 7日間
            max_age: Some(Duration::from_secs(7 * 24 * 60 * 60)),
            // 最大配信試行回数: 10回
            max_deliver: Some(10),
            // ACK待機時間: 30秒
            ack_wait: Some(Duration::from_secs(30)),
        },
    );
}
