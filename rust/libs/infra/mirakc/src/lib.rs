//! mirakc インフラ層
//!
//! このクレートはmirakcのAPIにアクセスするためのインフラ層の実装を提供します。

pub mod mirakc_client;
pub mod repositories;

// 再エクスポート
pub use mirakc_client::MirakcClient;
pub use repositories::version_repository_impl::VersionRepositoryImpl;
pub use repositories::domain_version_repository::DomainVersionRepositoryImpl;
