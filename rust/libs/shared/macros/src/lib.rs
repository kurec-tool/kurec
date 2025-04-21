use proc_macro::TokenStream;
#[cfg(test)]
use std::time::Duration;

mod event;
mod kvs;
mod stream; // KVSバケット定義マクロ用モジュール

#[cfg(test)]
fn parse_duration(opt: &Option<String>) -> Option<Duration> {
    opt.as_ref().map(|s| {
        humantime::parse_duration(s)
            .unwrap_or_else(|e| panic!("invalid duration literal `{}`: {}", s, e))
    })
}

#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::event_impl(attr, item)
}

/// ストリームを定義するマクロ
///
/// このマクロは、型にStreamトレイトを実装し、自動的にストリームレジストリに登録します。
///
/// # 属性パラメータ
///
/// - `name = "ストリーム名"`: ストリーム名を指定します（省略時は型名をケバブケースに変換）
/// - `max_age = "期間"`: メッセージの最大保持期間を指定します（例: "7d", "24h"）
/// - `max_msgs = 数値`: 最大メッセージ数を指定します
/// - `max_bytes = 数値`: 最大バイト数を指定します
/// - `max_msg_size = 数値`: 最大メッセージサイズを指定します
/// - `storage = "タイプ"`: ストレージタイプを指定します（"file" または "memory"）
/// - `discard = "ポリシー"`: 破棄ポリシーを指定します（"old" または "new"）
/// - `duplicate_window = "期間"`: 重複検出ウィンドウを指定します（例: "2m", "1h"）
/// - `allow_rollup = 真偽値`: ロールアップを許可するかどうかを指定します
/// - `deny_delete = 真偽値`: 削除を拒否するかどうかを指定します
/// - `deny_purge = 真偽値`: パージを拒否するかどうかを指定します
/// - `description = "説明"`: ストリームの説明を指定します
///
/// # 使用例
///
/// ```ignore
/// use shared_macros::stream;
///
/// #[stream(max_age = "7d")]
/// enum MirakcEvents;
///
/// #[stream(
///     max_age = "3d",
///     max_msgs = 10000,
///     storage = "file",
///     discard = "old",
///     description = "KuRec events stream"
/// )]
/// enum KurecEvents;
/// ```
#[proc_macro_attribute]
pub fn stream(attr: TokenStream, item: TokenStream) -> TokenStream {
    stream::stream_impl(attr, item)
}

/// KVSバケットを定義するマクロ
///
/// このマクロは、型に KvsBucket トレイトを実装します。
///
/// # 属性パラメータ
///
/// - `bucket_name = "バケット名"`: (必須) KVSバケットの名前を指定します。
/// - `description = "説明"`: バケットの説明。
/// - `max_value_size = 数値`: 値の最大サイズ (i32)。
/// - `history = 数値`: 履歴保持数 (i64)。
/// - `max_age = "期間"`: エントリの最大生存期間 (例: "7d", "1h")。
/// - `max_bytes = 数値`: バケットの最大バイト数 (i64)。
/// - `storage = "タイプ"`: ストレージタイプ ("file" または "memory")。
/// - `num_replicas = 数値`: レプリカ数 (usize)。
/// - `mirror_direct = 真偽値`: ミラーの直接アクセス許可。
/// - `compression = 真偽値`: 圧縮の有効/無効。
/// - `placement_cluster = "クラスタ名"`: 配置クラスタ名。
/// - `placement_tags = ["タグ1", "タグ2"]`: 配置タグ。
///
/// # 使用例
///
/// ```ignore
/// use shared_macros::define_kvs_bucket;
///
/// #[define_kvs_bucket(bucket_name = "my_kv_store", max_age = "1d")]
/// struct MyKvStore;
/// ```
#[proc_macro_attribute]
pub fn define_kvs_bucket(attr: TokenStream, item: TokenStream) -> TokenStream {
    kvs::kvs_bucket_impl(attr, item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parse_duration() {
        assert_eq!(
            parse_duration(&Some("1s".to_string())),
            Some(Duration::new(1, 0))
        );
        assert_eq!(
            parse_duration(&Some("1m".to_string())),
            Some(Duration::new(60, 0))
        );
        assert_eq!(
            parse_duration(&Some("1h".to_string())),
            Some(Duration::new(3600, 0))
        );
        assert_eq!(parse_duration(&None), None);
    }
}
