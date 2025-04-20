//! リポジトリインターフェース
//!
//! このモジュールはデータアクセスのためのリポジトリインターフェースを定義します。

pub mod kurec_program_repository;
pub mod mirakc_event_repository;
pub mod version_repository;

pub use kurec_program_repository::*;
pub use mirakc_event_repository::*;
pub use version_repository::*;
