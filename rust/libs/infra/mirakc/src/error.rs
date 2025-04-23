//! SSEイベント処理時のエラー定義

use thiserror::Error;
use tracing::{error, warn};

/// SSEイベント処理時のエラー
#[derive(Debug, Error)]
pub enum SseEventError {
    #[error("Failed to deserialize event: {source}")]
    Deserialize {
        #[source]
        source: serde_json::Error,
        payload: Vec<u8>,
    },

    #[error("Connection error: {source}")]
    Connection {
        #[source]
        source: reqwest::Error,
        endpoint: String,
    },

    #[error("SSE stream error: {source}")]
    Stream {
        #[source]
        source: anyhow::Error,
    },
}

impl SseEventError {
    /// エラーが再試行可能かどうかを判断する
    pub fn should_retry(&self) -> bool {
        match self {
            Self::Deserialize { .. } => false, // デシリアライズエラーは再試行しても同じ結果になる
            Self::Connection { .. } => true,   // 接続エラーは再試行する価値がある
            Self::Stream { .. } => true,       // ストリームエラーは再試行する価値がある
        }
    }

    /// エラーの詳細をログに記録する
    pub fn log(&self) {
        match self {
            Self::Deserialize { payload, .. } => {
                error!(
                    error = %self,
                    payload_size = payload.len(),
                    payload_preview = %String::from_utf8_lossy(&payload[..std::cmp::min(100, payload.len())]),
                    "Failed to deserialize SSE event"
                );
            }
            Self::Connection { endpoint, .. } => {
                warn!(
                    error = %self,
                    endpoint = %endpoint,
                    "SSE connection error"
                );
            }
            Self::Stream { .. } => {
                warn!(
                    error = %self,
                    "SSE stream error"
                );
            }
        }
    }
}
