//! mirakc インフラ層
//!
//! このクレートはmirakcのAPIにアクセスするためのインフラ層の実装を提供します。

pub mod factory; // 追加
pub mod mirakc_api_impl;
pub mod mirakc_client;
pub mod repositories;

// 再エクスポート
pub use factory::MirakcClientFactoryImpl; // 追加
pub use mirakc_api_impl::MirakcApiClientImpl;
pub use mirakc_client::MirakcClient;
pub use repositories::domain_version_repository::DomainVersionRepositoryImpl;
pub use repositories::mirakc_event_repository_impl::MirakcEventRepositoryImpl;
pub use repositories::version_repository_impl::VersionRepositoryImpl;
