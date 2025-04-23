//! イベント定義
//!
//! このモジュールはシステム内で使用されるイベントを定義します。

use chrono::{DateTime, Utc};
use serde::Deserialize;

pub mod kurec_events;
pub mod mirakc_events;

/// mirakcから受信したイベントの生データを表す構造体
/// (以前の MirakcEventDto に相当)
#[derive(Debug, Clone, Deserialize)]
pub struct MirakcEventInput {
    pub mirakc_url: String,
    pub event_type: String,
    pub data: String, // JSON string
    pub received_at: DateTime<Utc>,
}

pub use kurec_events::*;
pub use mirakc_events::*;
