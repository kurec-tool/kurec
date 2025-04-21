#![recursion_limit = "512"]

use shared_types::stream::Stream;

pub mod dtos;
pub mod error_handling;
pub mod event_metadata;
pub mod event_sink; // 追加
pub mod event_source; // 追加
pub mod repositories;
pub mod stream_worker;
pub mod streams;
pub mod worker;

// ストリーム定義の初期化
pub fn init_streams() {
    // TODO inventoryを見て全部初期化する
}

#[cfg(test)]
mod error_handling_test;

// #[cfg(test)] // 削除
// mod event_subscriber_test; // 削除

#[cfg(test)]
mod worker_test;

#[cfg(test)]
mod stream_worker_test;
