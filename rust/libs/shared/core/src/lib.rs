#![recursion_limit = "512"]

pub mod dtos;
pub mod error_handling;
pub mod event_metadata;
pub mod event_publisher;
pub mod event_subscriber;
pub mod repositories;
pub mod stream_worker;
pub mod streams;
pub mod streams_def;
pub mod worker;

// ストリーム定義の初期化
pub fn init_streams() {
    streams_def::init_streams();
}

#[cfg(test)]
mod error_handling_test;

#[cfg(test)]
mod event_subscriber_test;

#[cfg(test)]
mod worker_test;

#[cfg(test)]
mod stream_worker_test;
