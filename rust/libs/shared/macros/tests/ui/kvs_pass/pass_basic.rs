//! 必須属性のみを指定した成功ケース

// use async_nats::jetstream::kv::Config; // 不要なので削除
use shared_macros::define_kvs_bucket;
use shared_types::kvs::KvsBucket; // KvsBucket トレイトをインポート

#[define_kvs_bucket(bucket_name = "basic_kv")]
struct BasicKv;

fn main() {
    assert_eq!(BasicKv::BUCKET_NAME, "basic_kv");
    let config = BasicKv::config();
    assert_eq!(config.bucket, "basic_kv");
    // デフォルト値の確認 (一部)
    assert_eq!(config.history, 0); // デフォルト値は 0 のようなので修正
                                   // StorageType は stream モジュールから参照
    assert_eq!(
        config.storage,
        async_nats::jetstream::stream::StorageType::File
    );
}
