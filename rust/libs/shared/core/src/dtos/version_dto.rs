use serde::{Deserialize, Serialize};

/// mirakcのバージョン情報DTO
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VersionDto {
    /// 現在のバージョン
    pub current: String,
    /// 最新のバージョン
    pub latest: String,
}
