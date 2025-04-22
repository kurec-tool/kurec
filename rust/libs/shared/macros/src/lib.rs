use proc_macro::TokenStream;

// event, stream モジュールは削除
mod kvs;

// parse_duration は stream マクロで使われていたので削除

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

// tests モジュール内の parse_duration テストも削除
#[cfg(test)]
mod tests {
    // use super::*; // 不要になる可能性
    // use std::time::Duration; // 不要になる可能性

    // test_parse_duration は削除
}
