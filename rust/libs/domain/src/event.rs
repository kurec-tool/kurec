use serde::{de::DeserializeOwned, Serialize};

/// ドメインイベントを示すマーカートレイト。
///
/// イベントはシリアライズ/デシリアライズ可能で、スレッドセーフである必要があります。
pub trait Event: Serialize + DeserializeOwned + Send + Sync + 'static {}

// 注意: 以前の Event トレイトにあった event_name() は削除されました。
// イベントのサブジェクト名は、インフラ層 (例: JetStreamPublisher) が
// イベントの型名から自動的に導出します (例: ProgramUpdated -> "program_updated")。
