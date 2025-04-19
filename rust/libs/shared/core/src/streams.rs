use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

/// ストリーム設定
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// ストリーム名
    pub name: String,
    /// メッセージの最大保持期間
    pub max_age: Option<Duration>,
    /// 最大配信試行回数
    pub max_deliver: Option<u32>,
    /// ACK待機時間
    pub ack_wait: Option<Duration>,
}

// グローバルなストリーム設定レジストリ
static STREAM_CONFIGS: Lazy<RwLock<HashMap<String, StreamConfig>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// ストリーム設定を登録
pub fn register_stream(name: &str, config: StreamConfig) {
    let mut configs = STREAM_CONFIGS.write().unwrap();
    configs.insert(name.to_string(), config);
}

/// 特定のストリーム設定を取得
pub fn get_stream_config(name: &str) -> Option<StreamConfig> {
    let configs = STREAM_CONFIGS.read().unwrap();
    configs.get(name).cloned()
}

/// すべてのストリーム設定を取得
pub fn get_all_stream_configs() -> Vec<StreamConfig> {
    let configs = STREAM_CONFIGS.read().unwrap();
    configs.values().cloned().collect()
}

/// ストリーム設定をクリア（主にテスト用）
#[cfg(test)]
pub fn clear_stream_configs() {
    let mut configs = STREAM_CONFIGS.write().unwrap();
    configs.clear();
}
