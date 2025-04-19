use serde::{Deserialize, Serialize};
use shared_macros::event;

#[event(stream = "test-stream")]
#[derive(Serialize, Deserialize)]
pub enum TestEvt {
    Evt1,
}

fn main() {
    // コンパイルさえ通れば OK
}
