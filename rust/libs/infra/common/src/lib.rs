//! インフラ層の共通機能を提供するクレート
//!
//! このクレートは、イベントのAck機能やイベントソースの抽象化など、
//! インフラ層で共通して使用される機能を提供します。

pub mod ack;
pub mod ackable_event;
pub mod event_source;
