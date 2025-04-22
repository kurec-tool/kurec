use serde::{Deserialize, Serialize};
use shared_core::event::Event; // shared_core::event::Event をインポート
                               // use shared_core::event_metadata::Event; // #[event] マクロが自動で実装するため不要
use shared_macros::event; // #[event] マクロをインポート

/// EPG情報がKVSに保存されたことを示すイベント。
/// 後続のワーカー (例: Meilisearch登録ワーカー) をトリガーするために使用される。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Event トレイトを実装
#[event(stream = "kurec-epg-updated", subject = "epg.stored")] // 仮のストリーム名とサブジェクトプレフィックス
pub struct EpgStoredEvent {
    /// 番組情報を取得したmirakcのベースURL
    pub mirakc_url: String,
    /// 更新されたEPGのサービスID (Mirakurun Service ID)
    pub service_id: i64, // i32 -> i64
}

// EpgStoredEvent に Event トレイトを手動実装 (マクロが機能しない場合の確認用)
// TODO: マクロが修正されたら削除
// impl Event for EpgStoredEvent {
impl Event for EpgStoredEvent {
    fn event_name() -> &'static str {
        "epg_stored" // #[event] マクロが生成するであろう名前
    }
    fn stream_name() -> &'static str {
        "kurec-epg-updated" // #[event(stream = "...")] で指定された名前
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epg_stored_event_serialization_deserialization() {
        let event = EpgStoredEvent {
            mirakc_url: "http://mirakc.local:40772".to_string(),
            service_id: 101,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: EpgStoredEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}
