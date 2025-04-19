pub use inventory;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub trait Event: Serialize + DeserializeOwned + HasStreamDef + Send + Sync + 'static {}

/// デフォルト構成を持つ JetStream ストリーム定義
#[derive(Debug)]
pub struct StreamConfigDefaults {
    pub max_age: Option<Duration>,
    pub max_deliver: Option<u32>,
    pub ack_wait: Option<Duration>,
}

/// イベントストリームの定義
#[derive(Debug)]
pub struct StreamDef {
    pub name: &'static str,
    pub subjects: &'static [&'static str],
    pub default_config: StreamConfigDefaults,
}

/// イベント型がこのトレイトを実装すると、定義が取得できるようになる
pub trait HasStreamDef {
    fn stream_name() -> &'static str;
    fn stream_subject() -> &'static str;
}

// 全てのStreamDefはinventory経由で収集される（#[event]で登録）
inventory::collect!(StreamDef);
