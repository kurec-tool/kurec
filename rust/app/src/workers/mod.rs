// ワーカーモジュール
pub mod epg_worker;

// ワーカー実行関数をエクスポート
pub use epg_worker::run_process_epg_event;
