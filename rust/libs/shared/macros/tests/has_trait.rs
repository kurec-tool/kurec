use serde::{Deserialize, Serialize};
use shared_macros::event;

#[event(stream = "test")]
#[derive(Serialize, Deserialize)]
pub struct MyEvt;

#[test]
fn event_trait() {
    // このテストは何もしません
    // コンパイルが通れば成功
}
