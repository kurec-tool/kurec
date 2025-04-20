use thiserror::Error;

/// `infra_kvs` クレート固有のエラー型。
#[derive(Error, Debug)]
pub enum KvsError {
    /// NATSクライアント関連のエラー
    #[error("NATS client error: {0}")]
    NatsClient(#[from] async_nats::Error),

    // NatsKvStore と NatsKvEntry は削除し、anyhow でラップする
    // /// NATS KVストア操作のエラー
    // #[error("NATS KV store error: {0}")]
    // NatsKvStore(#[from] async_nats::jetstream::kv::StoreError),

    // /// NATS KVエントリー操作のエラー
    // #[error("NATS KV entry error: {0}")]
    // NatsKvEntry(#[from] async_nats::jetstream::kv::EntryError),
    /// シリアライズ/デシリアライズエラー
    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// データが見つからない場合のエラー (get操作などで使用)
    #[error("Data not found for key: {0}")]
    NotFound(String),

    /// その他のエラー (Anyhow経由)
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, KvsError>;
