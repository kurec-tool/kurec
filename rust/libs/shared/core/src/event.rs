use serde::{de::DeserializeOwned, Serialize};

/// イベントのメタデータを定義するトレイト
///
/// イベントの種類を一意に識別するための情報を提供します。
/// また、シリアライズ/デシリアライズ可能であることを要求します。
pub trait Event: Serialize + DeserializeOwned + Send + Sync + 'static {}
