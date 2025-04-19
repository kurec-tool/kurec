use serde::{Deserialize, Serialize};
use shared_macros::event;

#[event(stream = "test-stream")]
#[derive(Serialize, Deserialize)]
pub struct TestEvt;

fn main() {
    // コンパイルさえ通れば OK
}
