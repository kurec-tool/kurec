use serde::{Deserialize, Serialize};
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
