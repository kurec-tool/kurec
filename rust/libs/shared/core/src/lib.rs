#![recursion_limit = "512"]

use shared_types::stream::Stream;

pub mod dtos;
pub mod error_handling;
pub mod event; // 追加
pub mod event_metadata;
pub mod event_sink; // 追加
pub mod event_source; // 追加
pub mod repositories;
pub mod stream_worker;
pub mod streams;
pub mod worker;

#[cfg(test)]
mod error_handling_test;

// #[cfg(test)] // 削除
// mod event_subscriber_test; // 削除

#[cfg(test)]
mod worker_test;

#[cfg(test)]
mod stream_worker_test;
