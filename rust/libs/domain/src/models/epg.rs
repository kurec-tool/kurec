use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// mirakcから取得した番組情報をKurecで扱いやすい形式に変換したドメインモデル。
/// KVSへの保存や、後続の処理で利用される。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KurecProgram {
    /// Mirakurun Program ID (mirakc APIのprogram.id)
    pub id: i64,
    /// 番組情報を取得したmirakcのベースURL
    pub mirakc_url: String,
    /// Mirakurun Service ID (mirakc APIのprogram.serviceId)
    pub service_id: i32,
    /// Mirakurun Network ID (mirakc APIのprogram.networkId)
    pub network_id: i32,
    /// Mirakurun Event ID (mirakc APIのprogram.eventId)
    pub event_id: i32,
    /// チャンネル名 (mirakc APIのservice.name から取得)
    pub channel_name: String,
    /// チャンネルタイプ (例: "GR", "BS", "CS") (mirakc APIのservice.channel.type から取得)
    pub channel_type: String,
    /// チャンネル番号 (mirakc APIのservice.channel.channel から取得)
    pub channel: String,
    /// 番組名
    pub name: Option<String>,
    /// 番組説明
    pub description: Option<String>,
    /// 詳細情報 (JSON形式で保持)
    pub extended: Option<serde_json::Value>,
    /// 開始時刻 (Unixタイムスタンプ milliseconds を DateTime<Utc> に変換)
    pub start_at: DateTime<Utc>,
    /// 長さ (ミリ秒)
    pub duration_millis: i64,
    /// 無料放送かどうか
    pub is_free: bool,
    /// ジャンル文字列のリスト (変換後)
    pub genres: Vec<String>,
    /// ビデオ情報文字列 (変換後)
    pub video_info: Option<String>,
    /// オーディオ情報文字列のリスト (変換後)
    pub audio_infos: Vec<String>,
    /// シリーズ情報 (変換後)
    pub series_info: Option<KurecSeriesInfo>,
}

/// Kurecで扱うシリーズ情報
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KurecSeriesInfo {
    pub id: i32,
    pub repeat: i32,
    pub pattern: i32,
    pub expire_at: Option<DateTime<Utc>>, // Unixタイムスタンプ milliseconds を DateTime<Utc> に変換
    pub episode: i32,
    pub last_episode: i32,
    pub name: String,
}

// 必要に応じてテストを追加
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_kurec_program_serialization_deserialization() {
        let dt = Utc.timestamp_millis_opt(1678886400000).unwrap(); // 例: 2023-03-15T12:00:00Z
        let program = KurecProgram {
            id: 12345,
            mirakc_url: "http://mirakc:40772".to_string(),
            service_id: 101,
            network_id: 1,
            event_id: 54321,
            channel_name: "テストチャンネル".to_string(),
            channel_type: "GR".to_string(),
            channel: "27".to_string(),
            name: Some("テスト番組".to_string()),
            description: Some("これはテスト番組です。".to_string()),
            extended: Some(serde_json::json!({"key": "value"})),
            start_at: dt,
            duration_millis: 1800000, // 30分
            is_free: true,
            genres: vec!["ニュース・報道".to_string(), "天気".to_string()],
            video_info: Some("1080i(1125i), アスペクト比16:9 パンベクトルなし".to_string()),
            audio_infos: vec!["2/0モード(ステレオ)".to_string(), "日本語".to_string()],
            series_info: Some(KurecSeriesInfo {
                id: 99,
                repeat: 1,
                pattern: 1,
                expire_at: Some(Utc.timestamp_millis_opt(1710508800000).unwrap()), // 例: 2024-03-15T12:00:00Z
                episode: 5,
                last_episode: 10,
                name: "テストシリーズ".to_string(),
            }),
        };

        let serialized = serde_json::to_string(&program).unwrap();
        let deserialized: KurecProgram = serde_json::from_str(&serialized).unwrap();

        assert_eq!(program, deserialized);
    }
}
