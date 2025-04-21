//! JetStream Key-Value Store バケット定義用トレイト

use async_nats::jetstream::kv::Config; // StorageType はマクロ側で処理するため削除
use std::time::Duration;

/// JetStream Key-Value Store バケットを表すトレイト。
///
/// このトレイトは `shared_macros::define_kvs_bucket` マクロによって実装されます。
pub trait KvsBucket {
    /// KVSバケットの名前。
    const BUCKET_NAME: &'static str;

    /// KVSバケットの設定を生成します。
    ///
    /// マクロ属性で指定されなかった項目は `async_nats::jetstream::kv::Config` の
    /// デフォルト値になります。
    fn config() -> Config;
}

// async_nats の StorageType を再エクスポートするか、
// マクロ側で文字列から変換するか検討。ここでは再エクスポートしない方針。

// Placement 構造体も必要に応じて定義する (今回はマクロ内で直接生成)
// pub struct Placement {
//     pub cluster: Option<String>,
//     pub tags: Option<Vec<String>>,
// }
