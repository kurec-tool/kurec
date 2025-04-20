//! KVS (Key-Value Store) インフラストラクチャ実装
//!
//! このクレートは、ドメイン層で定義されたリポジトリトレイト (`KurecProgramRepository`) を
//! 具体的なKVS技術 (現在はNATS KVを想定) を用いて実装します。

pub mod error;
pub mod nats_kv; // NATS KV実装モジュール

// 必要に応じて他のKVS実装モジュールを追加 (例: redis, memory)

// 設定に応じて実装を切り替えるファクトリ関数などを定義することも可能
