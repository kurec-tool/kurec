use serde::{Deserialize, Serialize};

use crate::models::version::Version;

/// mirakcのバージョン情報DTO
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VersionDto {
    /// 現在のバージョン
    pub current: String,
    /// 最新のバージョン
    pub latest: String,
}

impl From<VersionDto> for Version {
    fn from(dto: VersionDto) -> Self {
        Self {
            current: dto.current,
            latest: dto.latest,
        }
    }
}
