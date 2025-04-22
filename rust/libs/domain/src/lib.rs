//! ドメイン層
//!
//! このクレートはドメインモデル、DTOとリポジトリインターフェースを定義します。

pub mod dtos;
// pub mod event; // shared_core に移動
pub mod events;
pub mod handlers; // 追加
pub mod models;
pub mod ports;
pub mod usecases;
