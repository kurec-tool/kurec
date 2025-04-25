use crate::event::Event;
use infra_macros::define_event_stream;
use serde::{Deserialize, Serialize};

/// EPG情報がKVSに保存されたことを示すイベント。
/// 後続のワーカー (例: Meilisearch登録ワーカー) をトリガーするために使用される。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[define_event_stream(stream = "kurec-epg-updated")] // ストリーム名を設定 (サブジェクト名は型名から自動導出)
pub struct EpgStoredEvent {
    /// 番組情報を取得したmirakcのベースURL
    pub mirakc_url: String,
    /// 更新されたEPGのサービスID (Mirakurun Service ID)
    pub service_id: i64,
}
impl Event for EpgStoredEvent {}

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
