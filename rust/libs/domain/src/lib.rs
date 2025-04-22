//! ドメイン層
//!
//! このクレートはドメインモデル、DTOとリポジトリインターフェースを定義します。

pub mod dtos;
pub mod event; // Event トレイトを定義するモジュール
pub mod events; // イベント構造体を定義するモジュール
pub mod handlers; // 追加
pub mod models;
pub mod ports;
pub mod usecases;
