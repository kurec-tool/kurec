use anyhow::Result;
use serde::{Deserialize, Serialize};

/// mirakcのバージョン情報
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Version {
    /// 現在のバージョン
    pub current: String,
    /// 最新のバージョン
    pub latest: String,
}

/// バージョン状態
#[derive(Debug, PartialEq, Eq)]
pub enum VersionStatus {
    /// 最新
    UpToDate,
    /// パッチバージョンの更新あり
    PatchUpdate,
    /// マイナーバージョンの更新あり
    MinorUpdate,
    /// メジャーバージョンの更新あり
    MajorUpdate,
    /// 開発版
    Development,
}

impl Version {
    /// バージョン情報を解析
    pub fn parse_versions(&self) -> Result<(semver::Version, semver::Version)> {
        let current = semver::Version::parse(&self.current)?;
        let latest = semver::Version::parse(&self.latest)?;
        Ok((current, latest))
    }

    /// 開発版かどうかを確認
    pub fn is_dev_version(&self) -> bool {
        self.current.contains("-dev.")
    }

    /// 最新バージョンかどうかを確認
    pub fn is_latest(&self) -> Result<bool> {
        let (current, latest) = self.parse_versions()?;
        Ok(current >= latest)
    }

    /// バージョン状態を取得
    pub fn version_status(&self) -> Result<VersionStatus> {
        // 開発版の場合
        if self.is_dev_version() {
            return Ok(VersionStatus::Development);
        }

        let (current, latest) = self.parse_versions()?;

        if current >= latest {
            return Ok(VersionStatus::UpToDate);
        }

        if current.major < latest.major {
            return Ok(VersionStatus::MajorUpdate);
        } else if current.minor < latest.minor {
            return Ok(VersionStatus::MinorUpdate);
        } else if current.patch < latest.patch {
            return Ok(VersionStatus::PatchUpdate);
        }

        Ok(VersionStatus::UpToDate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dev_version() {
        let dev_version = Version {
            current: "4.0.0-dev.0".to_string(),
            latest: "3.9.0".to_string(),
        };
        assert!(dev_version.is_dev_version());

        let stable_version = Version {
            current: "3.9.0".to_string(),
            latest: "3.9.0".to_string(),
        };
        assert!(!stable_version.is_dev_version());
    }

    #[test]
    fn test_parse_versions() {
        let version = Version {
            current: "1.2.3".to_string(),
            latest: "2.3.4".to_string(),
        };

        let (current, latest) = version.parse_versions().unwrap();
        assert_eq!(current.major, 1);
        assert_eq!(current.minor, 2);
        assert_eq!(current.patch, 3);
        assert_eq!(latest.major, 2);
        assert_eq!(latest.minor, 3);
        assert_eq!(latest.patch, 4);
    }

    #[test]
    fn test_parse_dev_version() {
        let version = Version {
            current: "1.2.3-dev.4".to_string(),
            latest: "1.2.0".to_string(),
        };

        let (current, _latest) = version.parse_versions().unwrap();
        assert_eq!(current.major, 1);
        assert_eq!(current.minor, 2);
        assert_eq!(current.patch, 3);

        // プレリリース部分を文字列として検証
        assert!(current.pre.len() > 0);
        assert_eq!(current.pre.to_string(), "dev.4");
    }

    #[test]
    fn test_version_status_dev() {
        let dev_version = Version {
            current: "4.0.0-dev.0".to_string(),
            latest: "3.9.0".to_string(),
        };
        assert_eq!(
            dev_version.version_status().unwrap(),
            VersionStatus::Development
        );
    }

    #[test]
    fn test_version_status_up_to_date() {
        let version = Version {
            current: "1.0.0".to_string(),
            latest: "1.0.0".to_string(),
        };
        assert_eq!(version.version_status().unwrap(), VersionStatus::UpToDate);
    }

    #[test]
    fn test_version_status_patch_update() {
        let version = Version {
            current: "1.0.0".to_string(),
            latest: "1.0.1".to_string(),
        };
        assert_eq!(
            version.version_status().unwrap(),
            VersionStatus::PatchUpdate
        );
    }

    #[test]
    fn test_version_status_minor_update() {
        let version = Version {
            current: "1.0.0".to_string(),
            latest: "1.1.0".to_string(),
        };
        assert_eq!(
            version.version_status().unwrap(),
            VersionStatus::MinorUpdate
        );
    }

    #[test]
    fn test_version_status_major_update() {
        let version = Version {
            current: "1.0.0".to_string(),
            latest: "2.0.0".to_string(),
        };
        assert_eq!(
            version.version_status().unwrap(),
            VersionStatus::MajorUpdate
        );
    }

    #[test]
    fn test_invalid_version_format() {
        let invalid_version = Version {
            current: "invalid".to_string(),
            latest: "1.0.0".to_string(),
        };
        assert!(invalid_version.parse_versions().is_err());
        assert!(invalid_version.version_status().is_err());
    }
}
