#![recursion_limit = "512"]

pub mod error_handling;
pub mod event_metadata;
pub mod event_publisher;
pub mod event_subscriber;
pub mod stream_worker;
pub mod worker;

#[cfg(test)]
mod error_handling_test;

#[cfg(test)]
mod event_subscriber_test;

#[cfg(test)]
mod worker_test;

#[cfg(test)]
mod stream_worker_test;
