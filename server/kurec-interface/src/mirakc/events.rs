use serde::{Deserialize, Serialize};

// これはMirakcのJSON表現ではなく、KuRec内部のイベント表現
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MirakcEventMessage {
    pub tuner_url: String,
    pub event: String, // epg.programs_updatedなど
    pub data: String,  // JSONが文字列としてそのまま入れてある
}

// 以降は上のJSONをデシリアライズするためのコード

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpgProgramsUpdatedMessageData {
    pub service_id: i64,
}
