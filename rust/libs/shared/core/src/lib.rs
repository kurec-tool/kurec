#![recursion_limit = "512"]

// use shared_types::stream::Stream; // 削除

pub mod dtos;
pub mod error_handling;
pub mod event; // 追加 (これは domain::event とは別？ 確認が必要)
               // pub mod event_metadata; // 削除
               // pub mod event_sink; // domain::ports に移動
               // pub mod event_source; // domain::ports に移動
pub mod repositories;
// pub mod stream_worker; // app::worker に移動
pub mod streams; // これは残す？ 中身を確認
                 // pub mod worker; // app::worker に移動

#[cfg(test)]
mod error_handling_test;

// #[cfg(test)] // 削除
// mod event_subscriber_test; // 削除

// worker_test と stream_worker_test は app::worker に移動したため削除
