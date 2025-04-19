use serde::{de::DeserializeOwned, Serialize};

/// イベントの基本トレイト
///
/// このトレイトを実装する型は、JetStreamを通じて送受信できるイベントとなる。
/// ストリーム名とイベント名を指定することで、サブジェクト名が自動的に生成される。
pub trait Event: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// イベントが属するストリーム名
    fn stream_name() -> &'static str;

    /// イベント名
    fn event_name() -> &'static str;

    /// サブジェクト名（ストリーム名.イベント名）
    fn stream_subject() -> String {
        format!("{}.{}", Self::stream_name(), Self::event_name())
    }
}
