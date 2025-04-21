//! bucket_name 属性がない場合にコンパイルエラーになることを確認

use shared_macros::define_kvs_bucket;

#[define_kvs_bucket] // bucket_name がない
struct MissingNameKv;

fn main() {}
