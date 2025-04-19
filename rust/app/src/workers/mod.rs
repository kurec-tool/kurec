// ワーカーモジュール
pub mod epg_worker;

// ワーカー実行関数をエクスポート
pub use epg_worker::process_epg_event_worker;
